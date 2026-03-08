# Policy Format

Policy files live under `.policies/` and must use this structure:

```md
---
name: Policy Name
description: Optional description
---

## Rule Title
- id: rule-id
### Match
- phrase one
- phrase two
### Examples
- Example request one
- Example request two
### Notes
- Optional note
```

Rules are matched by case-insensitive substring match against the normalized request text.
