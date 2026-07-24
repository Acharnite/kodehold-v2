---
phase:
  current: 6
  total: 6
  status:
    6: proposed
---

# ADR-0010: Loop Daemon for Continuous Workspace Maintenance

## Status

Proposed

## Context

KodeHold v2.0 is a specification compiler and code generation toolchain that
processes individual feature requests through a deterministic pipeline. However,
real-world software development is continuous — features arrive over time,
dependencies need updating, and the workspace requires ongoing maintenance.

We need a mechanism for:

1. **Continuous maintenance** — Workspaces should stay healthy without manual
   intervention (dependency updates, verification, garbage collection).
2. **Feature request processing** — Features described as issues or YAML files
   should be automatically implemented, verified, and merged.
3. **Isolation** — Multiple features in flight should not interfere with each
   other or with the main workspace.
4. **Git-native versioning** — All work must be tracked in git branches with
   clean history, using the same state machine as the core system.

### Constraints

- Must use git worktrees for isolation, not marker files or side channels
- Must integrate with the existing pipeline (parse → analyze → generate → verify → optimize)
- Must follow the existing git-native state pattern (tags, conventional commits)
- Must work in both daemon mode (continuous) and single-shot mode (CI)
- Must handle GitHub webhook events (issues opened → feature requests)

## Decision

We will implement a **loop daemon** as a new subsystem within the KodeHold
binary, with the following architecture:

### Loop Engine

The `LoopEngine` orchestrates the feature lifecycle across six phases per cycle:

```
Cycle:
  1. Discover pending feature requests from .kodehold/features/
  2. Start work on auto-processable features (create worktree + branch)
  3. Verify in-progress features by running pipeline in worktrees
  4. Merge completed features (if auto-merge is enabled)
  5. Run maintenance (git gc --auto, verify main)
  6. Prune abandoned worktrees
```

### Git Worktree Isolation

Each feature request gets:

- A branch `feature/<name>` forked from `main`
- A linked git worktree at `.kodehold/worktrees/<name>/`
- An isolated pipeline run inside the worktree

This is architecturally superior to in-repo branches because:

- The main workspace is never switched away from `main`
- CI/CD can run independently per feature
- Failed features are trivially discarded (remove worktree + delete branch)
- Multiple features can be developed concurrently without context switching

### Feature Request System

Feature requests are YAML files in `.kodehold/features/`:

```yaml
name: add-auth
title: Add JWT authentication
description: |
  Implement JWT-based authentication.
acceptance:
  - Tokens expire after 1 hour
source: Cli       # or GitHubIssue, Local, Scheduled, Cli
status: Pending   # Pending → InProgress → ReadyForReview → Approved → Merged
labels: [enhancement]
priority: P1      # P0=critical, P1=high, P2=normal, P3=low
```

### Merge Strategies

Three strategies for integrating completed features:

| Strategy | Command | When |
|----------|---------|------|
| merge | `git merge --no-ff` | Preserve branch history (default) |
| rebase | `git rebase main; git merge --ff-only` | Clean linear history |
| squash | `git merge --squash` | Single commit per feature |

### CLI Interface

```
kodehold loop start [--once] [--interval <sec>]   Start loop daemon
kodehold loop feature <name> --title <title>       Create feature request
kodehold loop list                                 List features + worktrees
kodehold loop maintain                             Run maintenance only
kodehold loop prune                                Clean up worktrees
```

### Event System Integration

The existing event dispatcher is extended to handle:

- `issues` (GitHub webhook) — Creates feature requests from opened issues
- `schedule` — Triggers a full loop cycle
- `POST /hooks/loop` — Manual loop cycle trigger

### Configuration

```yaml
# kodehold.yaml
loops:
  features_dir: .kodehold/features/
  worktrees_dir: .kodehold/worktrees/
  auto_maintain: true
  maintain_interval_secs: 3600
  merge_strategy: merge
  auto_merge: false
  max_concurrent_features: 3
  auto_labels: [auto]
```

## Consequences

### Positive

1. **Zero-touch maintenance** — Workspaces stay healthy automatically via
   scheduled verify + gc cycles.
2. **Feature pipeline** — Features flow from request to merged code without
   manual branch management.
3. **Isolation guarantees** — Git worktrees ensure no cross-feature
   interference and clean main workspace state.
4. **Deterministic lifecycle** — Every feature follows the same state machine,
   making it auditable and predictable.
5. **Event-driven** — GitHub issues automatically become feature requests,
   reducing friction.

### Negative

1. **Git worktree complexity** — Worktrees add git operational overhead and
   require careful cleanup. The `prune` command mitigates this.
2. **Disk usage** — Each worktree duplicates the repository. Mitigated by
   git's hard-link support for objects and the `max_concurrent_features` limit.
3. **Merge conflicts** — Auto-merge may fail on conflicting features. Users
   must manually resolve before merging.

### Risks

- **Worktree leak** — If the daemon crashes mid-operation, worktrees may be
   left behind. The `prune` command and automatic startup cleanup handle this.
- **Branch name collisions** — Feature names must be unique and git-safe.
   The system lowercases and replaces spaces with hyphens.
- **`main` branch assumption** — The system assumes the default branch is
   `main`. Projects using `master` need configuration.

## Implementation

The implementation spans:

1. **`src/loops/mod.rs`** — `LoopEngine` orchestrator with cycle logic
2. **`src/loops/feature.rs`** — `FeatureStore` for YAML feature request management
3. **`src/loops/worktree.rs`** — `WorktreeManager` for git worktree operations
4. **`src/cmd/loops.rs`** — CLI command handler for `kodehold loop`
5. **`src/events/mod.rs`** — Extended dispatcher with `LoopCycle` and
   `CreateFeatureFromIssue` actions
6. **`src/config.rs`** — Extended `Config` with `loops: LoopConfig` section
7. **`tests/integration.rs`** — Integration tests for loop CLI commands

## References

- ADR-0003: Git-native state machine (tag lifecycle pattern)
- ADR-0007: Event-driven architecture (webhook + watcher routing)
- ADR-0008: Single-binary Rust (all-in-one binary design)
