use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{ChildStdin, Command};
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::config::SharedConfig;

const OUTPUT_BUFFER_LIMIT: usize = 1_000_000;

#[derive(Clone)]
pub struct TerminalManager {
    config: SharedConfig,
    sessions: Arc<RwLock<HashMap<String, Arc<TerminalSession>>>>,
}

impl TerminalManager {
    pub fn new(config: SharedConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn ensure_started(&self, session_id: &str, workspace_path: &str) -> Result<()> {
        self.ensure_session(session_id, workspace_path).await?;
        Ok(())
    }

    async fn ensure_session(
        &self,
        session_id: &str,
        workspace_path: &str,
    ) -> Result<Arc<TerminalSession>> {
        if let Some(existing) = self.sessions.read().await.get(session_id).cloned() {
            if existing.is_active() {
                return Ok(existing);
            }
        }

        let created = TerminalSession::spawn(
            session_id.to_string(),
            PathBuf::from(workspace_path),
            self.config.terminal.command.clone(),
        )
        .await?;

        let mut sessions = self.sessions.write().await;
        match sessions.get(session_id) {
            Some(existing) if existing.is_active() => Ok(existing.clone()),
            _ => {
                sessions.insert(session_id.to_string(), created.clone());
                Ok(created)
            }
        }
    }

    pub async fn handle_socket(
        &self,
        socket: WebSocket,
        session_id: String,
        workspace_path: String,
    ) -> Result<()> {
        let terminal = self.ensure_session(&session_id, &workspace_path).await?;
        let snapshot = terminal.snapshot().await;
        let mut subscription = terminal.subscribe();
        let (mut sender, mut receiver) = socket.split();

        let snapshot_message = serde_json::to_string(&TerminalSocketMessage::Snapshot {
            cwd: snapshot.cwd,
            shell: snapshot.shell,
            data: snapshot.data,
            active: snapshot.active,
        })?;
        sender.send(Message::Text(snapshot_message.into())).await?;

        let mut forward_task = tokio::spawn(async move {
            loop {
                match subscription.recv().await {
                    Ok(message) => {
                        let payload = match serde_json::to_string(&message) {
                            Ok(payload) => payload,
                            Err(_) => continue,
                        };
                        if sender.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        let terminal_for_input = terminal.clone();
        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                match message {
                    Message::Text(payload) => {
                        let Ok(client_message) =
                            serde_json::from_str::<TerminalClientMessage>(&payload)
                        else {
                            continue;
                        };

                        match client_message {
                            TerminalClientMessage::Input { data } => {
                                if terminal_for_input.write_input(&data).await.is_err() {
                                    break;
                                }
                            }
                            TerminalClientMessage::Resize { cols, rows } => {
                                let _ = (cols, rows);
                            }
                        }
                    }
                    Message::Binary(_) => {}
                    Message::Ping(_) => {}
                    Message::Pong(_) => {}
                    Message::Close(_) => break,
                }
            }
        });

        tokio::select! {
            _ = &mut forward_task => {
                receive_task.abort();
            }
            _ = &mut receive_task => {
                forward_task.abort();
            }
        }

        Ok(())
    }
}

struct TerminalSession {
    cwd: String,
    shell: String,
    active: AtomicBool,
    stdin: Mutex<ChildStdin>,
    history: Mutex<String>,
    broadcaster: broadcast::Sender<TerminalSocketMessage>,
}

impl TerminalSession {
    async fn spawn(
        session_id: String,
        workspace_path: PathBuf,
        shell: String,
    ) -> Result<Arc<Self>> {
        if !workspace_path.exists() {
            anyhow::bail!(
                "workspace path does not exist: {}",
                workspace_path.display()
            );
        }

        let mut command = Command::new("/usr/bin/script");
        command
            .arg("-q")
            .arg("/dev/null")
            .arg(&shell)
            .arg("-i")
            .current_dir(&workspace_path)
            .env("TERM", "xterm-256color")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().with_context(|| {
            format!(
                "failed to start terminal command `{shell}` in {}",
                workspace_path.display()
            )
        })?;

        let stdin = child
            .stdin
            .take()
            .context("terminal stdin was not available")?;
        let stdout = child
            .stdout
            .take()
            .context("terminal stdout was not available")?;
        let stderr = child
            .stderr
            .take()
            .context("terminal stderr was not available")?;

        let (broadcaster, _) = broadcast::channel(256);
        let session = Arc::new(Self {
            cwd: workspace_path.display().to_string(),
            shell: shell.clone(),
            active: AtomicBool::new(true),
            stdin: Mutex::new(stdin),
            history: Mutex::new(String::new()),
            broadcaster,
        });

        session
            .publish(TerminalSocketMessage::Status {
                status: "connected".to_string(),
                message: Some(format!(
                    "Attached terminal for workspace {session_id} using `{shell}`."
                )),
                exit_code: None,
            })
            .await;

        Self::spawn_output_reader(stdout, session.clone());
        Self::spawn_output_reader(stderr, session.clone());
        Self::spawn_exit_watcher(child, session.clone());

        Ok(session)
    }

    fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    fn subscribe(&self) -> broadcast::Receiver<TerminalSocketMessage> {
        self.broadcaster.subscribe()
    }

    async fn snapshot(&self) -> TerminalSnapshot {
        TerminalSnapshot {
            cwd: self.cwd.clone(),
            shell: self.shell.clone(),
            data: self.history.lock().await.clone(),
            active: self.is_active(),
        }
    }

    async fn write_input(&self, data: &str) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(data.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    fn spawn_output_reader<R>(reader: R, session: Arc<Self>)
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            let mut reader = reader;
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(read) => {
                        let chunk = String::from_utf8_lossy(&buffer[..read]).to_string();
                        session
                            .publish(TerminalSocketMessage::Output { data: chunk })
                            .await;
                    }
                    Err(error) => {
                        session
                            .publish(TerminalSocketMessage::Error {
                                message: format!("Terminal stream error: {error}"),
                            })
                            .await;
                        break;
                    }
                }
            }
        });
    }

    fn spawn_exit_watcher(mut child: tokio::process::Child, session: Arc<Self>) {
        tokio::spawn(async move {
            let status = child.wait().await;
            session.active.store(false, Ordering::SeqCst);
            match status {
                Ok(status) => {
                    session
                        .publish(TerminalSocketMessage::Status {
                            status: "exited".to_string(),
                            message: Some("Terminal session ended.".to_string()),
                            exit_code: status.code(),
                        })
                        .await;
                }
                Err(error) => {
                    session
                        .publish(TerminalSocketMessage::Error {
                            message: format!("Terminal session failed: {error}"),
                        })
                        .await;
                }
            }
        });
    }

    async fn publish(&self, message: TerminalSocketMessage) {
        if let TerminalSocketMessage::Output { data } = &message {
            self.append_history(data).await;
        }
        let _ = self.broadcaster.send(message);
    }

    async fn append_history(&self, chunk: &str) {
        let mut history = self.history.lock().await;
        history.push_str(chunk);

        if history.len() <= OUTPUT_BUFFER_LIMIT {
            return;
        }

        let trim_to = trim_offset(&history, history.len() - OUTPUT_BUFFER_LIMIT);
        history.drain(..trim_to);
    }
}

fn trim_offset(buffer: &str, minimum: usize) -> usize {
    if minimum == 0 {
        return 0;
    }

    if let Some(relative) = buffer[minimum..].find('\n') {
        minimum + relative + 1
    } else {
        minimum
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TerminalSocketMessage {
    Snapshot {
        cwd: String,
        shell: String,
        data: String,
        active: bool,
    },
    Output {
        data: String,
    },
    Status {
        status: String,
        message: Option<String>,
        exit_code: Option<i32>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TerminalClientMessage {
    Input { data: String },
    Resize { cols: u16, rows: u16 },
}

struct TerminalSnapshot {
    cwd: String,
    shell: String,
    data: String,
    active: bool,
}
