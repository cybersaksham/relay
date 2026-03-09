# Backend Agent Guide

## Scope
- Rust service that owns Slack ingestion, orchestration, policy/workflow evaluation, workspace preparation, and API/SSE for the frontend.

## Startup Path
1. `src/main.rs` loads config, ensures runtime dirs, migrates DB.
2. App state wires services (`environments`, `sessions`, `workspaces`, `runner`, `slack`).
3. HTTP server and Slack socket worker run concurrently.

## Key Constraints
- Database is authoritative state.
- Session binding is immutable per thread:
- General thread cannot switch to env.
- Env thread cannot switch to different env.
- One active run per session.

## Implementation Expectations
- Add migration before using new columns/tables.
- Keep SQLx queries explicit and centralized in `src/db/queries.rs`.
- Route-level handlers stay thin; move logic to services/orchestrator.
- Use SSE endpoints for live streaming UX.

## Important Files
- Request flow: `src/tasks/orchestrator.rs`
- Environment + source sync: `src/environments/service.rs`
- Workspace/cache lifecycle: `src/workspaces/session_workspace.rs`
- API/SSE surface: `src/api/*`
