import Link from "next/link";

import { TaskSummary } from "@/lib/types";

export function TaskList({ tasks }: { tasks: TaskSummary[] }) {
  if (tasks.length === 0) {
    return (
      <div className="rounded-3xl border border-dashed border-line bg-white/60 p-8 text-sm text-slate-600">
        No tasks have been recorded yet.
      </div>
    );
  }

  return (
    <div className="overflow-hidden rounded-3xl border border-line bg-white/80 shadow-panel">
      <div className="grid grid-cols-[1.3fr_0.8fr_1fr_1fr] gap-4 border-b border-line px-5 py-3 text-xs font-semibold uppercase tracking-[0.18em] text-slate-500">
        <span>Task</span>
        <span>Status</span>
        <span>Workflow</span>
        <span>Started</span>
      </div>
      <div className="divide-y divide-line">
        {tasks.map((task) => (
          <Link
            key={task.id}
            href={`/tasks/${task.id}`}
            className="grid grid-cols-[1.3fr_0.8fr_1fr_1fr] gap-4 px-5 py-4 text-sm transition hover:bg-accentSoft/50"
          >
            <span className="font-medium text-ink">{task.id.slice(0, 8)}</span>
            <span className="capitalize text-slate-700">{task.status.replaceAll("_", " ")}</span>
            <span className="text-slate-600">{task.workflow_name ?? "Generic run"}</span>
            <span className="text-slate-600">
              {new Date(task.started_at).toLocaleString()}
            </span>
          </Link>
        ))}
      </div>
    </div>
  );
}
