---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0033: Crystals + Signals for KodeHold

## Status

Deprecated

**Phase:** Phase 5 (final) — Auto-crystallize triggers + inter-agent signaling.

## Context

Phase 3 introduced `memory_crystallize` as a manual step. Phase 4 standardized workflows via templates. Two gaps remain:

1. **Crystals are never auto-triggered** — Director must remember to call crystallize manually
2. **Agents cannot signal each other** — all communication flows through Director via Task tool, creating a bottleneck

## Decision

### 1. Auto-Crystallize — Four Triggers

Crystals compress completed action chains into LLM-digested summaries.

| Trigger | Condition | Scope |
|---------|-----------|-------|
| **Threshold** | Every 5 completed actions per project | The 5 most recent completed actions |
| **Gate transition** | Before executing a state transition | All actions in current phase |
| **Routine completion** | Last action of a `memory_routine_run` template | Entire routine chain |
| **Explicit** | User or Director says "crystallize" | Specified actions |

**Precedence:** Explicit > Gate > Routine > Threshold. Manual `memory_crystallize` still works.

### 2. Crystal Content

Each crystal is a memory entry with:
- `narrative` — LLM-generated summary of what was done
- `outcomes` — key results
- `decisions` — decisions made and rationale
- `files_affected` — files touched
- `lessons` — auto-extracted lessons
- `chain` — action metadata (count, types, teams, duration)
- `trigger` — which trigger created it

### 3. Inter-Agent Signaling — Five Signal Types

| Type | Purpose | Example |
|------|---------|---------|
| `info` | Informational | "Design review completed" |
| `request` | Ask for action | "Please review ADR-0033" |
| `response` | Reply to signal | "Code review: 2 issues found" |
| `alert` | Urgent notification | "Tests failing on main branch" |
| `handoff` | Transfer work | "FLS triage done, handing off to Engineers" |

### 4. Signal Flow

**Handoff pattern:**
```
1. Director sends: memory_signal_send(to="reviewers", type="handoff", content="Ready for review")
2. Team reads: memory_signal_read(agentId="reviewers", unreadOnly="true")
3. Team responds: memory_signal_send(to="director", type="response", content="APPROVED")
4. Director reads at session start: memory_signal_read(agentId="director", unreadOnly="true")
```

**Signals vs. Actions:** Actions track work (what needs to be done). Signals track communication (who needs to know what). They complement each other.

### 5. Signal Routing Rules

| From → To | Allowed Types |
|-----------|--------------|
| Any → Director | info, alert, handoff |
| architects → reviewers | request, info |
| engineers → reviewers | request, info |
| engineers → testers | info |
| fls → Director | alert, handoff |
| scribes → Any | info, request |
| director → Any | request, handoff, alert |

## Consequences

### Positive
1. **No manual crystallize** — four automatic triggers cover all cases
2. **Timely lessons** — crystals created minutes after completion
3. **Reduced Director bottleneck** — cross-team signals bypass Director for simple handoffs
4. **Event-driven workflows** — "when X completes, notify Y" without polling
5. **Natural chapter boundaries** — gate-triggered crystals create per-phase digests

### Negative
1. **Crystal storage growth** — mitigated by <1KB typical size and auto-consolidation
2. **Signal storms** — mitigated by TTL limits and reading only at delegation boundaries
3. **Debugging complexity** — signal chains harder to trace; mitigated by `replyTo` threading
4. **Learning curve** — teams must learn 4 trigger conditions and 5 signal types

### Follow-up
- [ ] Add auto-crystallize triggers to director.md
- [ ] Add signal patterns to director.md
- [ ] Add signal handling to scribes.md
- [ ] All agents: add `memory_signal_send`/`memory_signal_read` to permissions

## ADR References
- **ADR-0031** (Actions + Crystals) — established the crystal model
- **ADR-0032** (Routine Templates) — routine completion is a crystal trigger
- **ADR-0029** (Migration Strategy) — Phase 5 of the migration plan
- **ADR-0025** (A2A Protocol) — intent fulfilled by this ADR's signal mechanism
