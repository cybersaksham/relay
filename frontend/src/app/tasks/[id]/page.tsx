import { ChatTranscript } from "@/components/chat-transcript";
import { TerminalStream } from "@/components/terminal-stream";
import { getTask, getTaskMessages } from "@/lib/api";

export default async function TaskPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const [task, messages] = await Promise.all([getTask(id), getTaskMessages(id)]);

  return (
    <div className="grid gap-8 xl:grid-cols-[0.95fr_1.05fr]">
      <section className="space-y-6">
        <div className="rounded-[2rem] border border-line bg-white/80 p-8 shadow-panel">
          <p className="mb-2 text-xs uppercase tracking-[0.3em] text-accent">Task Run</p>
          <h1 className="text-3xl font-semibold">{task.run.id}</h1>
          <div className="mt-6 grid gap-4 md:grid-cols-2">
            <Meta label="Status" value={task.run.status} />
            <Meta label="Workflow" value={task.run.workflow_name ?? "Generic run"} />
            <Meta label="Workspace ID" value={task.session.workspace_id} mono />
            <Meta label="Workspace Path" value={task.session.workspace_path} mono />
            <Meta label="Thread TS" value={task.session.thread_ts} mono />
            <Meta label="Started" value={new Date(task.run.started_at).toLocaleString()} />
          </div>
        </div>
        <ChatTranscript messages={messages} />
      </section>
      <TerminalStream taskId={task.run.id} />
    </div>
  );
}

function Meta({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="rounded-3xl border border-line bg-fog p-4">
      <p className="text-xs uppercase tracking-[0.18em] text-slate-500">{label}</p>
      <p className={`mt-2 text-sm ${mono ? "font-mono" : ""}`}>{value}</p>
    </div>
  );
}
