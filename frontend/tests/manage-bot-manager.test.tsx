import React from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ManageBotManager } from "@/components/manage-bot-manager";

const lookupSlackMessage = vi.fn();
const updateSlackMessage = vi.fn();
const deleteSlackMessage = vi.fn();

vi.mock("@/lib/api", () => ({
  lookupSlackMessage: (...args: unknown[]) => lookupSlackMessage(...args),
  updateSlackMessage: (...args: unknown[]) => updateSlackMessage(...args),
  deleteSlackMessage: (...args: unknown[]) => deleteSlackMessage(...args),
}));

describe("ManageBotManager", () => {
  beforeEach(() => {
    lookupSlackMessage.mockReset();
    updateSlackMessage.mockReset();
    deleteSlackMessage.mockReset();
  });

  it("fetches a bot message and exposes edit and delete actions", async () => {
    lookupSlackMessage.mockResolvedValue({
      channel_id: "C123",
      ts: "1736451111.222233",
      thread_ts: null,
      text: "hello",
      raw_text: "hello",
      author_user_id: "U_BOT",
      author_bot_id: "B_BOT",
    });

    render(<ManageBotManager />);

    fireEvent.change(screen.getByLabelText("Slack message link"), {
      target: { value: "https://workspace.slack.com/archives/C123/p1736451111222233" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Fetch Message" }));

    await waitFor(() =>
      expect(lookupSlackMessage).toHaveBeenCalledWith(
        "https://workspace.slack.com/archives/C123/p1736451111222233",
      ),
    );

    expect(screen.getByRole("button", { name: "Save Changes" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Delete Message" })).toBeInTheDocument();
    expect(screen.getByDisplayValue("hello")).toBeInTheDocument();
  });
});
