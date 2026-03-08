import Link from "next/link";

import { formatUtcTimestamp } from "@/lib/format";
import { TaskSummary } from "@/lib/types";

export function TaskList({ tasks }: { tasks: TaskSummary[] }) {
  if (tasks.length === 0) {
    return (
      <div className="surface p-8 text-sm text-slate-600">No tasks have been recorded yet.</div>
    );
  }

  return (
    <div className="surface overflow-hidden">
      <table className="data-table">
        <thead>
          <tr>
            <th>Task</th>
            <th>Status</th>
            <th>Workflow</th>
            <th>Source</th>
            <th>Created</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map((task) => (
            <tr key={task.id} className="border-b border-line last:border-b-0">
              <td className="font-medium text-ink">{task.id.slice(0, 8)}</td>
              <td className="capitalize">{task.status.replaceAll("_", " ")}</td>
              <td>{task.workflow_name ?? "Generic run"}</td>
              <td>{task.runner_kind}</td>
              <td>{formatUtcTimestamp(task.started_at)}</td>
              <td>
                <Link
                  href={`/tasks/${task.id}`}
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
  );
}
