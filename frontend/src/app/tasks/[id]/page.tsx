import { ChatTranscript } from "@/components/chat-transcript";
import { TerminalStream } from "@/components/terminal-stream";
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

      <section className="surface overflow-hidden">
        <div className="surface-header">
          <h2 className="text-lg font-semibold text-ink">Run History</h2>
        </div>
        <div className="overflow-x-auto">
          <table className="data-table">
            <thead>
              <tr>
                <th>Run</th>
                <th>Status</th>
                <th>Workflow</th>
                <th>Started</th>
              </tr>
            </thead>
            <tbody>
              {task.runs.map((run) => (
                <tr key={run.id} className="border-b border-line last:border-b-0">
                  <td className="font-mono text-xs">{run.id.slice(0, 8)}</td>
                  <td className="capitalize">{run.status.replaceAll("_", " ")}</td>
                  <td>{run.workflow_name ?? "Generic run"}</td>
                  <td>{formatUtcTimestamp(run.started_at)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <div className="grid gap-6 xl:grid-cols-[0.92fr_1.08fr]">
        <ChatTranscript messages={messages} />
        {task.latest_run ? (
          <TerminalStream taskId={task.latest_run.id} />
        ) : (
          <section className="surface overflow-hidden">
            <div className="surface-header">
              <h2 className="text-lg font-semibold text-ink">Live Terminal</h2>
            </div>
            <div className="surface-body text-sm text-slate-600">
              No terminal output is available for this thread yet.
            </div>
          </section>
        )}
      </div>
    </div>
  );
}
