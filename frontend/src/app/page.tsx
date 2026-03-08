import Link from "next/link";

import { TaskList } from "@/components/task-list";
import { getDashboard } from "@/lib/api";

export default async function HomePage() {
  const dashboard = await getDashboard();

  return (
    <div className="space-y-8">
      <section className="rounded-[2rem] border border-line bg-white/80 p-8 shadow-panel">
        <div className="flex flex-col gap-5 md:flex-row md:items-end md:justify-between">
          <div className="max-w-2xl">
            <p className="mb-3 text-xs uppercase tracking-[0.3em] text-accent">
              Relay Control Plane
            </p>
            <h1 className="text-4xl font-semibold leading-tight text-ink">
              Operate Slack-triggered Codex tasks with pinned workspaces, strict policy gates, and live run visibility.
            </h1>
          </div>
          <div className="rounded-3xl bg-accentSoft px-6 py-5">
            <p className="text-sm text-slate-600">Environment count</p>
            <p className="text-4xl font-semibold text-accent">{dashboard.environment_count}</p>
          </div>
        </div>
        <div className="mt-6 flex gap-3">
          <Link
            href="/environments"
            className="rounded-full bg-accent px-5 py-2.5 text-sm font-medium text-white"
          >
            Manage Environments
          </Link>
        </div>
      </section>

      <section className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold">Recent Tasks</h2>
            <p className="text-sm text-slate-600">Latest Relay runs across all environments.</p>
          </div>
        </div>
        <TaskList tasks={dashboard.recent_tasks} />
      </section>
    </div>
  );
}
