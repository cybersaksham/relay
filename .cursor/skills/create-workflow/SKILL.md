---
name: create-workflow
description: Create or update Relay workflow definitions in `.workflows/global/*` or `.workflows/environments/env-slug/*` with valid `workflow.yaml` metadata and high-fidelity `prompt.md`. Use when asked to add, edit, tune, or fix workflow selection behavior.
---

# Create Workflow

## Gather Inputs
- Collect workflow intent, scope (`global` or `environment`), workflow id, and trigger phrases.
- If scope is environment, require a concrete environment slug.

## Choose Directory
- Global workflow path: `.workflows/global/<workflow-id>/`
- Environment workflow path: `.workflows/environments/<env-slug>/<workflow-id>/`

## Write `workflow.yaml`
- Include exactly these fields:
- `id`
- `name`
- `scope` (`global` or `environment`)
- `trigger_phrases` (non-empty list)
- `default_environment` (nullable/empty for global unless explicitly required)
- `instructions` (list of behavior constraints)
- `response_mode` (`reply` unless explicitly requested otherwise)

Use this template:

```yaml
id: <workflow-id>
name: <Human Name>
scope: <global|environment>
trigger_phrases:
  - <phrase-1>
default_environment:
instructions:
  - <instruction-1>
response_mode: reply
```

## Write `prompt.md`
- Encode execution contract clearly and deterministically.
- State required tools/skills and expected output shape.
- If task type is review-style, instruct findings-first output with concise summaries.
- If task type is action-style, instruct exact completion criteria and confirmation payload.

## Validate
- Confirm workflow files exist in the expected path.
- Confirm `id` matches directory name.
- Confirm trigger phrases are explicit enough to avoid accidental generic fallback.
- Run backend tests if selection behavior changed:

```bash
cd backend && cargo test
```

## Handoff Output
- Return created/updated workflow path.
- Return final `workflow.yaml` and key prompt constraints.
