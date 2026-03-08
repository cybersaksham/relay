import { TaskList } from "@/components/task-list";
import { getEnvironment, getEnvironmentTasks } from "@/lib/api";

export default async function EnvironmentDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const [detail, tasks] = await Promise.all([
    getEnvironment(id),
    getEnvironmentTasks(id),
  ]);

  return (
    <div className="space-y-8">
      <section className="rounded-[2rem] border border-line bg-white/80 p-8 shadow-panel">
        <p className="mb-2 text-xs uppercase tracking-[0.3em] text-accent">Environment</p>
        <h1 className="text-3xl font-semibold">{detail.environment.name}</h1>
        <p className="mt-1 text-slate-600">{detail.environment.slug}</p>
        <div className="mt-6 grid gap-4 md:grid-cols-2">
          <InfoCard label="Repo" value={detail.environment.git_ssh_url} mono />
          <InfoCard label="Default branch" value={detail.environment.default_branch} />
          <InfoCard label="Source path" value={detail.source_path} mono />
          <InfoCard label="Aliases" value={detail.environment.aliases} mono />
        </div>
      </section>
      <section className="space-y-4">
        <div>
          <h2 className="text-2xl font-semibold">Tasks</h2>
          <p className="text-sm text-slate-600">
            All task runs currently associated with this environment.
          </p>
        </div>
        <TaskList tasks={tasks} />
      </section>
    </div>
  );
}

function InfoCard({
  label,
  value,
  mono = false,
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
