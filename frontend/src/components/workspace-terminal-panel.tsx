"use client";

import { useEffect, useMemo, useRef, useState } from "react";

import { getWebSocketUrl } from "@/lib/api";

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
}: {
  sessionId: string;
  workspacePath: string;
}) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <button
          type="button"
          onClick={() => setIsOpen((current) => !current)}
          className="rounded-xl bg-accent px-4 py-2 text-sm font-medium text-white transition hover:bg-accent/90"
        >
          {isOpen ? "Hide Workspace Terminal" : "Open Workspace Terminal"}
        </button>
      </div>

      {isOpen ? <WorkspaceTerminal sessionId={sessionId} workspacePath={workspacePath} /> : null}
    </div>
  );
}

function WorkspaceTerminal({
  sessionId,
  workspacePath,
}: {
  sessionId: string;
  workspacePath: string;
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
    <section className="surface overflow-hidden">
      <div className="surface-header flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold text-ink">Workspace Terminal</h2>
          <p className="mt-1 text-sm text-slate-500">{statusMessage}</p>
        </div>
        <span
          className={`rounded-full border px-3 py-1 text-xs font-medium uppercase tracking-[0.12em] ${statusClassName}`}
        >
          {connectionState}
        </span>
      </div>
      <div className="surface-body space-y-4">
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
          className="h-[460px] overflow-hidden rounded-xl border border-slate-800 bg-slate-950 p-2"
        />
      </div>
    </section>
  );
}
