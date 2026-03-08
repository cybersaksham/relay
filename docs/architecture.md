# Relay Architecture

Relay has two runtime processes:

- `backend`: Rust service that owns Slack ingress, policy enforcement, workflow selection, workspace preparation, Codex execution, persistence, and streaming APIs
- `frontend`: Next.js portal that reads backend APIs for operations and monitoring

## Request Lifecycle

1. Slack delivers an app mention through Socket Mode.
2. Relay deduplicates by Slack event ID.
3. Relay checks active bans and then loads the full thread context.
4. Relay evaluates critical deny and non-master policy rules.
5. Relay resolves the environment and workflow.
6. Relay creates or reuses the session workspace.
7. Relay launches Codex in that workspace.
8. Relay persists transcript messages, task metadata, and terminal chunks.
9. Relay posts the final reply back into the Slack thread.
