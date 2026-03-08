CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    git_ssh_url TEXT NOT NULL,
    default_branch TEXT NOT NULL,
    aliases TEXT NOT NULL DEFAULT '[]',
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    thread_ts TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    workspace_path TEXT NOT NULL,
    environment_id TEXT NULL REFERENCES environments(id),
    current_workflow_id TEXT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(team_id, channel_id, thread_ts)
);

CREATE TABLE IF NOT EXISTS task_runs (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    trigger_message_ts TEXT NOT NULL,
    status TEXT NOT NULL,
    workflow_id TEXT NULL,
    workflow_name TEXT NULL,
    runner_kind TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT NULL,
    exit_code INTEGER NULL,
    error_summary TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_messages (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    task_run_id TEXT NULL REFERENCES task_runs(id),
    direction TEXT NOT NULL,
    slack_user_id TEXT NULL,
    raw_payload TEXT NOT NULL,
    resolved_payload TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS terminal_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_run_id TEXT NOT NULL REFERENCES task_runs(id),
    stream TEXT NOT NULL,
    chunk TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS policy_violations (
    id TEXT PRIMARY KEY NOT NULL,
    slack_user_id TEXT NOT NULL,
    team_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    thread_ts TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    request_excerpt TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS bans (
    id TEXT PRIMARY KEY NOT NULL,
    slack_user_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS slack_event_dedup (
    event_id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_lookup ON sessions(team_id, channel_id, thread_ts);
CREATE INDEX IF NOT EXISTS idx_task_runs_session ON task_runs(session_id);
CREATE INDEX IF NOT EXISTS idx_terminal_events_task_run_id_id ON terminal_events(task_run_id, id);
CREATE INDEX IF NOT EXISTS idx_policy_violations_user_time ON policy_violations(slack_user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_bans_user_time ON bans(slack_user_id, expires_at);
