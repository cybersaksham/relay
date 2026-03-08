# Workflow Format

Workflows live under `.workflows/global/*` or `.workflows/environments/<env-slug>/*`.

Each workflow directory contains:

- `workflow.yaml`: workflow metadata and trigger phrases
- `prompt.md`: prompt template injected into the Codex run

Example metadata:

```yaml
id: pr-review
name: Pull Request Review
scope: global
trigger_phrases:
  - review pr
  - review pull request
default_environment:
instructions:
  - Focus on correctness and regressions.
response_mode: reply
```
