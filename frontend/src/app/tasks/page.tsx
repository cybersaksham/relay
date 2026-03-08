import { ManualTaskForm } from "@/components/manual-task-form";
import { TaskList } from "@/components/task-list";
import { getDashboard, listEnvironments } from "@/lib/api";

export default async function TasksPage() {
  const [dashboard, environments] = await Promise.all([getDashboard(), listEnvironments()]);

  return (
    <div className="page-shell">
      <div>
        <h1 className="text-3xl font-semibold text-ink">Tasks</h1>
      </div>

      <ManualTaskForm environments={environments} />

      <section className="space-y-4">
        <div>
          <h2 className="text-lg font-semibold text-ink">Task History</h2>
        </div>
        <TaskList tasks={dashboard.recent_tasks} />
      </section>
    </div>
  );
}
