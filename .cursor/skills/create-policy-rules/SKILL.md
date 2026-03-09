---
name: create-policy-rules
description: Create or update Relay policy files under `.policies/` for non-master allow rules and critical deny rules using the markdown format parsed by backend. Use when asked to change access restrictions, deny categories, or ban-triggering behavior.
---

# Create Policy Rules

## Target Files
- `.policies/non-master.md`
- `.policies/critical-deny.md`

## Required Structure
- File must start with YAML frontmatter.
- Rules must be `##` sections.
- Each rule must include:
- `- id: <rule-id>`
- `### Match` with one or more `- <term>`
- `### Examples` with one or more `- <example>`
- `### Notes` is optional.

Use this exact skeleton:

```md
---
name: <Policy Name>
description: <Optional description>
---

## <Rule Title>
- id: <rule-id>
### Match
- <case-insensitive substring term>
### Examples
- <example request>
### Notes
- <optional implementation note>
```

## Writing Rules
- Keep `Match` terms short and concrete.
- Add at least two examples per rule when possible.
- For critical deny, include system-risk actions (delete/destructive/escalation).
- For non-master, include only explicitly allowed task classes.

## Validate
- Ensure no rule is missing `id`, `Match`, or `Examples`.
- Ensure rule ids are unique within each file.
- Run backend tests:

```bash
cd backend && cargo test
```

## Handoff Output
- Return updated file paths.
- Summarize new/changed rule ids and intent.
