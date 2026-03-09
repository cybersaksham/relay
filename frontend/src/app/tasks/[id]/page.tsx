import { ThreadConversation } from "@/components/thread-conversation";
import { getTask, getTaskMessages } from "@/lib/api";
import { formatUtcTimestamp } from "@/lib/format";

export default async function TaskPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const [task, messages] = await Promise.all([getTask(id), getTaskMessages(id)]);

  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">
          Thread {task.session.thread_ts}
        </h1>
      </div>

      <section className="surface overflow-hidden">
        <table className="data-table">
          <thead>
            <tr>
              <th>Field</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            <tr className="border-b border-line">
              <td>Status</td>
              <td className="capitalize">
                {(task.latest_run?.status ?? task.session.status).replaceAll("_", " ")}
              </td>
            </tr>
            <tr className="border-b border-line">
              <td>Latest Workflow</td>
              <td>{task.latest_run?.workflow_name ?? "Generic run"}</td>
            </tr>
            <tr className="border-b border-line">
              <td>Workspace ID</td>
              <td className="font-mono text-xs">{task.session.workspace_id}</td>
            </tr>
            <tr className="border-b border-line">
              <td>Workspace Path</td>
              <td className="font-mono text-xs">{task.session.workspace_path}</td>
            </tr>
            <tr className="border-b border-line">
              <td>Thread TS</td>
              <td className="font-mono text-xs">{task.session.thread_ts}</td>
            </tr>
            <tr>
              <td>Latest Run</td>
              <td>
                {task.latest_run
                  ? formatUtcTimestamp(task.latest_run.started_at)
                  : "No runs yet"}
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <ThreadConversation
        sessionId={task.session.id}
        runs={task.runs}
        messages={messages}
      />
    </div>
  );
}
