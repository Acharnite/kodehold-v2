---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0060: Loop-Engineering Integration for Workspace Management

## Version

- **v1.0 (2026-07-22):** Original — loop-engineering as external tool for workspace loop management
- **v1.1 (2026-07-22):** Updated per Reviewers and Second Opinion — added Documentation section, ADR-0048/049 compliance, version pinning, CLI integration details, testing strategy, error handling, marked crontab as provisional
- **v1.2 (2026-07-22):** Implementation complete — all Phase 1-3 features implemented and tested

## Status

Implemented

## Implementation Status

| Phase | Scope | Status | Verified |
|-------|-------|--------|----------|
| **Phase 1** | loop-init integration + loop subcommand | ✅ Complete | ✅ |
| **Phase 2** | enable/disable/run + cron commands | ✅ Complete | ✅ |
| **Phase 3** | audit/cost/sync commands | ✅ Complete | ✅ |

### Implemented Commands

```bash
# Loop management
workspace.py loop <name> list              # List active loops
workspace.py loop <name> enable <pattern>  # Enable a pattern
workspace.py loop <name> disable <pattern> # Disable a pattern
workspace.py loop <name> run <pattern>     # Run a loop manually

# Cron management
workspace.py cron install                  # Install crontab entries
workspace.py cron remove                   # Remove crontab entries
workspace.py cron list                     # Show crontab entries

# Monitoring
workspace.py audit <name>                  # Run loop-audit
workspace.py cost <name> <pattern>         # Estimate token cost
workspace.py sync <name>                   # Check STATE.md ↔ LOOP.md drift
```

## Documentation

| Field | Value |
|-------|-------|
| **Tool** | `@cobusgreyling/loop-engineering` (npx CLI tools) |
| **Official docs** | https://github.com/cobusgreyling/loop-engineering |
| **Version** | v1.6.0 (as of 2026-07-22) |
| **License** | MIT |

### Key CLI Tools

| Tool | Purpose | Command |
|------|---------|---------|
| `loop-init` | Scaffold loop files into workspace | `npx @cobusgreyling/loop-init . --pattern <pattern> --tool opencode` |
| `loop-audit` | Score loop readiness (0-100) | `npx @cobusgreyling/loop-audit . --suggest` |
| `loop-cost` | Estimate token cost per pattern | `npx @cobusgreyling/loop-cost --pattern <pattern> --level L1` |
| `loop-sync` | Detect drift STATE.md ↔ LOOP.md | `npx @cobusgreyling/loop-sync .` |
| `loop-context` | Circuit breaker for L2+ loops | `npx @cobusgreyling/loop-context --check --ledger run.json` |
| `loop-worktree` | Manage isolated git worktrees | `npx @cobusgreyling/loop-worktree create --run-id <id> --pattern <p>` |
| `loop-gate` | Enforce safety policy | `npx @cobusgreyling/loop-gate check --action auto-merge --paths <f1,f2,...>` |

### Key API Concepts

**Authentication patterns:**
- No authentication required for CLI tools
- opencode CLI must be authenticated separately for `opencode run` commands
- GitHub MCP requires token for PR/issue access (optional)

**Endpoint patterns:**
- All tools are local CLI commands, no remote API calls
- `npx` resolves packages from npm registry (requires internet)
- `opencode run` invokes local opencode CLI

**Config requirements:**
- `opencode.json` at project root defines agent configurations
- `skills/` directory at project root for skill discovery
- `STATE.md`, `LOOP.md` at project root for state management
- `gate.yaml` at project root for safety policy (optional)

**Version-specific behaviors:**
- v1.6.0: loop-context supports `--budget-from-pattern` for token budget resolution
- v1.5.0: loop-init added `--with-foundry` for harness-foundry integration
- v1.4.0: loop-audit added loopActivity dynamic proof scoring

**Gotchas (expanded):**
1. **npx requires internet** — first run downloads packages; offline environments need pre-installation via `npm install -g @cobusgreyling/loop-init`
2. **Pre-v1.0 API may change** — loop-engineering is v1.6.0 but CLI interfaces may evolve; pin versions for production
3. **`opencode run` syntax needs verification** — exact CLI flags may vary by opencode version; verify before implementation
4. **State files are pattern-specific** — each pattern has its own state file (STATE.md, pr-babysitter-state.md, etc.)
5. **Skills directory must be at project root** — opencode auto-discovers `skills/` at repo root
6. **Nested subcommands require argparse refactoring** — current flat CLI structure needs modification for `workspace.py loop` commands

### Configuration Prerequisites

- Node.js and npm installed
- opencode CLI installed and authenticated
- Git repository with remote (for PR-based patterns)
- Internet connection for npx commands (or pre-installed packages)

### Version Pinning Strategy

To prevent breaking changes from upstream affecting workspaces:

```bash
# Pin to specific version for production
npx @cobusgreyling/loop-init@1.6.0 . --pattern daily-triage --tool opencode

# Or use package.json for project-level pinning
npm install --save-dev @cobusgreyling/loop-init@1.6.0
```

**Recommendation:** Pin to minor version (e.g., `@1.6.0`) for stability, update quarterly after testing.

### ADR-0058 Relationship

**ADR-0060 supersedes ADR-0058.** ADR-0058 (Loop Engineering Integration) was archived because:
- It duplicated loop patterns instead of using loop-engineering as a tool
- It created custom loop_runner.py that required ongoing maintenance
- It only implemented 3 of 7 available patterns

**ADR-0060 addresses these failures by:**
- Using loop-engineering's CLI tools directly (no duplication)
- Leveraging community maintenance and updates
- Providing access to all 7 patterns
- Maintaining separation of concerns (kodehold = workspaces, loop-engineering = loops)

**Existing loop_runner.py code:** Will be removed as part of ADR-0060 implementation. The crontab entries will be replaced with `opencode run` commands.

### Gotchas

1. **npx requires internet** — first run downloads packages; offline environments need pre-installation
2. **Pre-v1.0 API may change** — loop-engineering is v1.6.0 but CLI interfaces may evolve
3. **`opencode run` syntax needs verification** — exact CLI flags may vary by opencode version
4. **State files are pattern-specific** — each pattern has its own state file (STATE.md, pr-babysitter-state.md, etc.)
5. **Skills directory must be at project root** — opencode auto-discovers `skills/` at repo root

## Context

KodeHold manages 6 workspaces (bob, deepresearch, krypto-agent, media-health-dashboard, pai-model-router, radarr-lang-router). Currently, workspace management is limited to:

- Creating/adopting workspaces
- Managing lifecycle states (INIT → ACTIVE → REVIEW → CLOSED)
- Gate transitions

There is no automated loop management for workspaces. The previous attempt (ADR-0058) tried to integrate loop-engineering directly into kodehold, but this created duplication and maintenance overhead. ADR-0058 was archived because:

1. **Duplication** — kodehold recreated loop patterns that already existed in loop-engineering
2. **Maintenance burden** — custom loop_runner.py required ongoing maintenance
3. **Missing patterns** — only 3 of 7 patterns were implemented
4. **No upstream benefits** — updates from loop-engineering community were not leveraged

Meanwhile, loop-engineering (cobusgreyling/loop-engineering, 9k stars) provides a mature ecosystem:

- 7 production patterns (Daily Triage, PR Babysitter, CI Sweeper, Dependency Sweeper, Changelog Drafter, Post-Merge Cleanup, Issue Triage)
- CLI tools (loop-init, loop-audit, loop-cost, loop-sync, loop-context, loop-worktree, loop-gate)
- Opencode integration via `opencode run` commands
- Active community and maintenance

The key insight: **kodehold should use loop-engineering as a tool, not duplicate it.**

## Decision

We integrate loop-engineering as a **tool** that kodehold uses to set up and manage loops in workspaces, rather than duplicating loop functionality within kodehold.

### ADR-0049 Justification (The Ladder)

**Stdlib alternative evaluated:** Building loop management from scratch would:
- Duplicate 7 existing patterns with ~5000 lines of code
- Lose community updates and bug fixes from 9k-star repository
- Require ongoing maintenance burden on kodehold team
- Miss proven safety patterns (circuit breaker, worktree isolation, gate enforcement)

**Dependency justified:** loop-engineering provides mature, tested, community-maintained loop patterns that align with kodehold's workspace management goals. The dependency is external CLI tools (npx), not library imports, minimizing coupling.

### Key Principles

1. **kodehold** = workspace/project management tool
2. **loop-engineering** = tool that kodehold calls to manage loops in workspaces
3. **Separation of concerns** — kodehold handles workspaces, loop-engineering handles loops
4. **No duplication** — use loop-engineering's existing CLI tools
5. **Upstream leverage** — benefit from loop-engineering's community and updates

### Integration Points

| Kodehold Action | loop-engineering Call |
|-----------------|----------------------|
| `workspace.py init <name>` | `npx @cobusgreyling/loop-init . --pattern daily-triage --tool opencode` |
| `workspace.py adopt <name>` | `npx @cobusgreyling/loop-init . --pattern daily-triage --tool opencode` |
| `workspace.py loop <name> enable <pattern>` | Update `loops.enabled` in workspace config |
| `workspace.py loop <name> run <pattern>` | `opencode run "Run <pattern>" --agent <agent>` (syntax to be verified before implementation) |
| `workspace.py cron install` | Generate crontab entries |
| `workspace.py audit <name>` | `npx @cobusgreyling/loop-audit . --suggest` |
| `workspace.py cost <name> <pattern>` | `npx @cobusgreyling/loop-cost --pattern <pattern> --level L1` |
| `workspace.py sync <name>` | `npx @cobusgreyling/loop-sync .` |

### New Commands

```bash
# Loop management
workspace.py loop <name> enable <pattern>    # Activate a pattern
workspace.py loop <name> disable <pattern>   # Deactivate a pattern
workspace.py loop <name> run <pattern>       # Run a loop manually
workspace.py loop <name> list                # Show active loops

# Crontab management
workspace.py cron install                   # Install crontab entries
workspace.py cron remove                    # Remove crontab entries
workspace.py cron list                      # Show current crontab

# Monitoring
workspace.py audit <name>                   # Run loop-audit in workspace
workspace.py cost <name> <pattern>          # Estimate token cost
workspace.py sync <name>                    # Check drift STATE.md ↔ LOOP.md
```

### Supported Patterns

| Pattern | Cadence | Risk | Token Cost | Level |
|---------|---------|------|------------|-------|
| Daily Triage | 1d–2h | Low | Low (~50k/run) | L1 |
| PR Babysitter | 5–15m | Medium | High (~250k/run) | L1→L2 |
| CI Sweeper | 5–15m | Medium | Very high (~200k/run) | L2 |
| Dependency Sweeper | 6h–1d | Medium | Medium (~300k/run) | L2 |
| Changelog Drafter | 1d | Low | Low (~35k/run) | L1 |
| Post-Merge Cleanup | 1d–6h | Low | Low (~40k/run) | L1 |
| Issue Triage | 2h–1d | Low | Low (~30k/run) | L1 |

### Workspace Configuration

Each workspace gets loop-engineering files:

```
workspaces/<name>/
├── STATE.md                    # Loop state
├── LOOP.md                     # Loop descriptions
├── AGENTS.md                   # Agent rules
├── loop-budget.md              # Token logs
├── loop-constraints.md         # Safety rules
├── loop-run-log.md             # Run history
├── gate.yaml                   # Safety policy
├── skills/
│   └── loop-triage/SKILL.md    # Triage skill
├── .opencode/
│   └── opencode.json           # Agent definitions
└── .kodehold-state             # KodeHold lifecycle state
```

### Crontab Structure

**⚠️ Provisional — `opencode run` syntax must be verified before implementation.**

```bash
# Daily Triage for deepresearch - every weekday at 08:00
# VERIFY: opencode run syntax may vary by version
0 8 * * 1-5 cd /home/kiffer/project/kodehold/workspaces/deepresearch && opencode run "Run loop-triage. Read STATE.md first. Update High Priority and Watch List. No auto-fix in week one." --agent loop-triage 2>&1 | tee -a /tmp/loop-deepresearch.log

# Daily Triage for krypto-agent - every weekday at 08:05
5 8 * * 1-5 cd /home/kiffer/project/kodehold/workspaces/krypto-agent && opencode run "Run loop-triage. Read STATE.md first. Update High Priority and Watch List. No auto-fix in week one." --agent loop-triage 2>&1 | tee -a /tmp/loop-krypto-agent.log

# PR Babysitter for deepresearch - every 4 hours
0 8,12,16 * * 1-5 cd /home/kiffer/project/kodehold/workspaces/deepresearch && opencode run "Run pr-babysitter. Read pr-babysitter-state.md. Watch open PRs. No code changes." --agent pr-babysitter 2>&1 | tee -a /tmp/loop-deepresearch.log
```

### Dataflow

```
kodehold                              loop-engineering
──────────                            ─────────────────
workspace.py init →                   loop-init in workspace
workspace.py loop enable →            updates loops.enabled config
workspace.py loop run →               opencode run with pattern
workspace.py cron install →           crontab entries
workspace.py audit →                  loop-audit in workspace
workspace.py cost →                   loop-cost for pattern
workspace.py sync →                   loop-sync in workspace
```

### Testing Strategy

| Test Type | Scope | Method |
|-----------|-------|--------|
| **Unit tests** | `workspace.py loop/cron/audit` subcommands | Mock npx calls, verify CLI parsing |
| **Integration tests** | `loop-init` → workspace creation | Real npx calls in test directory |
| **Manual testing** | Crontab entries, opencode run syntax | Verify on dev machine before deployment |
| **Regression tests** | Existing workspace.py commands | Ensure nested subcommands don't break existing functionality |

**Per ADR-0047:** All new subcommands must have unit tests before acceptance.

### CLI Integration Details

Current `workspace.py` uses flat argparse subcommands. Adding nested subcommands requires:

```python
# Current structure
parser = argparse.ArgumentParser()
subparsers = parser.add_subparsers(dest='command')
subparsers.add_parser('init')
subparsers.add_parser('adopt')
# ...

# New structure with nested subcommands
parser = argparse.ArgumentParser()
subparsers = parser.add_subparsers(dest='command')
init_parser = subparsers.add_parser('init')
adopt_parser = subparsers.add_parser('adopt')
# ...

# Loop subcommand group
loop_parser = subparsers.add_parser('loop')
loop_subparsers = loop_parser.add_subparsers(dest='loop_command')
loop_subparsers.add_parser('enable')
loop_subparsers.add_parser('disable')
loop_subparsers.add_parser('run')
loop_subparsers.add_parser('list')

# Cron subcommand group
cron_parser = subparsers.add_parser('cron')
cron_subparsers = cron_parser.add_subparsers(dest='cron_command')
cron_subparsers.add_parser('install')
cron_subparsers.add_parser('remove')
cron_subparsers.add_parser('list')
```

**Backward compatibility:** Existing commands (`init`, `adopt`, `list`, `state`, `gate`, `deploy-ready`, `ensure-git`) remain unchanged. Nested subcommands are additive.

### Phased Rollout

| Phase | Scope | Sessions | Gate |
|-------|-------|----------|------|
| **Phase 1** | `workspace.py init/adopt` + `loop-init` integration + Daily Triage pattern | 2 | loop-audit score ≥ 40 |
| **Phase 2** | `workspace.py loop` commands + remaining patterns (PR Babysitter, Issue Triage) + crontab | 3 | loop-audit score ≥ 70 |
| **Phase 3** | `workspace.py audit/cost/sync` monitoring + L2 patterns (CI Sweeper, Dependency Sweeper) | 2 | loop-audit score ≥ 80 |

### Effort Estimates

| Task | Effort | Dependencies |
|------|--------|--------------|
| Add `loop-init` call to `workspace.py init/adopt` | 0.5 sessions | None |
| Create `workspace.py loop` subcommand | 1 session | Phase 1 |
| Create `workspace.py cron` subcommand | 1 session | Phase 2 |
| Create `workspace.py audit/cost/sync` subcommands | 1 session | Phase 3 |
| Test with existing workspaces | 0.5 sessions | All phases |
| Documentation and ADR updates | 0.5 sessions | All phases |
| **Total** | **4.5 sessions** | — |

## Consequences

### Positive

- **Access to all 7 patterns** — not just 3 as before
- **No duplication** — uses loop-engineering's existing tools
- **Community updates** — automatic from loop-engineering's npm packages
- **Flexibility** — each workspace can have different patterns
- **Token estimation** — loop-cost provides realistic estimates
- **Drift detection** — loop-sync detects when STATE.md and LOOP.md are out of sync
- **Security** — loop-gate, loop-context, loop-constraints are built-in

### Negative

- **External dependency** — requires loop-engineering npm packages
- **Internet connection** — npx commands require internet
- **Breaking changes** — upstream may introduce breaking changes

### Neutral

- **Replaceable** — can always switch to other loop tools later
- **Independent** — loop files in workspaces can exist without kodehold
- **Compatible** — loop-engineering's files are compatible with other tools

### Error Handling

| Error Scenario | Fallback Behavior |
|----------------|-------------------|
| `npx @cobusgreyling/loop-init` fails | Log error, skip loop setup for workspace, continue with standard workspace creation |
| `opencode run` times out | Log timeout, mark loop as failed in state, retry on next cron cycle |
| `loop-audit` score < 40 | Warn user, do not enable auto-fix patterns |
| Crontab entries conflict | Detect existing entries, skip duplicates, log warning |
| Internet unavailable | Pre-install npm packages, use offline cache if available |

## References

- [loop-engineering](https://github.com/cobusgreyling/loop-engineering) — 9k stars, active development
- [ADR-0048](ADR-0048-mandatory-documentation-review.md) — Mandatory Tool Documentation Review
- [ADR-0049](ADR-0049-lazy-senior-dev-philosophy.md) — The Ladder (dependency justification)
- [ADR-0058](../inactive/ADR-0058-loop-engineering-integration.md) — Archived (failed attempt)
- [ADR-0059](../inactive/ADR-0059-workspace-as-mini-kodehold.md) — Archived (failed attempt)
- [loop-init](https://github.com/cobusgreyling/loop-engineering/tree/main/tools/loop-init) — Scaffold tool
- [loop-audit](https://github.com/cobusgreyling/loop-engineering/tree/main/tools/loop-audit) — Readiness scoring
- [loop-cost](https://github.com/cobusgreyling/loop-engineering/tree/main/tools/loop-cost) — Token estimation
- [loop-sync](https://github.com/cobusgreyling/loop-engineering/tree/main/tools/loop-sync) — Drift detection

## Review Notes

| Date | Reviewer | Action | Changes |
|------|----------|--------|---------|
| 2026-07-22 | Architects | Created | Original draft |
| 2026-07-22 | Reviewers | Requested Changes | Added Documentation section, ADR-0048/049 references, effort estimates, phased rollout, error handling, fixed mixed language |
| 2026-07-22 | Second Opinion | Requested Changes | Added "Key sections read", version pinning strategy, ADR-0058 relationship, CLI integration details, testing strategy, marked crontab as provisional |
| 2026-07-22 | Architects | Accepted | All review findings addressed |
| 2026-07-22 | Engineers | Implemented | Phase 1-3 complete: loop-init integration, enable/disable/run, cron, audit/cost/sync |
