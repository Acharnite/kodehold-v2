---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0005: Plugin Architecture for Per-Workspace Customization

## Status

Proposed

## Context

KodeHold v1.0's workspace management is centralized in `scripts/workspace.py`
(1261 lines). This single script handles:

- Workspace creation, adoption, listing, state queries
- Gate transitions (delegates to gate.py)
- Loop management (7 patterns: enable, disable, run, audit)
- Cron management (install, remove, list)
- Cost estimation (token cost per pattern)

When a workspace needs custom behavior (e.g., krypto-agent needs different
test execution than deepresearch), the change must be made in workspace.py
and affects ALL workspaces. This creates:

1. **Monolithic coupling**: Every workspace change risks breaking other
   workspaces.
2. **Lowest-common-denominator**: Features that only some workspaces need
   either bloat workspace.py or don't get built.
3. **No workspace-level overrides**: You can't customize the gate check
   for a specific project without editing gate.py.
4. **Script dependency**: Workspaces implicitly depend on kodehold's
   Python environment being available.

## Decision

Replace the monolithic manager with a plugin architecture.

### Architecture

```
~/.kodehold/bin/kodehold       ← Single core binary (ADR-0008)
│
├── Built-in commands (gate, verify, ship, state)
│
└── Per-workspace plugins
    my-project/
    ├── kodehold.yaml           ← Workspace config
    └── hooks/
        ├── pre-generate        ← Runs before generate phase
        ├── post-verify         ← Runs after verify phase
        ├── on-ship             ← Runs on ship
        └── custom-gate         ← Custom gate logic (overrides default)
```

### Plugin Interface

Plugins are executables (any language, any binary). The core engine communicates
via stdin/stdout JSON:

```
stdin  → {"event": "pre-generate", "context": {...}, "workspace": "..."}
stdout → {"status": "ok", "modified_context": {...}}  or
         {"status": "blocked", "reason": "lint failing"}
stderr → free-form log output
exit 0 → proceed
exit 1 → block pipeline
```

### kodehold.yaml Schema

```yaml
# kodehold.yaml — per-workspace configuration
version: "2.0"
name: my-trading-bot
language: rust

pipeline:
  generate:
    fallback: ollama/qwen3.5:9b
  verify:
    commands:
      - cargo test
      - cargo clippy -- -D warnings
  optimize:
    commands:
      - cargo fmt --check

hooks:
  pre-generate: hooks/pre-generate
  post-verify: hooks/post-verify
  on-ship: hooks/on-ship

events:
  - github_webhook:
      endpoint: /hooks/github
  - inotify:
      paths: [src/]

custom_gates:
  - path: hooks/custom-gate
    overrides: [REVIEW_TO_CLOSED]
    reason: "Custom compliance check for financial software"
```

### Plugin Discovery

1. Core loads `kodehold.yaml` → finds hook paths
2. Validates each hook is executable (`chmod +x`)
3. Runs hooks at defined pipeline points
4. If a hook overrides a built-in gate, the hook runs instead of the default

## Consequences

### Positive

- Workspaces are self-contained: clone the repo and you have everything
- Any language can be a plugin (shell script, Python, compiled binary)
- Core engine stays small — custom logic lives in workspaces
- Teams can ship custom gates without waiting for kodehold core updates
- Workspace-specific behavior doesn't risk other workspaces

### Negative

- Plugin quality varies: no guarantee a workspace's hooks are well-tested
- Security: hooks execute arbitrary code. Must trust the workspace author.
- Debugging: pipeline failure could be in core OR a plugin — harder to trace
- Discoverability: no central registry of useful plugins

### Risks

- **Plugin isolation**: A malicious plugin could read secrets from other
  workspaces. Mitigation: sandbox plugins (Docker/podman container per hook).
- **Performance**: Plugin startup cost per pipeline phase. Mitigation: cache
  plugin results, allow long-running plugin daemons.

### Migration

V1.0 workspace.py features map to v2.0 plugins:
- `workspace.py gate` → `kodehold gate` (built-in) + `hooks/custom-gate` (plugin override)
- `workspace.py loop enable daily-triage` → `kodehold.yaml` events section
- `workspace.py cron install` → `kodehold.yaml` events.schedule
- `workspace.py cost daily-triage` → built-in `kodehold cost` command
