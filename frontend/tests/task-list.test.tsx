import React from "react";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { TaskList } from "@/components/task-list";

describe("TaskList", () => {
  it("shows fallback copy when empty", () => {
    render(<TaskList tasks={[]} />);
    expect(screen.getByText(/No task threads have been recorded yet/i)).toBeInTheDocument();
  });
});
