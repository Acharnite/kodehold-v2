---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0003: Git-Native State Machine

## Status

Proposed

## Context

KodeHold v1.0 uses filesystem marker files for lifecycle state:

```
.kodehold-state        ← STATE=ACTIVE, DESIGN_DOC_APPROVED=false, ...
.design_reviewed       ← touch to mark design as reviewed
.testers_done          ← touch to mark tests complete
.code_reviewed         ← touch to mark code reviewed
.second_opinion_done   ← touch to mark cross-model validation complete
.impact_analysis_done  ← touch to mark impact analysis complete
```

This approach has recurring problems:

1. **Fragility**: `git clean -fd` wipes all markers. Workspace state lost.
2. **Merge conflicts**: Markers are git-ignored, so they don't merge. Branch
   switches lose state.
3. **Audit trail**: No history of when a marker was created, by whom, or why.
4. **Race conditions**: Two agents can both `touch .design_reviewed` — no
   conflict detection.
5. **CI blindness**: CI systems don't see ignored files. State is invisible
   in GitHub Actions.
6. **Inventory**: 5+ marker files to track. Easy to forget one.

## Decision

Replace all marker files with git-native state.

### State as Git Tags

| V1.0 Marker | V2.0 Equivalent |
|---|---|
| STATE=INIT | `git tag v2-init` |
| STATE=ACTIVE | `git tag v2-active` |
| STATE=REVIEW | `git tag v2-review` |
| STATE=CLOSED | `git tag v2-closed` |
| STATE=REOPEN | `git tag v2-reopen-N` |
| .design_reviewed | Annotated tag: `git tag -a v2-design-reviewed -m "Reviewer: ..."` |
| .testers_done | Commit with `test:` convention + passing CI |
| .code_reviewed | Commit with `review:` convention |

### Conventional Commits as Audit Trail

```
feat: add regime classifier              ← implementation
test: add classifier test suite          ← tests done
review: approve regime classifier (ADR-0002)  ← code reviewed
chore: bump to v2.0.1                    ← version bump
```

The state is derived from git history, not stored separately:

```
$ kodehold state
State: ACTIVE (tag v2-active, 3 commits ago)
Design: reviewed (tag v2-design-reviewed by @acharnite, 2026-07-23)
Tests: passing (test: commit abc123, CI green)
Code: reviewed (review: commit def456)
```

### Immutable States

Once a state tag is created, it is NEVER deleted. State transitions create NEW
tags:

```
v2-init → v2-active → v2-design-reviewed → v2-review → v2-closed
                                                          ↓
                                                    v2-reopen-1 → v2-active-2
```

This creates an append-only audit log that survives any git operation.

## Consequences

### Positive

- State survives `git clean`, branch switches, and force-pulls
- Full audit trail: `git log --oneline --tags` shows the entire lifecycle
- CI-native: GitHub Actions can read tags natively
- No race conditions: git tag creation is atomic
- No marker files to maintain or forget

### Negative

- Tag namespace pollution: a long-lived project accumulates many tags
- State derivation is slower (needs `git log` parsing) than reading a file
- Requires git knowledge from users (can't just `cat .kodehold-state`)
- Annotated tag messages need structured format for parsing

### New Requirements

- `kodehold state` command to display derived state (hides tag complexity)
- Tag naming convention must be strict (validated in gate)
- CI workflows must be updated to read state from git tags

### Migration

Existing workspaces: `kodehold migrate-v1-state` reads `.kodehold-state` and
creates equivalent git tags. Marker files are archived to
`.git/kodehold/v1-markers/` and git-ignored thereafter.
