"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";

import { deleteEnvironment, refreshEnvironment, updateEnvironment } from "@/lib/api";
import { parseAliases } from "@/lib/environments";
import { formatUtcTimestamp } from "@/lib/format";
import { EnvironmentDetail, SessionSummary } from "@/lib/types";

import {
  EnvironmentForm,
  environmentToFormValues,
} from "@/components/environment-form";
import { TaskList } from "@/components/task-list";

function sourceStatusCopy(status: string) {
  switch (status) {
    case "ready":
      return "Ready";
    case "syncing":
      return "Syncing";
    case "failed":
      return "Failed";
    default:
      return "Pending";
  }
}

export function EnvironmentDetailManager({
  initialDetail,
  tasks,
}: {
  initialDetail: EnvironmentDetail;
  tasks: SessionSummary[];
}) {
  const router = useRouter();
  const [detail, setDetail] = useState(initialDetail);
  const [deletePending, setDeletePending] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [refreshPending, setRefreshPending] = useState(false);
  const [refreshError, setRefreshError] = useState<string | null>(null);
  const [refreshSuccess, setRefreshSuccess] = useState<string | null>(null);

  async function handleUpdate(values: {
    name: string;
    slug: string;
    git_ssh_url: string;
    default_branch: string;
    aliases: string[];
    enabled: boolean;
    source_setup_script: string;
    workspace_setup_script: string;
  }) {
    const response = await updateEnvironment(detail.environment.id, {
      ...values,
      source_setup_script: values.source_setup_script || null,
      workspace_setup_script: values.workspace_setup_script || null,
    });
    setDetail(response);
  }

  async function handleRefreshCache() {
    setRefreshPending(true);
    setRefreshError(null);
    setRefreshSuccess(null);
    try {
      const response = await refreshEnvironment(detail.environment.id);
      setDetail(response);
      setRefreshSuccess("Cache refresh started.");
      router.refresh();
    } catch (error) {
      setRefreshError(error instanceof Error ? error.message : "Failed to refresh cache.");
    } finally {
      setRefreshPending(false);
    }
  }

  async function handleDelete() {
    setDeletePending(true);
    setDeleteError(null);
    try {
      await deleteEnvironment(detail.environment.id);
      router.push("/environments");
      router.refresh();
    } catch (error) {
      setDeleteError(error instanceof Error ? error.message : "Failed to delete environment");
    } finally {
      setDeletePending(false);
    }
  }

  return (
    <>
      <div className="page-shell">
        <div>
          <h1 className="text-3xl font-semibold text-ink">
            Environment: {detail.environment.name}
          </h1>
          <p className="mt-2 text-sm text-slate-600">
            <Link
              href="/environments"
              className="underline decoration-slate-300 underline-offset-4"
            >
              Back to environments
            </Link>
          </p>
        </div>

        <section className="surface overflow-hidden">
          <div className="surface-header">
            <h2 className="text-lg font-semibold text-ink">Edit Environment</h2>
          </div>
          <div className="surface-body space-y-4">
            <EnvironmentForm
              title="Edit Environment"
              description="Update repo metadata and refresh the source clone for this environment."
              submitLabel="Save Changes"
              submittingLabel="Saving..."
              hideHeader
              initialValues={environmentToFormValues(detail.environment)}
              onSubmit={handleUpdate}
            />
            <p className="text-sm text-slate-500">
              Created: {formatUtcTimestamp(detail.environment.created_at)} | Updated:{" "}
              {formatUtcTimestamp(detail.environment.updated_at)}
            </p>
            <p className="text-sm text-slate-500">
              Source clone:{" "}
              <code className="rounded bg-slate-100 px-1.5 py-0.5 text-xs text-slate-700">
                {detail.source_path}
              </code>
            </p>
            <p className="text-sm text-slate-500">
              Source sync status:{" "}
              <span className="font-medium text-ink">
                {sourceStatusCopy(detail.environment.source_sync_status)}
              </span>
              {detail.environment.source_synced_at ? (
                <>
                  {" "}
                  on {formatUtcTimestamp(detail.environment.source_synced_at)}
                </>
              ) : null}
            </p>
            {detail.environment.source_sync_error ? (
              <p className="text-sm text-red-700">
                Last source sync error: {detail.environment.source_sync_error}
              </p>
            ) : null}
            <div className="flex flex-wrap items-center gap-3">
              <button
                type="button"
                onClick={handleRefreshCache}
                disabled={refreshPending || detail.environment.source_sync_status === "syncing"}
                className="rounded-md border border-line px-4 py-2 text-sm font-medium text-slate-700 transition hover:bg-fog disabled:cursor-not-allowed disabled:opacity-50"
              >
                {refreshPending ? "Refreshing..." : "Refresh Cache"}
              </button>
              {refreshSuccess ? <p className="text-sm text-emerald-700">{refreshSuccess}</p> : null}
              {refreshError ? <p className="text-sm text-red-700">{refreshError}</p> : null}
            </div>
            <p className="text-sm text-slate-500">
              Aliases: {parseAliases(detail.environment.aliases).join(", ") || "-"}
            </p>
          </div>
        </section>

        <section className="surface overflow-hidden border-red-200">
          <div className="surface-header">
            <h2 className="text-lg font-semibold text-ink">Danger Zone</h2>
          </div>
          <div className="surface-body space-y-4">
            <p className="text-sm text-slate-600">
              Permanent delete removes this environment and its cached source clone. Existing tasks
              already linked to this environment will block deletion.
            </p>
            <button
              type="button"
              onClick={() => {
                setDeleteError(null);
                setConfirmDelete(true);
              }}
              className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700"
            >
              Delete Permanently
            </button>
            {deleteError ? <p className="text-sm text-red-700">{deleteError}</p> : null}
          </div>
        </section>

        <section className="space-y-4">
          <div>
            <h2 className="text-lg font-semibold text-ink">Threads In This Environment</h2>
          </div>
          <TaskList tasks={tasks} />
        </section>
      </div>

      {confirmDelete ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 px-4 py-8">
          <div className="w-full max-w-xl">
            <div className="mb-3 flex items-center justify-between text-white">
              <h2 className="text-lg font-semibold">Delete Environment</h2>
              <button
                type="button"
                onClick={() => setConfirmDelete(false)}
                className="rounded-md border border-white/20 px-3 py-1.5 text-sm transition hover:bg-white/10"
              >
                Close
              </button>
            </div>
            <div className="surface p-6">
              <p className="text-sm text-slate-600">
                Delete <span className="font-semibold text-ink">{detail.environment.name}</span> and
                remove its cached source clone. This cannot be undone.
              </p>
              {deleteError ? <p className="mt-4 text-sm text-red-700">{deleteError}</p> : null}
              <div className="mt-6 flex flex-wrap gap-3">
                <button
                  type="button"
                  onClick={handleDelete}
                  disabled={deletePending}
                  className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  {deletePending ? "Deleting..." : "Delete Permanently"}
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmDelete(false)}
                  className="rounded-md border border-line px-4 py-2 text-sm font-medium text-slate-700 transition hover:bg-fog"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}
