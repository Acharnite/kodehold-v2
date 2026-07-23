---
name: workspace-loop-management
description: >
  Manage loops in KodeHold workspaces using loop-engineering integration.
  Use this skill when you need to enable, disable, run, or monitor loops in workspaces.
user_invocable: true
---

# Workspace Loop Management Skill

You manage loops in KodeHold workspaces using the workspace.py CLI commands.

## Available Commands

### Loop Management

```bash
# List active loops in a workspace
workspace.py loop <name> list

# Enable a loop pattern
workspace.py loop <name> enable <pattern>

# Disable a loop pattern
workspace.py loop <name> disable <pattern>

# Run a loop manually
workspace.py loop <name> run <pattern>
```

### Cron Management

```bash
# Install crontab entries for all workspaces
workspace.py cron install

# Remove crontab entries
workspace.py cron remove

# Show current crontab entries
workspace.py cron list
```

### Monitoring

```bash
# Run loop-audit to check loop readiness
workspace.py audit <name>

# Estimate token cost for a pattern
workspace.py cost <name> <pattern>

# Check drift between STATE.md and LOOP.md
workspace.py sync <name>
```

## Supported Patterns

| Pattern | Cadence | Risk | Token Cost | Level |
|---------|---------|------|------------|-------|
| Daily Triage | 1d–2h | Low | Low (~50k/run) | L1 |
| PR Babysitter | 5–15m | Medium | High (~250k/run) | L1→L2 |
| CI Sweeper | 5–15m | Medium | Very high (~200k/run) | L2 |
| Dependency Sweeper | 6h–1d | Medium | Medium (~300k/run) | L2 |
| Changelog Drafter | 1d | Low | Low (~35k/run) | L1 |
| Post-Merge Cleanup | 1d–6h | Low | Low (~40k/run) | L1 |
| Issue Triage | 2h–1d | Low | Low (~30k/run) | L1 |

## Workflow

### Setting up loops for a workspace

1. Create or adopt a workspace:
   ```bash
   workspace.py init <name>
   # or
   workspace.py adopt <name> <path>
   ```

2. Enable patterns:
   ```bash
   workspace.py loop <name> enable daily-triage
   workspace.py loop <name> enable issue-triage
   ```

3. Install crontab:
   ```bash
   workspace.py cron install
   ```

4. Verify setup:
   ```bash
   workspace.py audit <name>
   ```

### Running loops manually

```bash
# Run daily triage
workspace.py loop <name> run daily-triage

# Run issue triage
workspace.py loop <name> run issue-triage
```

### Monitoring loops

```bash
# Check loop readiness
workspace.py audit <name>

# Estimate token cost
workspace.py cost <name> daily-triage

# Check STATE.md ↔ LOOP.md drift
workspace.py sync <name>
```

## Rules

- Start with L1 (report-only) patterns before enabling L2 (auto-fix) patterns
- Always run `workspace.py audit <name>` after enabling new patterns
- Check `workspace.py cost <name> <pattern>` to understand token implications
- Use `workspace.py sync <name>` to detect configuration drift
- Never enable L2 patterns without verifier skill and safety docs

## References

- [ADR-0060](docs/adr/ADR-0060-loop-engineering-integration.md) — Loop-Engineering Integration
- [loop-engineering](https://github.com/cobusgreyling/loop-engineering) — Upstream project
- [workspace.py](scripts/workspace.py) — CLI implementation
