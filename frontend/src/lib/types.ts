export type TaskStatus =
  | "queued"
  | "running"
  | "waiting_for_reply"
  | "succeeded"
  | "failed"
  | "blocked"
  | "cancelled"
  | "timed_out"
  | "idle";

export interface EnvironmentSummary {
  id: string;
  name: string;
  slug: string;
  git_ssh_url: string;
  default_branch: string;
  aliases: string;
  enabled: boolean;
  source_sync_status: "pending" | "syncing" | "ready" | "failed" | string;
  source_sync_error: string | null;
  source_synced_at: string | null;
  source_setup_script: string | null;
  workspace_setup_script: string | null;
  created_at: string;
  updated_at: string;
}

export interface EnvironmentDetail {
  environment: EnvironmentSummary;
  source_path: string;
}

export interface DeleteEnvironmentResponse {
  deleted_id: string;
}

export interface TaskSummary {
  id: string;
  session_id: string;
  trigger_message_ts: string;
  status: TaskStatus;
  workflow_id: string | null;
  workflow_name: string | null;
  runner_kind: string;
  started_at: string;
  finished_at: string | null;
  exit_code: number | null;
  error_summary: string | null;
  created_at: string;
  updated_at: string;
}

export interface Session {
  id: string;
  team_id: string;
  channel_id: string;
  thread_ts: string;
  workspace_id: string;
  workspace_path: string;
  environment_id: string | null;
  current_workflow_id: string | null;
  status: TaskStatus;
  created_at: string;
  updated_at: string;
}

export interface TaskDetail {
  run: TaskSummary;
  session: Session;
}

export interface TaskMessage {
  id: string;
  session_id: string;
  task_run_id: string | null;
  direction: "inbound" | "outbound";
  slack_user_id: string | null;
  raw_payload: string;
  resolved_payload: string;
  created_at: string;
}

export interface TerminalEvent {
  id: number;
  task_run_id: string;
  stream: "stdout" | "stderr";
  chunk: string;
  sequence: number;
  created_at: string;
}

export interface DashboardResponse {
  environment_count: number;
  recent_sessions: SessionSummary[];
}

export interface SessionSummary {
  session: Session;
  latest_run: TaskSummary | null;
  run_count: number;
}

export interface SessionDetail {
  session: Session;
  latest_run: TaskSummary | null;
  runs: TaskSummary[];
}

export interface CancelTaskResponse {
  task_run_id: string;
  status: string;
}
