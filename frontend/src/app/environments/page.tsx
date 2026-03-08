import { EnvironmentManager } from "@/components/environment-manager";
import { listEnvironments } from "@/lib/api";

export default async function EnvironmentsPage() {
  const environments = await listEnvironments();

  return <EnvironmentManager initialEnvironments={environments} />;
}
