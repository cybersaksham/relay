"use client";

import Link from "next/link";
import { ReactNode, useState } from "react";

import { createEnvironment, deleteEnvironment, updateEnvironment } from "@/lib/api";
import { EnvironmentSummary } from "@/lib/types";

import {
  EnvironmentForm,
  environmentToFormValues,
  parseAliases,
} from "@/components/environment-form";

export function EnvironmentManager({
  initialEnvironments,
}: {
  initialEnvironments: EnvironmentSummary[];
}) {
  const [environments, setEnvironments] = useState(initialEnvironments);
  const [editing, setEditing] = useState<EnvironmentSummary | null>(null);
  const [deleting, setDeleting] = useState<EnvironmentSummary | null>(null);
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [deletePending, setDeletePending] = useState(false);

  async function handleCreate(values: {
    name: string;
    slug: string;
    git_ssh_url: string;
    default_branch: string;
    aliases: string[];
    enabled: boolean;
  }) {
    const response = await createEnvironment(values);
    setEnvironments((current) =>
      [response.environment, ...current].sort((left, right) => left.name.localeCompare(right.name)),
    );
  }

  async function handleUpdate(values: {
    name: string;
    slug: string;
    git_ssh_url: string;
    default_branch: string;
    aliases: string[];
    enabled: boolean;
  }) {
    if (!editing) {
      return;
    }

    const response = await updateEnvironment(editing.id, values);
    setEnvironments((current) =>
      current
        .map((environment) =>
          environment.id === editing.id ? response.environment : environment,
        )
        .sort((left, right) => left.name.localeCompare(right.name)),
    );
    setEditing(null);
  }

  async function handleDelete() {
    if (!deleting) {
      return;
    }

    setDeletePending(true);
    setDeleteError(null);
    try {
      await deleteEnvironment(deleting.id);
      setEnvironments((current) => current.filter((environment) => environment.id !== deleting.id));
      setDeleting(null);
    } catch (error) {
      setDeleteError(error instanceof Error ? error.message : "Failed to delete environment");
    } finally {
      setDeletePending(false);
    }
  }

  return (
    <>
      <div className="grid gap-8 lg:grid-cols-[1.1fr_0.9fr]">
        <section className="space-y-4">
          <div>
            <h1 className="text-3xl font-semibold">Environments</h1>
            <p className="text-sm text-slate-600">
              Repo-backed execution contexts mapped into persistent Relay sources and workspaces.
            </p>
          </div>
          <div className="space-y-4">
            {environments.length === 0 ? (
              <div className="rounded-3xl border border-dashed border-line bg-white/60 p-8 text-sm text-slate-600">
                No environments have been created yet.
              </div>
            ) : null}
            {environments.map((environment) => (
              <article
                key={environment.id}
                className="rounded-3xl border border-line bg-white/80 p-5 shadow-panel"
              >
                <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-3">
                      <Link
                        href={`/environments/${environment.id}`}
                        className="text-xl font-semibold transition hover:text-accent"
                      >
                        {environment.name}
                      </Link>
                      <span className="rounded-full bg-accentSoft px-3 py-1 text-xs font-medium text-accent">
                        {environment.default_branch}
                      </span>
                    </div>
                    <p className="mt-1 text-sm text-slate-600">{environment.slug}</p>
                    <p className="mt-3 font-mono text-xs text-slate-600">{environment.git_ssh_url}</p>
                    <div className="mt-4 flex flex-wrap gap-2">
                      {parseAliases(environment.aliases).map((alias) => (
                        <span
                          key={alias}
                          className="rounded-full border border-line bg-fog px-3 py-1 text-xs text-slate-600"
                        >
                          {alias}
                        </span>
                      ))}
                    </div>
                  </div>
                  <div className="flex shrink-0 gap-3">
                    <button
                      type="button"
                      onClick={() => setEditing(environment)}
                      className="rounded-full border border-line px-4 py-2 text-sm font-medium text-slate-700 transition hover:bg-fog"
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        setDeleteError(null);
                        setDeleting(environment);
                      }}
                      className="rounded-full border border-red-200 px-4 py-2 text-sm font-medium text-red-700 transition hover:bg-red-50"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </article>
            ))}
          </div>
        </section>
        <EnvironmentForm
          title="Create Environment"
          description="Register a repo-backed environment and prime its source clone."
          submitLabel="Create Environment"
          submittingLabel="Creating..."
          onSubmit={handleCreate}
        />
      </div>

      {editing ? (
        <ModalShell title={`Edit ${editing.name}`} onClose={() => setEditing(null)}>
          <EnvironmentForm
            title="Edit Environment"
            description="Update repo metadata and refresh the source clone for this environment."
            submitLabel="Save Changes"
            submittingLabel="Saving..."
            initialValues={environmentToFormValues(editing)}
            onSubmit={handleUpdate}
            onCancel={() => setEditing(null)}
          />
        </ModalShell>
      ) : null}

      {deleting ? (
        <ModalShell title="Delete Environment" onClose={() => setDeleting(null)}>
          <div className="rounded-3xl border border-line bg-white/80 p-6 shadow-panel">
            <p className="text-sm text-slate-600">
              Delete <span className="font-semibold text-ink">{deleting.name}</span> and remove its
              cached source clone. This cannot be undone.
            </p>
            <p className="mt-3 text-xs text-slate-500">
              Relay will block deletion if existing tasks already reference this environment.
            </p>
            {deleteError ? <p className="mt-4 text-sm text-red-700">{deleteError}</p> : null}
            <div className="mt-6 flex flex-wrap gap-3">
              <button
                type="button"
                onClick={handleDelete}
                disabled={deletePending}
                className="rounded-full bg-red-600 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
              >
                {deletePending ? "Deleting..." : "Delete Environment"}
              </button>
              <button
                type="button"
                onClick={() => setDeleting(null)}
                className="rounded-full border border-line px-5 py-2.5 text-sm font-medium text-slate-700 transition hover:bg-fog"
              >
                Cancel
              </button>
            </div>
          </div>
        </ModalShell>
      ) : null}
    </>
  );
}

function ModalShell({
  title,
  children,
  onClose,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 px-4 py-8">
      <div className="w-full max-w-2xl">
        <div className="mb-3 flex items-center justify-between text-white">
          <h2 className="text-lg font-semibold">{title}</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded-full border border-white/20 px-3 py-1.5 text-sm transition hover:bg-white/10"
          >
            Close
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
