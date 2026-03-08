"use client";

import { FormEvent, useState } from "react";

import { EnvironmentSummary } from "@/lib/types";

export function ManualTaskForm({
  environments,
}: {
  environments: EnvironmentSummary[];
}) {
  const [message, setMessage] = useState<string | null>(null);

  function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setMessage("Manual task launches are not wired in the portal yet. Use Slack to trigger a real run.");
  }

  return (
    <div className="surface overflow-hidden">
      <div className="surface-header">
        <h2 className="text-lg font-semibold text-ink">Run Task</h2>
      </div>
      <form onSubmit={onSubmit} className="surface-body space-y-4">
        <label className="block space-y-2">
          <span className="text-sm font-medium text-slate-700">Environment</span>
          <select className="w-full rounded-lg border border-line bg-white px-3 py-2 text-sm">
            <option value="">General workspace</option>
            {environments.map((environment) => (
              <option key={environment.id} value={environment.id}>
                {environment.name}
              </option>
            ))}
          </select>
        </label>
        <label className="block space-y-2">
          <span className="text-sm font-medium text-slate-700">Prompt</span>
          <textarea
            className="min-h-28 w-full rounded-lg border border-line bg-white px-3 py-2 text-sm"
            placeholder="Describe your task. Relay will use a local workflow template if it matches; otherwise it runs your prompt directly."
          />
        </label>
        <label className="block space-y-2">
          <span className="text-sm font-medium text-slate-700">Title</span>
          <input
            className="w-full rounded-lg border border-line bg-white px-3 py-2 text-sm"
            placeholder="Optional task title"
          />
        </label>
        {message ? <p className="text-sm text-slate-600">{message}</p> : null}
        <button
          type="submit"
          className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-slate-800"
        >
          Run Task
        </button>
      </form>
    </div>
  );
}
