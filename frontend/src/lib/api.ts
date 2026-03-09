import {
  CancelTaskResponse,
  DashboardResponse,
  DeleteSlackMessageResponse,
  DeleteEnvironmentResponse,
  EnvironmentDetail,
  EnvironmentSummary,
  ManagedSlackMessage,
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
  source_setup_script?: string | null;
  workspace_setup_script?: string | null;
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
    source_setup_script?: string | null;
    workspace_setup_script?: string | null;
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

export async function refreshEnvironment(id: string): Promise<EnvironmentDetail> {
  return request(`/api/environments/${id}/refresh`, {
    method: "POST",
  });
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

export async function lookupSlackMessage(permalink: string): Promise<ManagedSlackMessage> {
  return request("/api/slack/messages/lookup", {
    method: "POST",
    body: JSON.stringify({ permalink }),
  });
}

export async function updateSlackMessage(payload: {
  channel_id: string;
  ts: string;
  thread_ts?: string | null;
  text: string;
}): Promise<ManagedSlackMessage> {
  return request("/api/slack/messages", {
    method: "PUT",
    body: JSON.stringify(payload),
  });
}

export async function deleteSlackMessage(payload: {
  channel_id: string;
  ts: string;
  thread_ts?: string | null;
}): Promise<DeleteSlackMessageResponse> {
  return request("/api/slack/messages", {
    method: "DELETE",
    body: JSON.stringify(payload),
  });
}

export function getSseUrl(path: string): string {
  return `${SSE_BASE_URL}${path}`;
}

export function getWebSocketUrl(path: string): string {
  const url = new URL(path, API_BASE_URL);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}
