import Link from "next/link";

import { formatUtcTimestamp } from "@/lib/format";
import { SessionSummary } from "@/lib/types";

export function TaskList({ tasks }: { tasks: SessionSummary[] }) {
  if (tasks.length === 0) {
    return (
      <div className="surface p-8 text-sm text-slate-600">No task threads have been recorded yet.</div>
    );
  }

  return (
    <div className="surface overflow-hidden">
      <table className="data-table">
        <thead>
          <tr>
            <th>Thread</th>
            <th>Status</th>
            <th>Workflow</th>
            <th>Runs</th>
            <th>Updated</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map((task) => (
            <tr key={task.session.id} className="border-b border-line last:border-b-0">
              <td>
                <div className="font-medium text-ink">{task.session.thread_ts}</div>
                <div className="mt-1 text-xs text-slate-500">{task.session.workspace_id}</div>
              </td>
              <td className="capitalize">
                {(task.latest_run?.status ?? task.session.status).replaceAll("_", " ")}
              </td>
              <td>{task.latest_run?.workflow_name ?? "Generic run"}</td>
              <td>{task.run_count}</td>
              <td>
                {formatUtcTimestamp(task.latest_run?.updated_at ?? task.session.updated_at)}
              </td>
              <td>
                <Link
                  href={`/tasks/${task.session.id}`}
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
