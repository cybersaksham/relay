"use client";

import Link from "next/link";
import { useState } from "react";

import { createEnvironment } from "@/lib/api";
import { parseAliases } from "@/lib/environments";
import { formatUtcTimestamp } from "@/lib/format";
import { EnvironmentSummary } from "@/lib/types";

import { EnvironmentForm } from "@/components/environment-form";

function sourceStatusLabel(environment: EnvironmentSummary) {
  switch (environment.source_sync_status) {
    case "ready":
      return "Source ready";
    case "syncing":
      return "Source syncing";
    case "failed":
      return "Source failed";
    default:
      return "Source pending";
  }
}

export function EnvironmentManager({
  initialEnvironments,
}: {
  initialEnvironments: EnvironmentSummary[];
}) {
  const [environments, setEnvironments] = useState(initialEnvironments);

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

  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Environments</h1>
      </div>

      <section className="surface overflow-hidden">
        <div className="surface-header">
          <h2 className="text-lg font-semibold text-ink">Create Environment</h2>
        </div>
        <div className="surface-body">
          <EnvironmentForm
            title="Create Environment"
            description="Register a repo-backed environment and prime its source clone."
            submitLabel="Create"
            submittingLabel="Creating..."
            hideHeader
            onSubmit={handleCreate}
          />
        </div>
      </section>

      <section className="surface overflow-hidden">
        <div className="surface-header">
          <h2 className="text-lg font-semibold text-ink">Registered Environments</h2>
        </div>
        {environments.length === 0 ? (
          <div className="surface-body text-sm text-slate-600">
            No environments have been created yet.
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Repo</th>
                  <th>Base</th>
                  <th>Updated</th>
                  <th>Open</th>
                </tr>
              </thead>
              <tbody>
                {environments.map((environment) => (
                  <tr key={environment.id} className="border-b border-line last:border-b-0">
                    <td>
                      <div className="font-medium text-ink">{environment.name}</div>
                      <div className="mt-1 text-xs text-slate-500">{environment.slug}</div>
                      <div className="mt-2">
                        <span className="rounded-full border border-line px-2 py-1 text-[11px] font-medium uppercase tracking-[0.12em] text-slate-600">
                          {sourceStatusLabel(environment)}
                        </span>
                      </div>
                      {parseAliases(environment.aliases).length > 0 ? (
                        <div className="mt-2 text-xs text-slate-500">
                          {parseAliases(environment.aliases).join(", ")}
                        </div>
                      ) : null}
                    </td>
                    <td className="font-mono text-xs">{environment.git_ssh_url}</td>
                    <td>{environment.default_branch}</td>
                    <td>{formatUtcTimestamp(environment.updated_at)}</td>
                    <td>
                      <Link
                        href={`/environments/${environment.id}`}
                        className="text-sm font-medium text-slate-900 underline decoration-slate-300 underline-offset-4"
                      >
                        Open
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}
