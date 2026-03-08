import React from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { EnvironmentDetailManager } from "@/components/environment-detail-manager";

const updateEnvironment = vi.fn();
const deleteEnvironment = vi.fn();
const push = vi.fn();
const refresh = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push,
    refresh,
  }),
}));

vi.mock("@/lib/api", () => ({
  updateEnvironment: (...args: unknown[]) => updateEnvironment(...args),
  deleteEnvironment: (...args: unknown[]) => deleteEnvironment(...args),
}));

describe("EnvironmentDetailManager", () => {
  beforeEach(() => {
    updateEnvironment.mockReset();
    deleteEnvironment.mockReset();
    push.mockReset();
    refresh.mockReset();
  });

  it("renders edit and delete flows on the environment detail page", async () => {
    deleteEnvironment.mockResolvedValue({ deleted_id: "env-1" });

    render(
      <EnvironmentDetailManager
        initialDetail={{
          environment: {
            id: "env-1",
            name: "Newton Web",
            slug: "newton-web",
            git_ssh_url: "git@github.com:Newton-School/newton-web.git",
            default_branch: "master",
            aliases: JSON.stringify(["newton", "web"]),
            enabled: true,
            source_sync_status: "syncing",
            source_sync_error: null,
            source_synced_at: null,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
          source_path: "/Users/test/.relay/sources/newton-web",
        }}
        tasks={[]}
      />,
    );

    expect(screen.getByRole("heading", { name: /Edit Environment/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save Changes" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Delete Permanently" })).toBeInTheDocument();
    expect(screen.getByText(/Source sync status:/i)).toBeInTheDocument();
    expect(screen.getByText("Syncing")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "Delete Permanently" })[0]);
    expect(screen.getByRole("heading", { name: "Delete Environment" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("heading", { name: "Delete Environment" })).not.toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "Delete Permanently" })[0]);
    fireEvent.click(screen.getAllByRole("button", { name: "Delete Permanently" })[1]);

    await waitFor(() => expect(deleteEnvironment).toHaveBeenCalledWith("env-1"));
    await waitFor(() => expect(push).toHaveBeenCalledWith("/environments"));
    expect(refresh).toHaveBeenCalled();
  });
});
