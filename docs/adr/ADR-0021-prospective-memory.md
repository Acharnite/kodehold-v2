---
status: Accepted
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0021: Prospective Memory (Task Queue & Scheduler)

## Status

**Reactivated — Accepted (2026-07-15)**

Originally superseded by ADR-0031 (agentmemory Actions + Crystals). With agentmemory removed per ADR-0050 chain, the infrastructure described in this ADR (`.opencode/memory/prospective/`) was never replaced and remains in active use as of 2026-07-15.

The prospective task system now operates via structured markdown files in `.opencode/memory/prospective/`, with CRUD lifecycle managed by Scribes per the Scribes agent definition.

## Context

KodeHold has no automated task scheduling or deferred action system. The `TODO.md` file is manually maintained and only reflects current intentions — not deferred actions, scheduled checks, or recurring tasks. When a session ends, any "I should check X next time" or "remind me about Y in 3 days" is lost unless manually recorded.

The current approach has these limitations:

- No way to schedule deferred actions ("check this in 2 days")
- No recurring task support ("run smoke tests every morning")
- No trigger-based execution ("when PR is merged, update changelog")
- TODO.md is a flat list with no scheduling semantics
- Session restart requires manual context reconstruction about "what was I going to do?"
- The session context compression (ADR-0019) preserves past context but not future intentions

The key forces are:

- AI agents have no inherent sense of time — they need explicit scheduling
- ICM already stores memories with timestamps — natural fit for deferred actions
- The session start protocol (ADR-0019 wake-up integration) provides a natural check point
- Deferred actions must survive session boundaries (ICM provides persistence)
- Too many scheduled tasks would bloat context — must be selective

## Decision

Add a prospective memory layer with task queue, scheduler/trigger engine, and deferred actions stored in ICM.

### Task Queue Structure

Each prospective memory is stored in ICM with structured fields:

```
id: <short-uuid>
type: deferred|recurring|trigger
action: <what to do>
trigger_condition: <when to execute>
created_at: <timestamp>
execute_after: <timestamp>  (for deferred)
recurring_interval: <duration>  (for recurring)
trigger_event: <event pattern>  (for trigger)
priority: critical|high|medium|low
context: <additional context needed>
status: pending|completed|expired
```

### Task Types

| Type | Description | Example |
|------|-------------|---------|
| **Deferred** | Execute after a time delay | "Check if PR review is done in 2 days" |
| **Recurring** | Execute on a regular schedule | "Run smoke tests every morning" |
| **Trigger** | Execute when an event occurs | "When PR is merged, update changelog" |

### Scheduler Integration

At session start, after `icm_wake_up` and session summary loading:

1. Query ICM for pending tasks with `execute_after <= now`
2. Query ICM for triggered tasks matching current context
3. Present due tasks to Director for execution decision
4. Director prioritizes and delegates to appropriate teams

### Trigger Engine

| Trigger Event | Pattern Match | Action |
|---------------|--------------|--------|
| State transition | `gate.sh --transition` output | Check for transition-dependent tasks |
| PR merged | `gh pr list --state merged` | Update changelog, run tests |
| File modified | `git diff --name-only` | Check for dependent tasks |
| Session start | `icm_wake_up` | Load all due deferred/recurring tasks |
| Time-based | `execute_after <= now()` | Execute deferred tasks |

### Task Lifecycle

```
Created → Pending → [Triggered] → Executing → Completed
                                          ↓
                                       Expired (if past deadline)
```

### Storage in ICM

Prospective memories are stored in ICM with topic prefix:

```
Topic: kodehold-<project>-prospective
Tags: ["prospective", "task-type:<deferred|recurring|trigger>"]
Importance: high (for critical tasks), medium (for others)
```

### Token Budget

| Category | Max Tasks | Rationale |
|----------|-----------|-----------|
| Critical | 5 | Must execute — blocking issues |
| High | 10 | Important but not blocking |
| Medium | 15 | Nice-to-have deferred actions |
| Low | 5 | Recurring maintenance, informational |

Total prospective memory budget: ~35 tasks max. Scribes consolidates or expires stale tasks.

### Implementation Plan

| File | Change |
|------|--------|
| scribes.md | Add prospective memory CRUD, task lifecycle management |
| director.md | Add session-start task check, trigger event monitoring |
| design doc | Add section 7.7 — Prospective Memory |

## Consequences

- Positive: Tasks survive session boundaries — no more "I should remember to check X"
- Positive: Recurring tasks reduce manual maintenance overhead
- Positive: Zero new infrastructure — leverages existing ICM memory store
- Negative: Adds state management overhead — task lifecycle must be maintained
- Negative: Token budget for prospective tasks competes with other context needs
- Negative: Expired/stale tasks must be cleaned up to prevent memory bloat
- Neutral: Task priority levels may need tuning based on actual workflow patterns
- Deferred: Trigger-based execution (event monitoring) deferred to future iteration — AI agents lack reliable time/event sensing

## Implementation Plan (v1)

| File | Change |
|------|--------|
| `docs/design/README.md` | Add section 7.7 — Prospective Memory |
| `docs/adr/ADR-0021-prospective-memory.md` | Status → Accepted, add implementation plan |
| `.opencode/agents/director.md` | Add step 1.5 to Session Lifecycle: prospective task check |
| `.opencode/agents/scribes.md` | Add Prospective Memory CRUD section with ICM operations |
| `TODO.md` | Add "Prospective Tasks" summary line |

### No new files, scripts, or skills required.
