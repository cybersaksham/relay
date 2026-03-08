import React from "react";
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { EnvironmentManager } from "@/components/environment-manager";

const createEnvironment = vi.fn();

vi.mock("@/lib/api", () => ({
  createEnvironment: (...args: unknown[]) => createEnvironment(...args),
}));

describe("EnvironmentManager", () => {
  beforeEach(() => {
    createEnvironment.mockReset();
  });

  it("renders create and open actions but keeps edit and delete off the list page", () => {
    render(
      <EnvironmentManager
        initialEnvironments={[
          {
            id: "env-1",
            name: "Newton Web",
            slug: "newton-web",
            git_ssh_url: "git@github.com:Newton-School/newton-web.git",
            default_branch: "master",
            aliases: JSON.stringify(["newton", "web"]),
            enabled: true,
            source_sync_status: "ready",
            source_sync_error: null,
            source_synced_at: new Date().toISOString(),
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]}
      />,
    );

    expect(screen.getByRole("button", { name: "Create" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Open" })).toBeInTheDocument();
    expect(screen.getByText("Source ready")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Edit" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Delete" })).not.toBeInTheDocument();
  });
});
