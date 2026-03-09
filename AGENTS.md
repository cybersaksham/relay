# Relay Monorepo Agent Guide

## Product Intent
- Relay is a Slack-driven engineering assistant with:
- `backend/` (Rust, Axum, SQLx, Slack socket mode, orchestration).
- `frontend/` (Next.js portal for environments, chats/tasks, transcripts, terminals).

## Core Runtime Flow
1. Slack mention enters backend socket handler.
2. Thread context is normalized and policy checks are applied.
3. Environment/workspace is resolved and bound to the thread session.
4. Optional workflow is selected and rendered into a runner prompt.
5. Runner executes, terminal/transcript events are persisted, Slack reply is posted.
6. Portal reads REST data and SSE streams for live state.

## Local-Only Runtime Folders
- `.policies/` and `.workflows/` are local runtime config and are gitignored.
- Runtime workspace/cache is under `~/.relay/`.

## Working Rules For Agents
- Use env-configured ports only; never hardcode ports in code.
- Preserve one-thread-one-workspace binding semantics.
- Keep Slack-facing failures concise; store detailed diagnostics in DB logs/terminal events.
- Add/adjust migrations for DB shape changes; do not silently rely on stale schema.

## Navigation
- Backend architecture: `backend/AGENTS.md`
- Frontend architecture: `frontend/AGENTS.md`
- Product/runtime reference: `docs/*.md`

## Local Skills
- `create-workflow`
- Use when asked to add/update workflow definitions or improve workflow matching behavior.
- Path: `.codex/skills/create-workflow/SKILL.md`
- `create-policy-rules`
- Use when asked to change non-master allow rules, critical deny rules, or policy markdown format.
- Path: `.codex/skills/create-policy-rules/SKILL.md`
- `create-environment`
- Use when asked to onboard/edit/refresh environments, including source/workspace setup scripts.
- Path: `.codex/skills/create-environment/SKILL.md`
