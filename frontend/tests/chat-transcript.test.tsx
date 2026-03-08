import React from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { ChatTranscript } from "@/components/chat-transcript";

describe("ChatTranscript", () => {
  it("renders resolved message text", () => {
    render(
      <ChatTranscript
        messages={[
          {
            id: "1",
            session_id: "session",
            task_run_id: "task",
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
  });
});
