import React from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ThreadConversation } from "@/components/thread-conversation";

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    refresh: vi.fn(),
  }),
}));

vi.mock("@/lib/api", () => ({
  cancelTask: vi.fn(),
}));

vi.mock("@/components/terminal-stream", () => ({
  TerminalStream: ({ title }: { title: string }) => <div>{title}</div>,
}));

describe("ThreadConversation", () => {
  it("renders resolved message text with per-run processing section", () => {
    render(
      <ThreadConversation
        sessionId="session"
        runs={[
          {
            id: "run-1",
            session_id: "session",
            trigger_message_ts: "123.456",
            status: "succeeded",
            workflow_id: null,
            workflow_name: "Generic run",
            runner_kind: "codex_cli",
            started_at: new Date().toISOString(),
            finished_at: new Date().toISOString(),
            exit_code: 0,
            error_summary: null,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]}
        messages={[
          {
            id: "1",
            session_id: "session",
            task_run_id: "run-1",
            direction: "inbound",
            slack_user_id: "U123",
            raw_payload: "{}",
            resolved_payload: JSON.stringify({ text: "hello relay" }),
            created_at: new Date().toISOString(),
          },
        ]}
      />,
    );

    expect(screen.getByText("hello relay")).toBeInTheDocument();
    expect(screen.getByText("Processing")).toBeInTheDocument();
  });
});
