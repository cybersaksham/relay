# Relay

Relay is a Slack-triggered control plane for Codex-driven tasks. It combines:

- a Rust backend for Slack Socket Mode, policy enforcement, environment/workspace management, task orchestration, and persistence
- a Next.js frontend for environment management and task monitoring

## Monorepo Layout

- `backend/`: Rust API server, Slack worker, migrations, orchestration logic
- `frontend/`: Next.js portal for environments, tasks, transcript, and live terminal output
- `.policies/`: Markdown policy definitions for non-master allow rules and critical deny rules
- `.workflows/`: File-based workflow definitions and prompt templates
- `docs/`: Architecture and format references

## Runtime Layout

Relay keeps its runtime assets under `~/.relay` by default:

```text
~/.relay/
├── sources/
│   └── <env-slug>/
│       └── <repo clone>
└── workspaces/
    ├── <env-slug>/
    │   └── <workspace-id>/
    └── general/
        └── <workspace-id>/
```

## Backend Setup

1. Edit `backend/.env` with the real backend values. Keep `backend/.env.example` only as the template structure.
2. Start the backend:

```bash
cd backend
set -a
source .env
set +a
cargo run
```

## Frontend Setup

1. Edit `frontend/.env` with the real frontend values. Keep `frontend/.env.example` only as the template structure.
2. Install dependencies and start the portal:

```bash
cd frontend
set -a
source .env
set +a
npm install
npm run dev
```

## Run Both Services

Use the launcher script to start backend and frontend together from the repo root:

```bash
./scripts/run_server
```

The script reads `backend/.env` and `frontend/.env`, starts both services together, and cleans up both listening ports on exit.

## Docker Compose

1. Edit `.env` for compose-level port values.
2. Edit `backend/.env` for backend runtime values.
3. Edit `frontend/.env` for frontend runtime values.
4. Start both services:

```bash
docker compose --env-file .env up --build
```

## Development Notes

- Local DB default: SQLite via `DATABASE_URL=sqlite:relay.db`
- Socket Mode requires both `SLACK_BOT_TOKEN` and `SLACK_APP_TOKEN`
- Codex execution uses the `codex` binary from `PATH` unless overridden by `CODEX_BIN`
- Workspace terminal sessions use `TERMINAL_COMMAND` and default to `/bin/zsh`
- All Git operations use the system `git` and SSH configuration directly.
- Backend and frontend ports are required env values. Neither service falls back to an in-code default port.
- Environment creation requires an explicit default branch. There is no global branch fallback in env.
