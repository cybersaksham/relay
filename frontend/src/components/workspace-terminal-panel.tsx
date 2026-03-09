"use client";

import { useEffect, useMemo, useRef, useState } from "react";

import { getWebSocketUrl } from "@/lib/api";

import { WorkspaceGitDiffSheet } from "@/components/workspace-git-diff-sheet";

type ConnectionState = "connecting" | "connected" | "disconnected" | "error";

type TerminalServerMessage =
  | {
      kind: "snapshot";
      cwd: string;
      shell: string;
      data: string;
      active: boolean;
    }
  | {
      kind: "output";
      data: string;
    }
  | {
      kind: "status";
      status: string;
      message: string | null;
      exit_code: number | null;
    }
  | {
      kind: "error";
      message: string;
    };

export function WorkspaceTerminalPanel({
  sessionId,
  workspacePath,
  threadTs,
  status,
  workflowName,
}: {
  sessionId: string;
  workspacePath: string;
  threadTs: string;
  status: string;
  workflowName: string;
}) {
  const [activePanel, setActivePanel] = useState<"terminal" | "git" | null>(null);

  useEffect(() => {
    if (activePanel === null) {
      return;
    }

    const previousOverflow = document.body.style.overflow;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setActivePanel(null);
      }
    };

    document.body.style.overflow = "hidden";
    window.addEventListener("keydown", handleKeyDown);

    return () => {
      document.body.style.overflow = previousOverflow;
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [activePanel]);

  return (
    <>
      <section className="surface sticky top-4 z-20 overflow-hidden">
        <div className="flex flex-wrap items-center justify-between gap-4 px-5 py-4">
          <div className="min-w-0 flex-1">
            <div className="text-xs uppercase tracking-[0.18em] text-slate-500">Task</div>
            <div className="mt-1 flex flex-wrap items-center gap-2">
              <h1 className="text-xl font-semibold text-ink">Thread {threadTs}</h1>
              <span className="rounded-full border border-line px-2.5 py-1 text-xs font-medium uppercase tracking-[0.12em] text-slate-600">
                {status}
              </span>
              <span className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600">
                {workflowName}
              </span>
            </div>
            <p className="mt-1 truncate font-mono text-xs text-slate-500">{workspacePath}</p>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setActivePanel("git")}
              aria-label="Open workspace git diff"
              className="inline-flex h-11 w-11 items-center justify-center rounded-full border border-line bg-white text-slate-700 transition hover:bg-fog"
            >
              <GitDiffIcon />
            </button>
            <button
              type="button"
              onClick={() => setActivePanel("terminal")}
              aria-label="Open workspace terminal"
              className="inline-flex h-11 w-11 items-center justify-center rounded-full border border-line bg-white text-slate-700 transition hover:bg-fog"
            >
              <TerminalIcon />
            </button>
          </div>
        </div>
      </section>

      <WorkspaceGitDiffSheet
        sessionId={sessionId}
        workspacePath={workspacePath}
        isOpen={activePanel === "git"}
        onClose={() => setActivePanel(null)}
      />

      {activePanel === "terminal" ? (
        <div
          className="fixed inset-0 z-50 bg-slate-950/45"
          onClick={() => setActivePanel(null)}
          role="presentation"
        >
          <div className="flex h-full items-end justify-center px-4 pb-0 pt-8">
            <section
              className="max-h-[85vh] w-full max-w-6xl overflow-hidden rounded-t-[28px] border border-line bg-white shadow-2xl"
              onClick={(event) => event.stopPropagation()}
              aria-label="Workspace terminal"
            >
              <div className="flex justify-center pt-3">
                <div className="h-1.5 w-14 rounded-full bg-slate-200" />
              </div>
              <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-5 py-4">
                <div>
                  <h2 className="text-lg font-semibold text-ink">Workspace Terminal</h2>
                  <p className="mt-1 text-sm text-slate-500">{workspacePath}</p>
                </div>
                <button
                  type="button"
                  onClick={() => setActivePanel(null)}
                  className="rounded-full border border-line px-3 py-1.5 text-sm font-medium text-slate-700 transition hover:bg-fog"
                >
                  Close
                </button>
              </div>

              <div className="overflow-y-auto">
                <WorkspaceTerminal
                  sessionId={sessionId}
                  workspacePath={workspacePath}
                  inSheet
                />
              </div>
            </section>
          </div>
        </div>
      ) : null}
    </>
  );
}

function WorkspaceTerminal({
  sessionId,
  workspacePath,
  inSheet = false,
}: {
  sessionId: string;
  workspacePath: string;
  inSheet?: boolean;
}) {
  const terminalRef = useRef<HTMLDivElement | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>("connecting");
  const [shell, setShell] = useState<string>("/bin/zsh");
  const [currentPath, setCurrentPath] = useState(workspacePath);
  const [statusMessage, setStatusMessage] = useState<string | null>("Connecting to workspace...");

  const statusClassName = useMemo(() => {
    switch (connectionState) {
      case "connected":
        return "border-emerald-200 bg-emerald-50 text-emerald-700";
      case "connecting":
        return "border-amber-200 bg-amber-50 text-amber-700";
      case "error":
        return "border-red-200 bg-red-50 text-red-700";
      default:
        return "border-slate-200 bg-slate-100 text-slate-600";
    }
  }, [connectionState]);

  useEffect(() => {
    let disposed = false;
    let socket: WebSocket | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let terminal: import("@xterm/xterm").Terminal | null = null;
    let fitAddon: import("@xterm/addon-fit").FitAddon | null = null;
    const cleanup: Array<{ dispose: () => void }> = [];

    async function connect() {
      if (!terminalRef.current) {
        return;
      }

      setConnectionState("connecting");
      setStatusMessage("Connecting to workspace...");

      const [{ Terminal }, { FitAddon }] = await Promise.all([
        import("@xterm/xterm"),
        import("@xterm/addon-fit"),
      ]);

      if (disposed || !terminalRef.current) {
        return;
      }

      terminal = new Terminal({
        cursorBlink: true,
        fontFamily:
          '"SFMono-Regular", ui-monospace, "Cascadia Code", "Source Code Pro", Menlo, monospace',
        fontSize: 13,
        lineHeight: 1.25,
        scrollback: 5000,
        theme: {
          background: "#020617",
          foreground: "#cbd5e1",
          cursor: "#f8fafc",
          selectionBackground: "#334155",
        },
      });
      fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(terminalRef.current);
      fitAddon.fit();
      terminal.focus();

      cleanup.push(
        terminal.onData((data) => {
          if (socket?.readyState === WebSocket.OPEN) {
            socket.send(JSON.stringify({ kind: "input", data }));
          }
        }),
      );
      cleanup.push(
        terminal.onResize(({ cols, rows }) => {
          if (socket?.readyState === WebSocket.OPEN) {
            socket.send(JSON.stringify({ kind: "resize", cols, rows }));
          }
        }),
      );

      resizeObserver = new ResizeObserver(() => {
        fitAddon?.fit();
      });
      resizeObserver.observe(terminalRef.current);

      socket = new WebSocket(getWebSocketUrl(`/api/tasks/${sessionId}/workspace-terminal/ws`));
      socket.onopen = () => {
        if (!disposed) {
          setConnectionState("connected");
          terminal?.focus();
        }
      };
      socket.onerror = () => {
        if (!disposed) {
          setConnectionState("error");
          setStatusMessage("Terminal connection failed.");
        }
      };
      socket.onclose = () => {
        if (!disposed) {
          setConnectionState("disconnected");
          setStatusMessage((current) => current ?? "Terminal disconnected.");
        }
      };
      socket.onmessage = (event) => {
        if (!terminal) {
          return;
        }

        let message: TerminalServerMessage;
        try {
          message = JSON.parse(event.data) as TerminalServerMessage;
        } catch {
          return;
        }

        switch (message.kind) {
          case "snapshot":
            terminal.reset();
            if (message.data) {
              terminal.write(message.data);
            }
            setShell(message.shell);
            setCurrentPath(message.cwd);
            setConnectionState(message.active ? "connected" : "disconnected");
            setStatusMessage(
              message.active
                ? "Connected to the workspace shell."
                : "Previous terminal session ended. Reopen to start a fresh shell.",
            );
            break;
          case "output":
            terminal.write(message.data);
            break;
          case "status":
            if (message.message) {
              setStatusMessage(message.message);
            }
            if (message.status === "exited") {
              setConnectionState("disconnected");
            } else if (message.status === "connected") {
              setConnectionState("connected");
            }
            break;
          case "error":
            setConnectionState("error");
            setStatusMessage(message.message);
            break;
        }
      };
    }

    void connect();

    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      socket?.close();
      cleanup.forEach((item) => item.dispose());
      terminal?.dispose();
    };
  }, [sessionId, workspacePath]);

  return (
    <section className={inSheet ? "space-y-4 p-5" : "surface overflow-hidden"}>
      <div
        className={
          inSheet
            ? "flex flex-wrap items-center justify-between gap-3"
            : "surface-header flex flex-wrap items-center justify-between gap-3"
        }
      >
        <div>
          <h2 className="text-lg font-semibold text-ink">
            {inSheet ? "Live Workspace Shell" : "Workspace Terminal"}
          </h2>
          <p className="mt-1 text-sm text-slate-500">{statusMessage}</p>
        </div>
        <span
          className={`rounded-full border px-3 py-1 text-xs font-medium uppercase tracking-[0.12em] ${statusClassName}`}
        >
          {connectionState}
        </span>
      </div>
      <div className={inSheet ? "space-y-4" : "surface-body space-y-4"}>
        <div className="grid gap-3 rounded-xl border border-line bg-slate-50 p-4 text-sm text-slate-600 lg:grid-cols-2">
          <div>
            <div className="text-xs uppercase tracking-[0.18em] text-slate-500">Shell</div>
            <div className="mt-1 font-mono text-xs text-ink">{shell}</div>
          </div>
          <div>
            <div className="text-xs uppercase tracking-[0.18em] text-slate-500">Workspace</div>
            <div className="mt-1 font-mono text-xs text-ink">{currentPath}</div>
          </div>
        </div>

        <div
          ref={terminalRef}
          className={`overflow-hidden rounded-xl border border-slate-800 bg-slate-950 p-2 ${
            inSheet ? "h-[min(52vh,460px)]" : "h-[460px]"
          }`}
        />
      </div>
    </section>
  );
}

function TerminalIcon() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="h-5 w-5"
    >
      <path d="m4 17 6-5-6-5" />
      <path d="M12 19h8" />
      <rect x="3" y="4" width="18" height="16" rx="2.5" />
    </svg>
  );
}

function GitDiffIcon() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="h-5 w-5"
    >
      <path d="M7 4v16" />
      <path d="M17 4v16" />
      <path d="m10 8 4 4-4 4" />
      <path d="M4 7h6" />
      <path d="M14 17h6" />
    </svg>
  );
}
