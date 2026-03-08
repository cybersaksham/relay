import React from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { EnvironmentManager } from "@/components/environment-manager";

const createEnvironment = vi.fn();
const updateEnvironment = vi.fn();
const deleteEnvironment = vi.fn();

vi.mock("@/lib/api", () => ({
  createEnvironment: (...args: unknown[]) => createEnvironment(...args),
  updateEnvironment: (...args: unknown[]) => updateEnvironment(...args),
  deleteEnvironment: (...args: unknown[]) => deleteEnvironment(...args),
}));

describe("EnvironmentManager", () => {
  beforeEach(() => {
    createEnvironment.mockReset();
    updateEnvironment.mockReset();
    deleteEnvironment.mockReset();
  });

  it("opens edit and delete flows for an existing environment", async () => {
    deleteEnvironment.mockResolvedValue({ deleted_id: "env-1" });

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
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit" }));
    expect(screen.getByRole("heading", { name: "Edit Environment" })).toBeInTheDocument();
    expect(screen.getByDisplayValue("Newton Web")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(screen.getByText(/This cannot be undone/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Delete Environment" }));

    await waitFor(() =>
      expect(deleteEnvironment).toHaveBeenCalledWith("env-1"),
    );
    await waitFor(() =>
      expect(screen.queryByText("Newton Web")).not.toBeInTheDocument(),
    );
  });
});
