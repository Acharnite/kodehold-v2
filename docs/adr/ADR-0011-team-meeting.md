---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0011: Team Meeting — Collective Project Review

## Status

Accepted

## Context

KodeHold's shipping gate includes a "Director final approval" step where the single Director agent compares the design doc with the final product and approves or rejects before shipment. This creates a bottleneck and a single point of failure:

- The Director alone decides if the project matches the design, despite not having written the code or tests
- There is no forum for teams to present their work, raise concerns, or flag design drift
- Knowledge silos form — each team knows only its own slice of the project
- The Director may miss subtle misalignments that a specialist team would catch

The key forces are:
- Collective intelligence beats individual judgment for complex quality assessments
- Teams need a structured forum to present and defend their work
- The review must remain lightweight enough not to stall the shipping gate
- All teams must participate, not just those involved in implementation

## Decision

Replace the Director-only final approval with a **Team Meeting** — a structured synchronous review where all six teams (Architects, Engineers, Testers, Reviewers, Scribes, FLS) present their findings to each other and the Director facilitates.

### Meeting Structure

The Team Meeting runs once per project phase transition, but is **mandatory** before the `REVIEW → CLOSED` shipping gate:

```
Director  (facilitator)
├── Architects    — "Does the implementation match the design doc?"
├── Engineers     — "What was built, what changed during implementation?"
├── Testers       — "Test coverage, regressions, edge cases?"
├── Reviewers     — "Code quality, standards compliance, concerns?"
├── Scribes       — "ICM memories stored, documentation complete?"
└── FLS           — "Support readiness — do we know the project?"
```

### Protocol

1. **Director** opens the meeting, states the project and phase
2. **Each team** presents in order (Architects → Engineers → Testers → Reviewers → Scribes → FLS):
   - What they delivered
   - What they observed (deviations from design, quality concerns, risks)
   - Whether they approve the transition
3. **Director** records each team's verdict (approve / approve with concerns / block)
4. **If all approve** → Director proceeds to shipping gate
5. **If concerns raised** → Director assigns follow-up tasks, schedules a brief follow-up
6. **If any team blocks** → Transition is blocked. Director delegates fixes to the responsible team, meeting is reconvened after fixes

### Token Budget

The Team Meeting is a single task delegation — not multiple round-trips. The Director invokes a single Task call with `subagent_type: reviewers` (or a dedicated meeting orchestration prompt) that loads all team perspectives at once. Token budget: **8k max** for the full meeting context.

### ADR References

- ADR-0003: Design document lifecycle — team meeting replaces the Director-only final review gate
- ADR-0008: Project lifecycle — team meeting is the new gateway between REVIEW and CLOSED
- ADR-0006: Second opinion — team meeting serves a similar validation function but synchronously

## Consequences

- Positive: Six perspectives catch more issues than one — reduces design-implementation drift
- Positive: Teams build shared context about the full project, not just their slice
- Positive: FLS gets direct handover from implementation teams, improving support readiness
- Positive: Blocks are surfaced early rather than discovered post-shipment
- Negative: Additional orchestration step before shipping gate (mitigated by single Task call)
- Negative: Token cost for the meeting context (8k budget, ~1-2k actual per meeting)
- Negative: If multiple teams block, multiple fix-and-reconvene cycles may be needed
