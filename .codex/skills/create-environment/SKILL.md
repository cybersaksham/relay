---
name: create-environment
description: Create, update, or refresh Relay environments with correct git source settings and lifecycle hooks (`source_setup_script`, `workspace_setup_script`). Use when onboarding a repo, fixing cache sync behavior, or configuring environment bootstrap commands.
---

# Create Environment

## Inputs
- `name`
- `slug`
- `git_ssh_url`
- `default_branch` (required)
- `aliases` (optional list)
- `source_setup_script` (optional shell command)
- `workspace_setup_script` (optional shell command)

## API Contracts
- Create: `POST /api/environments`
- Update: `PUT /api/environments/:id`
- Refresh cache: `POST /api/environments/:id/refresh`

## Hook Semantics
- `source_setup_script` runs in source cache (`~/.relay/sources/<env-slug>`) after sync.
- `workspace_setup_script` runs only when a new workspace is provisioned from source cache.
- Failures are fail-fast and must surface concise status errors.

## Implementation Checklist
1. Persist env record and hook scripts.
2. Trigger source sync status flow (`syncing` -> `ready|failed`).
3. Emit sync logs for terminal visibility.
4. Keep refresh conflict-safe while sync is already active.

## Manual Verification
1. Create environment from portal.
2. Open environment detail page.
3. Trigger refresh and watch sync terminal.
4. Confirm cache path exists in `~/.relay/sources/<slug>`.
5. Confirm new thread workspace is created under `~/.relay/workspaces/<slug>/<workspace-id>`.

## Handoff Output
- Return environment id/slug and source path.
- Report hook commands and latest sync status.
