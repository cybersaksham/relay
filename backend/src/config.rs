use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub slack: SlackConfig,
    pub authorization: AuthorizationConfig,
    pub codex: CodexConfig,
    pub terminal: TerminalConfig,
    pub hooks: HookConfig,
    pub paths: PathConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub portal_base_url: String,
    pub rust_log: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct SlackConfig {
    pub bot_token: String,
    pub app_token: String,
}

#[derive(Debug, Clone)]
pub struct AuthorizationConfig {
    pub master_slack_user_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct CodexConfig {
    pub bin: String,
    pub default_args: Vec<String>,
    pub timeout_seconds: u64,
    pub browser_task_timeout_seconds: u64,
    pub playwright_cli_wrapper: PathBuf,
    pub playwright_cli_preflight_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct HookConfig {
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct PathConfig {
    pub relay_home: PathBuf,
    pub sources_dir: PathBuf,
    pub workspaces_dir: PathBuf,
    pub policies_dir: PathBuf,
    pub workflows_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let current_dir = env::current_dir().context("failed to resolve current directory")?;
        let relay_home = env_path("RELAY_HOME")
            .or_else(default_relay_home)
            .unwrap_or_else(|| current_dir.join(".relay"));
        let sources_dir =
            env_path("RELAY_SOURCES_DIR").unwrap_or_else(|| relay_home.join("sources"));
        let workspaces_dir =
            env_path("RELAY_WORKSPACES_DIR").unwrap_or_else(|| relay_home.join("workspaces"));
        let policies_dir =
            env_path("RELAY_POLICIES_DIR").unwrap_or_else(|| current_dir.join("../.policies"));
        let workflows_dir =
            env_path("RELAY_WORKFLOWS_DIR").unwrap_or_else(|| current_dir.join("../.workflows"));
        let app_base_url = required_env("APP_BASE_URL")?;
        let portal_base_url = env::var("PORTAL_BASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| app_base_url.clone());

        Ok(Self {
            server: ServerConfig {
                host: env_or("APP_HOST", "127.0.0.1"),
                port: required_env("APP_PORT")?
                    .parse()
                    .context("APP_PORT must be a valid u16")?,
                base_url: app_base_url,
                portal_base_url,
                rust_log: env_or("RUST_LOG", "relay_backend=debug,tower_http=info"),
            },
            database: DatabaseConfig {
                url: env_or("DATABASE_URL", "sqlite:relay.db"),
            },
            slack: SlackConfig {
                bot_token: env::var("SLACK_BOT_TOKEN").unwrap_or_default(),
                app_token: env::var("SLACK_APP_TOKEN").unwrap_or_default(),
            },
            authorization: AuthorizationConfig {
                master_slack_user_ids: env::var("MASTER_SLACK_USER_IDS")
                    .unwrap_or_default()
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect(),
            },
            codex: CodexConfig {
                bin: env_or("CODEX_BIN", "codex"),
                default_args: env::var("CODEX_DEFAULT_ARGS")
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect(),
                timeout_seconds: env::var("CODEX_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "1800".to_string())
                    .parse()
                    .context("CODEX_TIMEOUT_SECONDS must be a valid u64")?,
                browser_task_timeout_seconds: env::var("BROWSER_TASK_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "420".to_string())
                    .parse()
                    .context("BROWSER_TASK_TIMEOUT_SECONDS must be a valid u64")?,
                playwright_cli_wrapper: env_path("PLAYWRIGHT_CLI_WRAPPER")
                    .or_else(default_playwright_cli_wrapper)
                    .unwrap_or_else(|| {
                        current_dir.join("../.codex/skills/playwright/scripts/playwright_cli.sh")
                    }),
                playwright_cli_preflight_timeout_seconds: env::var(
                    "PLAYWRIGHT_CLI_PREFLIGHT_TIMEOUT_SECONDS",
                )
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .context("PLAYWRIGHT_CLI_PREFLIGHT_TIMEOUT_SECONDS must be a valid u64")?,
            },
            terminal: TerminalConfig {
                command: env_or("TERMINAL_COMMAND", "/bin/zsh"),
            },
            hooks: HookConfig {
                timeout_seconds: env::var("ENV_HOOK_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "900".to_string())
                    .parse()
                    .context("ENV_HOOK_TIMEOUT_SECONDS must be a valid u64")?,
            },
            paths: PathConfig {
                relay_home,
                sources_dir,
                workspaces_dir,
                policies_dir,
                workflows_dir,
            },
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn required_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("{key} must be set in the environment"))
}

fn env_path(key: &str) -> Option<PathBuf> {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn default_relay_home() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".relay"))
}

fn default_playwright_cli_wrapper() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex/skills/playwright/scripts/playwright_cli.sh"))
}

pub type SharedConfig = Arc<Config>;
