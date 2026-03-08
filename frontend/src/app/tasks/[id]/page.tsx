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
        <h1 className="text-3xl font-semibold text-ink">Task {task.run.id.slice(0, 8)}</h1>
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
              <td className="capitalize">{task.run.status.replaceAll("_", " ")}</td>
            </tr>
            <tr className="border-b border-line">
              <td>Workflow</td>
              <td>{task.run.workflow_name ?? "Generic run"}</td>
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
              <td>Started</td>
              <td>{formatUtcTimestamp(task.run.started_at)}</td>
            </tr>
          </tbody>
        </table>
      </section>

      <div className="grid gap-6 xl:grid-cols-[0.92fr_1.08fr]">
        <ChatTranscript messages={messages} />
        <TerminalStream taskId={task.run.id} />
      </div>
    </div>
  );
}
