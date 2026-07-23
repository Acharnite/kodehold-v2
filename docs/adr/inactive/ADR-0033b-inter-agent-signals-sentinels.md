---
id: ADR-0033b
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0033b: Inter-Agent Signals + Sentinels

## Status

Superseded

> Superseded by ADR-0033 (Crystals + Signals for KodeHold). Signals implemented via agentmemory's native signal system.

## Context

### The Problem

Currently, all inter-agent communication in KodeHold flows through the Director. The communication patterns are:

1. **Director → Team (delegation):** Via Task tool. The Director selects a subagent type and passes a task description.
2. **Team → Director (completion):** Via Task tool output. The team returns results. The Director reads the output and decides next steps.
3. **Team → Team (cross-team coordination):** **Does not exist.** If architects need input from reviewers during design, they must go through the Director: architects → Director (report stuck) → Director → reviewers (delegate).

**Consequences of the Director-centric model:**

- **Director bottleneck.** Every cross-team interaction requires Director mediation. The Director's context window fills with routing metadata instead of high-level decisions.
- **No event-driven workflows.** All workflows are poll-based. The Director must actively check if work is complete. There is no "notify me when X is done" pattern.
- **No parallel agent execution.** Teams cannot signal each other to parallelize work. The Director is the single point of coordination.
- **No exploratory branching.** When investigating a bug, FLS cannot create an ephemeral "what if" branch without Director involvement.
- **No gated events.** There is no mechanism to say "run X when condition Y is met" — everything is manually sequenced.

### Key Forces

1. **Reduce Director bottleneck.** Cross-team signals should not require Director mediation.
2. **Event-driven, not poll-driven.** Agents should be notified when relevant events occur, not forced to poll for status.
3. **Parallel agent execution.** Multiple agents should be able to work simultaneously when their tasks don't conflict.
4. **Safe exploration.** Ephemeral work (investigations, what-if analysis) should be isolated from permanent action chains.
5. **No signal storms.** Without proper controls, agents could flood each other with messages.
6. **Audit trail.** All signals must be traceable for debugging. No undetectable side-channel communication.

### Prior Art

- **ADR-0025** (A2A Protocol — Agent-to-Agent Coordination) — **Deprecated**. Attempted to define a custom A2A protocol for inter-agent communication. It was deprecated because custom protocols are brittle. Agentmemory's `memory_signal` provides a production-grade replacement.
- **ADR-0031** (Actions + Crystals for Director Delegation) — established the action model; signals enable event-driven transitions between actions.
- **ADR-0029** (ICM → Agentmemory Migration Strategy) — Phase 5 of the migration plan; this ADR implements that phase.
- **Actions+Crystals design doc** (`docs/design/actions-crystals-integration.md`, Section 9) — signal/sentinel/sketch integration patterns.
- **ADR-0011** (Team Meeting) — the original manual coordination mechanism; signals can automate some team-meeting patterns (e.g., "review complete" → notify).

## Decision

### Adopt Agentmemory Signals + Sentinels for Inter-Agent Coordination

#### 1. Signals — Asynchronous Inter-Agent Messaging

Use agentmemory's `memory_signal_send` and `memory_signal_read` for direct agent-to-agent communication.

**Signal Types:**

| Type | Purpose | Content Pattern | TTL |
|------|---------|----------------|-----|
| `info` | Status notification | "Design complete for ADR-0032" | 24h |
| `request` | Ask for input | "Review needed: ADR-0032 design" | 72h |
| `response` | Reply to request | "Review approved: ADR-0032" | 72h |
| `alert` | Problem notification | "Test failure: implement-004" | 48h |
| `handoff` | Transfer responsibility | "Taking over issue X" | 24h |

**Routing Rules:**

| From | To | Allowed Signal Types | Director Notified? |
|------|----|---------------------|-------------------|
| Any team | Director | info, alert, handoff | N/A (target) |
| architects | reviewers | request, info | info only |
| engineers | reviewers | request, info | info only |
| engineers | testers | info | No |
| fls | Director | alert, handoff | Yes |
| fls | engineers | request, info | No |
| scribes | Any team | info, request | No |
| reviewers | Any team | response, info | No |
| director | Any team | request, handoff | N/A (source) |

**Key patterns:**

- **Completion notification:** `memory_signal_send(to="reviewers", type="info", content="Design ADR-0033 complete, ready for review")`
- **Request input:** `memory_signal_send(to="architects", type="request", content="ADR-0033: need clarification on signal routing")`
- **Alert:** `memory_signal_send(to="director", type="alert", content="Implement-004 test failure: 3 of 15 tests failed")`

#### 2. Sentinels — Event-Driven Gates

Use agentmemory's `memory_sentinel_create` and `memory_sentinel_trigger` for event-driven workflow progression.

**Sentinel Types:**

| Type | Use Case | Example |
|------|----------|---------|
| `timer` | Time-based unblocking | "Auto-close review after 24h if no response" |
| `approval` | Wait for manual approval | "Block ship until team meeting sign-off" |
| `pattern` | Match content pattern | "When memory_save contains 'CRITICAL', alert Director" |
| `webhook` | External event | "When CI completes, trigger ship gate" |

**Key patterns:**

- **Gate progression:** After design review: `memory_sentinel_trigger(sentinel_id="review-done")` → unblocks `implement` action
- **Timeout escalation:** If a review is pending for 24h: sentinel fires → `memory_signal_send(alert)` to Director
- **Conditional branching:** A sentinel watches for test results. If tests pass → unblock gate. If tests fail → unblock bugfix flow.

#### 3. Sketches — Ephemeral Exploration

Use `memory_sketch_create` and `memory_sketch_promote` for isolated investigative work.

**When to use sketches:**

- **Bug investigation:** FLS creates a sketch to explore root causes. Sketch auto-expires in 1 hour. If root cause found, promote sketch actions to permanent.
- **What-if analysis:** Architects explore a design alternative. Sketch actions are isolated from main action chain.
- **Failed experiments:** If a sketch's conclusions are negative (e.g., "approach X won't work"), the sketch expires cleanly without polluting the action graph.

**Sketch lifecycle:**
1. `memory_sketch_create(title="Investigate memory_leak_in_agentmemory", expiresInMs=3600000)`
2. FLS works within the sketch (actions created within sketch are ephemeral)
3. If root cause found: `memory_sketch_promote(sketch_id)` → actions become permanent
4. If no root cause: sketch auto-expires — no cleanup needed

#### 4. Crystal Consumption — Automated Lesson Extraction

Crystals (from ADR-0031) are consumed by Scribes to extract lessons:

```
1. memory_crystallize(chain_ids) → crystal
2. Scribes reads crystal content
3. Scribes calls memory_lesson_save(content=crystal.narrative, ...)
4. Scribes updates design doc if crystal contains design decisions
```

#### 5. ADR-0025 Revival

ADR-0025 (A2A Protocol) was deprecated because it defined a custom inter-agent protocol. Agentmemory's `memory_signal` provides the same capability with production-grade infrastructure. This ADR effectively revives the intent of ADR-0025 using agentmemory primitives.

- **ADR-0025 status:** Remains Deprecated. This ADR does not revive the custom A2A protocol. Instead, it uses agentmemory's built-in signal mechanism.
- **Cross-reference:** Update ADR-0025 to note that its intent is fulfilled by ADR-0033's agentmemory signal approach.

### What This Changes

- **Director agent file:** Add signal routing logic. Director can read signals and act on them (escalate alerts, process handoffs).
- **Scribes agent file:** Add crystal consumption workflow — extract lessons from crystals.
- **ADR-0025:** Update status to include "Intent fulfilled by ADR-0033" note.
- **Agent definitions (architects, engineers, reviewers, testers, fls):** Allow `memory_signal_send` and `memory_signal_read` in their tool permissions. No behavioral change to core workflows (signals are additive).
- **Agent definitions (fls):** Add `memory_sketch_create` and `memory_sketch_promote` for investigative work.

## Consequences

### Positive

1. **Reduced Director bottleneck.** Cross-team signals bypass the Director. Reviewers can directly respond to architect requests.
2. **Event-driven workflows.** Sentinels enable "when X completes, start Y" without Director polling. Reduces Director's context window usage.
3. **Parallel agent execution.** Multiple teams can work concurrently when their actions are independent, coordinated via signals.
4. **Safe exploration.** Sketches provide isolated, auto-expiring workspaces for investigations. Failed experiments don't pollute the action graph.
5. **Automated lesson extraction.** Crystals feed directly into agentmemory's lesson system. Scribes don't need to manually summarize every completed flow.
6. **ADR-0025 intent fulfilled.** The A2A protocol's goal (inter-agent coordination) is achieved with production-grade infrastructure instead of a custom implementation.

### Negative

1. **Signal storms.** Without controls, agents could flood each other. Mitigation: signal routing rules (only specific signal types per agent pair), TTL limits on all signals, Director monitoring of signal volume.
2. **Debugging complexity.** Signal chains (A signals B, B signals C, C creates action) are harder to trace than linear Director-mediated flows. Mitigation: all signals have audit trail via agentmemory; use `memory_recall` to reconstruct signal chains.
3. **Split coordination responsibility.** Some coordination happens via signals (direct), some via the Director (mediated). Teams must know which pattern to use when. Mitigation: clear routing table in agent definitions.
4. **Sentinels add statefulness.** Timer-based sentinels (e.g., "auto-escalate after 24h") introduce time-dependent behavior. Mitigation: sentinel TTLs are generous; sentinels are visible via `memory_sentinel_list`.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Signal storms** — agent loops (A signals B, B signals A infinitely) | Low | High | Signal routing table restricts who can signal whom. Director monitors signal volume. |
| 2 | **Sentinel reliability** — timer sentinels drift or fail to fire | Low | Medium | Sentinels are advisory, not critical-path. Director polls if no response within expected window. |
| 3 | **Sketch data loss** — promote fails and ephemeral work is lost | Low | Medium | Teams should copy critical findings to permanent storage before sketch expires. |
| 4 | **Signal overload** — agents overwhelmed by incoming signals | Low | Low | Agents read signals at delegation boundaries, not continuously. Signal TTL prevents stale queue buildup. |
| 5 | **ADR-0025 confusion** — team references old A2A protocol instead of signals | Low | Low | ADR-0025 updated with cross-reference to this ADR. Deprecated status is clear. |

### Follow-up Items

- [ ] Update `.opencode/agents/director.md` — add signal reading/routing logic
- [ ] Update `.opencode/agents/scribes.md` — add crystal consumption workflow
- [ ] Update `.opencode/agents/fls.md` — add `memory_sketch_create`/`memory_sketch_promote` permissions
- [ ] Update remaining agent definitions — allow `memory_signal_send`/`memory_signal_read` tools
- [ ] Update ADR-0025 status to include "Intent fulfilled by ADR-0033"
- [ ] Define initial sentinel configurations (review timeout, gate progression)
- [ ] Document signal routing table in agent definitions and/or kodehold-protocol.md

### How to Revert

1. **Stop signal usage.** Remove `memory_signal_send`/`memory_signal_read` from agent tool permissions.
2. **Stop sentinel usage.** Remove `memory_sentinel_create`/`memory_sentinel_trigger` from agent definitions.
3. **Remove sketch permissions.** Remove `memory_sketch_create`/`memory_sketch_promote` from agent definitions.
4. **Revert to Director-mediated communication.** All cross-team coordination flows through the Director as before.
5. **This ADR becomes Deprecated.** ADR-0025 remains Deprecated (its intent was fulfilled but the mechanism is no longer needed).

## ADR References

- **ADR-0025** (A2A Protocol — Agent-to-Agent Coordination) — **Intent fulfilled by this ADR.** The custom A2A protocol is replaced by agentmemory's `memory_signal` mechanism.
- **ADR-0031** (Actions + Crystals for Director Delegation) — establishes the action and crystal model; signals enable event-driven transitions between actions; crystals feed into lesson extraction described here.
- **ADR-0032** (Routine Templates) — templates can trigger signals on completion; signals can trigger template instantiation.
- **ADR-0029** (ICM → Agentmemory Migration Strategy) — Phase 5 of the migration plan; this ADR implements that phase.
- **ADR-0011** (Team Meeting) — some team-meeting coordination patterns can be automated via signals.
- **Actions+Crystals design doc** (`docs/design/actions-crystals-integration.md`, Section 9) — signal/sentinel/sketch integration patterns.

### Source Files Referenced

- `docs/adr/ADR-0025-a2a-protocol.md` — A2A protocol to be cross-referenced as fulfilled by signals
- `.opencode/agents/director.md` — agent definition to be extended with signal routing
- `.opencode/agents/scribes.md` — agent definition to be extended with crystal consumption
- `.opencode/agents/fls.md` — agent definition to be extended with sketch permissions
- All `.opencode/agents/*.md` files — signal tool permissions to be added
- `docs/design/actions-crystals-integration.md` — Section 9: Signal/Sentinel/Sketch/Checkpoint Integration
