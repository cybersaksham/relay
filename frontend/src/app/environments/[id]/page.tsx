import { EnvironmentDetailManager } from "@/components/environment-detail-manager";
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

  return <EnvironmentDetailManager initialDetail={detail} tasks={tasks} />;
}
