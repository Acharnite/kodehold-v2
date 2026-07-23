---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0031: Actions + Crystals for Director Delegation

## Status

Deprecated

**Phase:** Phase 3 (Frontier-Driven Delegation) — replaces the Director's manual `todowrite` sequence protocol with agentmemory's action orchestration layer.

## Context

### The Problem

The KodeHold Director currently manages multi-team workflows through three ad-hoc mechanisms:

1. **Manual `todowrite` lists** — sequential todo markers with no dependency enforcement, priority ordering, or status tracking. Director maintains entire sequence in working memory.
2. **Flat `.kodehold-state` file** — tracks lifecycle state (`INIT`, `ACTIVE`, `REVIEW`, `CLOSED`, `REOPEN`). No dependency graph, parallel execution tracking, or automated state validation.
3. **Ad-hoc ICM checkpoint stores** — unstructured memory writes with no relationship to task graph.

**Consequences:**
- No dependency graph enforcement. Downstream delegation could start before upstream finishes.
- No parallel execution awareness. Cannot easily determine which actions can run in parallel.
- No lease mechanism. Two concurrent sessions could delegate same task.
- No automated lesson extraction. Completed work manually summarized.
- No frontier. Director manually scans pending items.

### Key Forces

1. **Backward compatibility.** Phase 1 dual-writes actions alongside todowrite. Director can fall back.
2. **Opt-in adoption.** Director chooses when to start reading `memory_frontier`.
3. **Gradual migration.** Phase 1: action creation (fire-and-forget). Phase 3: frontier primary. Phase 4: routine templates.
4. **Lease safety.** No two agents on same action simultaneously.
5. **Crystal value.** Completed action chains automatically compressed.
6. **Graceful degradation.** If agentmemory unavailable, fall back to todowrite.

### Prior Art

- **ADR-0029** (ICM → Agentmemory Migration Strategy) — established 6-phase migration; this ADR implements Phase 3
- **Actions+Crystals design doc** (`docs/design/actions-crystals-integration.md`) — action types, dependency model, frontier flow, crystal strategy (Sections 4-7)
- **ADR-0004** (ICM and RTK Integration) — to be deprecated; replaced by action-based delegation
- **ADR-0009** (ICM MCP Integration) — to be deprecated; agentmemory's MCP tools replace ICM's
- **ADR-0021** (Prospective Memory) — superseded; task queues migrate to actions
- **ADR-0015** (Director Delegation Enforcement) — extended with lease-based enforcement

## Decision

### Adopt Agentmemory Actions + Crystals as the Primary Delegation Workflow

#### Action Types

10 types covering all KodeHold work:

| Type | Priority | Default Team | Used For |
|------|----------|-------------|----------|
| `design` | 8 | architects | ADRs, design docs, architecture decisions |
| `review` | 7 | reviewers | Design/code review |
| `implement` | 8 | engineers | Feature implementation |
| `test` | 6 | testers | Test writing + execution |
| `gate-validation` | 9 | reviewers | Gate.sh --validate-only |
| `gate-execution` | 9 | director | Gate.sh --transition |
| `document` | 5 | scribes | Documentation updates |
| `triage` | 7 | fls | Bug investigation |
| `ship` | 9 | director | Shipping gate |
| `second-opinion` | 7 | second-opinion | Cross-model validation |

Priorities: 6-9 range. Higher = more urgent. Gate and ship (9) first. Documentation (5) last.

#### Dependency Model

Actions use `requires` to define a DAG:

```
memory_action_create(type="design", requires=[])           # No dependencies — can start immediately
memory_action_create(type="review", requires=["design-001"])  # Blocks until design-001 is done
memory_action_create(type="implement", requires=["review-001"])  # Blocks until review approves
```

Rules:
- Actions with empty `requires` are **unblocked** and returned by `memory_frontier`
- Actions with unsatisfied `requires` are **blocked** and hidden from `memory_frontier`
- Multiple actions can require same action (fan-out)
- Action can require multiple actions (fan-in)

#### Frontier Flow (Director Delegation Loop)

Replace `todowrite` with action frontier:

```
# Current (todowrite):
1. Director maps task sequence in working memory
2. Director creates todowrite with pending/completed markers
3. Director delegates to team
4. Director updates todowrite status
5. Director stores checkpoint in ICM

# New (actions + frontier):
1. Director creates actions with dependency chain
   memory_action_create(type="design", requires=[], priority=8)
   memory_action_create(type="review", requires=["design-001"], priority=7)
   
2. Director reads frontier for next unblocked action
   memory_frontier → returns action with highest priority + no blockers
   
3. Director acquires lease on action
   memory_lease(action_id, agentId="director") → exclusive lock
   
4. Director delegates to team via Task tool
   
5. Team completes work (uses agentmemory for context + storage)
   
6. Director updates action with result
   memory_action_update(action_id, status="done", result="summary")
   
7. Director crystallizes completed chain (Phase 5)
   memory_crystallize(chain_ids) → auto-digest
   
8. Director releases lease
   memory_lease(action_id, agentId="director", operation="release")
```

#### Lease Management

Prevents double-delegation:
- `memory_lease(action_id, agentId)` acquires exclusive lock
- Lease TTL: default 10 min, max 1 hour
- Expired lease re-exposes action
- On completion: `memory_lease(action_id, ..., operation="release")`
- On failure: lease expiry re-exposes action

#### Crystal Strategy

| Trigger | Scope | Timing |
|---------|-------|--------|
| Per-flow completion | All actions in flow template | After last action `done` |
| Per-gate | Actions in lifecycle transition | After gate.sh passes |
| Explicit | Single action or chain | When `crystallize: true` set |

Each crystal contains: narrative summary, key outcomes, decisions, files affected, lessons learned, action chain metadata.

### What This Changes

- **Director delegation loop:** From `todowrite` → `memory_frontier` + `memory_lease`
- **Director agent file:** Replace "Todo Sequence Protocol" with "Action Frontier Protocol"
- **Scribes post-task:** Add `memory_action_update` and `memory_crystallize` steps
- **ADR-0004:** Mark as Deprecated
- **ADR-0009:** Mark as Deprecated
- **ADR-0021:** Mark as Superseded
- **ADR-0019:** Mark as Superseded
- **Benchmark scripts:** Rewrite for agentmemory or deprecate
- **Consolidate scripts:** Deprecate

### Migration Path Within This ADR

| Sub-Phase | What Happens | Behavior |
|-----------|-------------|----------|
| 3.0 | Actions created alongside todowrite | No behavioral change |
| 3.1 | Director reads `memory_frontier` for awareness but follows todowrite | Monitoring only |
| 3.2 | Director uses frontier as primary driver, falls back to todowrite if frontier empty | Opt-in |
| 3.3 | Todowrite removed; only `memory_frontier` used | Full migration |

## Consequences

### Positive

1. **Dependency graph enforcement.** `requires` chains ensure correct order.
2. **Frontier-driven prioritization.** `memory_frontier` returns highest-priority unblocked action.
3. **Lease-based delegation safety.** No double-delegation; leases expire and re-expose stuck actions.
4. **Automated lesson extraction.** `memory_crystallize` generates compact digests.
5. **Parallel execution awareness.** Unblocked independent actions can run in parallel.
6. **Graceful degradation.** Agentmemory unavailable → fall back to todowrite.
7. **Reduced cognitive load.** Frontier handles ordering.

### Negative

1. **Agentmemory coupling increases.** Core delegation depends on agentmemory; without it, degraded manual mode.
2. **Learning curve.** New tools: `memory_action_create/update`, `memory_frontier`, `memory_lease`, `memory_crystallize`.
3. **Performance overhead.** 3-4 API calls per delegation instead of one todowrite.
4. **Lease expiry edge cases.** Lease expires while working; another agent could claim. Mitigation: generous TTL (30 min default).
5. **Crystal storage.** Crystals accumulate. Mitigation: compact; periodic consolidation.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Frontier returns unexpected results** — blocks when it shouldn't or unblocks when it shouldn't | Medium | Critical | Phase 3.0-3.1 dual-writes actions + todowrite. Director has manual fallback. |
| 2 | **Lease expires during long-running task** — agent reclaims action mid-work | Low | High | Set lease TTL per action type (implement: 60min, review: 30min). Director can renew. |
| 3 | **Action chain corruption** — `requires` references invalid action ID | Low | Medium | Use action types + sequence IDs; validate before create. |
| 4 | **Crystalization fails** — `memory_crystallize` returns unexpected or empty digest | Low | Low | Crystals are additive; original actions remain accessible. |
| 5 | **Director too reliant on frontier** — loses situational awareness | Medium | Medium | Director runs `memory_action_list` periodically. |

### Follow-up Items

- [ ] Update `.opencode/agents/director.md` — replace "Todo Sequence Protocol" with "Action Frontier Protocol"
- [ ] Update `.opencode/agents/scribes.md` — add action management and crystal consumption
- [ ] Mark ADR-0004 as Deprecated
- [ ] Mark ADR-0009 as Deprecated
- [ ] Mark ADR-0021 as Superseded
- [ ] Mark ADR-0019 as Superseded
- [ ] Deprecate `scripts/benchmark.sh` or rewrite for agentmemory
- [ ] Deprecate `scripts/consolidate-all.sh`
- [ ] Update design doc Section 5 to reference Actions-based workflow

### How to Revert

1. **Soft revert:** Stop reading `memory_frontier`. Fall back to todowrite-based delegation (todowrite existed during dual-write).
2. **Hard revert:** `git restore` director.md and scribes.md to pre-action state. Existing actions orphaned but harmless.
3. **ADR revert:** Reactivate ADR-0004, ADR-0009, ADR-0021. Deprecate this ADR.

## ADR References

- **ADR-0004** — **Deprecated**. ICM-based delegation replaced by agentmemory actions.
- **ADR-0009** — **Deprecated**. ICM MCP tools replaced by agentmemory MCP tools.
- **ADR-0019** — **Superseded**. Crystals replace ICM-based compression.
- **ADR-0021** — **Superseded**. Task queues become actions.
- **ADR-0029** — Phase 3 of migration; this ADR implements that phase.
- **ADR-0015** — Extended by this ADR; leases add enforcement beyond tool permissions.
- **ADR-0030** — Prerequisite; agentmemory must be primary memory system.
- **ADR-0032** — Builds on this ADR's action model; routines are parameterized action DAGs.
- **ADR-0033** — Builds on this ADR; signals activate between actions.
- **Actions+Crystals design doc** (`docs/design/actions-crystals-integration.md`) — Sections 4-7 define action schema, dependency model, frontier flow, crystal strategy.

### Source Files Referenced

- `.opencode/agents/director.md` — contains "Todo Sequence Protocol" section to be replaced
- `.opencode/agents/scribes.md` — post-task workflows to be extended with action management
- `docs/design/actions-crystals-integration.md` — action schema, types, priorities, dependency model (Sections 4-7)
- `docs/adr/ADR-0004-icm-rtk-integration.md` — to be deprecated
- `docs/adr/ADR-0009-icm-mcp-integration.md` — to be deprecated
- `docs/adr/ADR-0019-session-context-compression.md` — to be superseded
- `docs/adr/ADR-0021-prospective-memory.md` — to be superseded
- `scripts/benchmark.sh`, `scripts/consolidate-all.sh` — to be deprecated or rewritten