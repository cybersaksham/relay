import {
  CancelTaskResponse,
  DashboardResponse,
  DeleteEnvironmentResponse,
  EnvironmentDetail,
  EnvironmentSummary,
  SessionDetail,
  SessionSummary,
  TaskMessage,
} from "@/lib/types";

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;
const SSE_BASE_URL = process.env.NEXT_PUBLIC_SSE_BASE_URL;

if (!API_BASE_URL) {
  throw new Error("NEXT_PUBLIC_API_BASE_URL must be set in the environment");
}

if (!SSE_BASE_URL) {
  throw new Error("NEXT_PUBLIC_SSE_BASE_URL must be set in the environment");
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(await response.text());
  }

  return response.json() as Promise<T>;
}

export async function getDashboard(): Promise<DashboardResponse> {
  return request("/api/tasks");
}

export async function listEnvironments(): Promise<EnvironmentSummary[]> {
  return request("/api/environments");
}

export async function createEnvironment(payload: {
  name: string;
  slug: string;
  git_ssh_url: string;
  default_branch: string;
  aliases: string[];
  enabled?: boolean;
}): Promise<EnvironmentDetail> {
  return request("/api/environments", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export async function updateEnvironment(
  id: string,
  payload: {
    name: string;
    slug: string;
    git_ssh_url: string;
    default_branch: string;
    aliases: string[];
    enabled?: boolean;
  },
): Promise<EnvironmentDetail> {
  return request(`/api/environments/${id}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function deleteEnvironment(id: string): Promise<DeleteEnvironmentResponse> {
  return request(`/api/environments/${id}`, {
    method: "DELETE",
  });
}

export async function getEnvironment(id: string): Promise<EnvironmentDetail> {
  return request(`/api/environments/${id}`);
}

export async function getEnvironmentTasks(id: string): Promise<SessionSummary[]> {
  return request(`/api/environments/${id}/tasks`);
}

export async function getTask(id: string): Promise<SessionDetail> {
  return request(`/api/tasks/${id}`);
}

export async function getTaskMessages(id: string): Promise<TaskMessage[]> {
  return request(`/api/tasks/${id}/messages`);
}

export async function cancelTask(id: string): Promise<CancelTaskResponse> {
  return request(`/api/tasks/${id}/cancel`, {
    method: "POST",
  });
}

export function getSseUrl(path: string): string {
  return `${SSE_BASE_URL}${path}`;
}
