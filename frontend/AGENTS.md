# Frontend Agent Guide

## Scope
- Next.js portal for monitoring and managing Relay runtime:
- environments
- tasks/chats
- transcripts
- live terminal/event streams

## Architecture
- App Router pages in `src/app`.
- Stateful UI components in `src/components`.
- Typed backend client and contracts in `src/lib`.

## Rules
- Treat backend as source of truth; do not invent client-side state transitions.
- Use SSE for live views (task terminal/status, environment sync logs).
- Keep terminal panes read-only.

## Important Files
- Environment pages: `src/app/environments/*`
- Environment/task UI: `src/components/*`
- API/SSE contracts: `src/lib/api.ts`, `src/lib/types.ts`, `src/lib/sse.ts`
