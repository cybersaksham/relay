import Link from "next/link";

import { TaskList } from "@/components/task-list";
import { getDashboard } from "@/lib/api";

export default async function HomePage() {
  const dashboard = await getDashboard();
  const runningCount = dashboard.recent_sessions.filter(
    (session) => session.latest_run?.status === "running",
  ).length;
  const queuedCount = dashboard.recent_sessions.filter(
    (session) => session.latest_run?.status === "queued",
  ).length;

  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Relay Local</h1>
        <p className="mt-2 text-sm text-slate-600">
          Slack-driven local task orchestration with guardrails and workspace isolation.
        </p>
      </div>

      <div className="grid gap-6 xl:grid-cols-[0.9fr_1.1fr]">
        <section className="surface overflow-hidden">
          <div className="surface-header">
            <h2 className="text-lg font-semibold text-ink">System Snapshot</h2>
          </div>
          <div className="surface-body space-y-2 text-sm text-slate-700">
            <p>Environments: {dashboard.environment_count}</p>
            <p>Threads: {dashboard.recent_sessions.length}</p>
            <p>Running: {runningCount}</p>
            <p>Queued: {queuedCount}</p>
            <p>Banned users: 0</p>
          </div>
        </section>

        <section className="surface overflow-hidden">
          <div className="surface-header">
            <h2 className="text-lg font-semibold text-ink">Quick Actions</h2>
          </div>
          <div className="surface-body space-y-3 text-sm">
            <p>
              <Link href="/environments" className="underline decoration-slate-300 underline-offset-4">
                Create or edit environment
              </Link>
            </p>
            <p>
              <Link href="/tasks" className="underline decoration-slate-300 underline-offset-4">
                Run a task manually
              </Link>
            </p>
            <p>
              <Link href="/chats" className="underline decoration-slate-300 underline-offset-4">
                View read-only general chats
              </Link>
            </p>
            <p>
              <Link href="/policies" className="underline decoration-slate-300 underline-offset-4">
                Update guardrail markdown
              </Link>
            </p>
            <p>
              <Link href="/slack" className="underline decoration-slate-300 underline-offset-4">
                Inspect Slack request decisions
              </Link>
            </p>
            <p>
              <Link href="/manage-bot" className="underline decoration-slate-300 underline-offset-4">
                Edit or delete a bot message
              </Link>
            </p>
          </div>
        </section>
      </div>

      <section className="space-y-4">
        <div>
          <h2 className="text-lg font-semibold text-ink">Latest Thread Activity</h2>
        </div>
        <TaskList tasks={dashboard.recent_sessions} />
      </section>
    </div>
  );
}
