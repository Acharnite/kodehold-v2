# ADR-0059: Workspace as Self-Contained Loop-Ready Mini KodeHold

## Status

**Accepted** — 2026-07-22

## Context

KodeHold manages multiple projects as "workspaces" under `workspaces/<name>/`.
The current system has several structural problems:

1. **Symlinks break.** `workspace.py adopt` creates a symlink from `workspaces/<name>/`
   to the real project directory. If the target is moved or deleted, the symlink
   becomes a dangling pointer.

2. **No loop infrastructure.** Workspaces don't get loop files (LOOP.md, STATE.md,
   loop-budget.md, loop-constraints.md) on creation — they must be added manually.

3. **Registry format mismatch.** `workspaces/.catalog` is JSON in a project that
   standardized on YAML per ADR-0037. It lives inside `workspaces/` rather than
   `config/`. It does not track loop status, last run, or state.

4. **No workspace-scoped loop execution.** There is no `workspace.py loop <name>`
   command. Running a loop against a workspace requires manual setup.

5. **No per-workspace constraints or budget.** Loop budgets and constraints are
   critical for Loop Engineering (ADR-0058) but no workspace has them by default.

## Decision

### 1. Replace Symlinks with Copy (Default)

`workspace.py adopt <name> <path>` now **copies** the source project into
`workspaces/<name>/` by default. The symlink approach is preserved as
`workspace.py adopt --link <name> <path>` for backwards compatibility.

**Rationale:** Symlinks are brittle. Copying makes each workspace truly
self-contained — its files, state, loops, and git all live together under
`workspaces/<name>/`.

### 2. Registry Migration: `.catalog` → `config/workspaces.yaml`

The catalog moves from `workspaces/.catalog` (JSON) to `config/workspaces.yaml`
(YAML format per ADR-0037).

### 3. Loop Scaffolding on Init and Adopt

Every `workspace.py init` and `workspace.py adopt` creates four loop files
in the workspace root: LOOP.md, STATE.md, loop-budget.md, loop-constraints.md.

### 4. New `workspace.py loop` Subcommand

`workspace.py loop <name> <pattern>` runs a loop pattern against a workspace:
validates workspace, checks constraints, calls scripts/loop-run.sh, logs output.

### 5. Migration Command

`workspace.py migrate <name>` adds loop scaffolding to existing workspaces.
`workspace.py migrate --all` applies to all.

### 6. Symlink Behavior for `--link`

When `adopt --link` is used: symlink preserved, loop scaffolding created inside
symlink target, `.kodehold-loop-state` marker in `workspaces/<name>/`.

### 7. State File Updates

`.kodehold-state` gains `LOOP_READY=false` field.

## Consequences

### Positive
- No broken symlinks. Copy mode makes every workspace self-contained.
- Every workspace is loop-ready. Loop files ship with init/adopt.
- YAML consistency. `config/workspaces.yaml` matches ADR-0037 format.
- One-command migration. `workspace.py migrate --all` catches up existing workspaces.

### Negative
- Adopted projects double storage. Mitigation: `--link` flag.
- Migration debt. Six existing workspaces need migration.
- Break in convention. Existing users who expect symlink need `--link`.

## ADR References
- Supersedes (partial): ADR-0012 §"Mechanism: Symlink + Sidecar Artifacts"
- Related: ADR-0037 (YAML Configuration), ADR-0046 (Git Init), ADR-0058 (Loop Engineering)
