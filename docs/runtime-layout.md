# Runtime Layout

Relay defaults to `~/.relay` for runtime storage:

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

- `sources/` is the canonical source clone cache per environment
- `workspaces/` contains per-session workspaces that persist across follow-up mentions in the same Slack thread
