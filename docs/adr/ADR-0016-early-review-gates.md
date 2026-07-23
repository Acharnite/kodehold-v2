---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0016: Early Review Gates in ACTIVE Phase

## Status

Accepted

## Context

KodeHold's original ACTIVE phase had a single review checkpoint: Testers run tests, then Reviewers review everything at the end. This late-stage review means design flaws and implementation drift are caught only after significant rework has already been done.

Without intermediate review gates:

- Design flaws by Architects are not caught until after Engineers have implemented based on them
- Implementation drift (code not matching design) is caught only during final review
- Test failures caused by design issues require rework across multiple teams
- The Reviewer role is limited to a single end-of-phase checkpoint
- Rework cost increases exponentially the later a defect is discovered

The key forces are:

- Each defect category (design, implementation, integration) has an optimal detection point
- Early detection is cheaper — fixing a design flaw before implementation costs a document edit, fixing it after costs a code rewrite + retest
- Adding review checkpoints increases token budget per ACTIVE cycle (~30-50% more Reviewer work)
- The gates must be lightweight — they should catch obvious issues, not be full reviews
- Gate markers must integrate with the existing gate.sh validation system

## Decision

We add two intermediate review checkpoints within the ACTIVE phase, creating a shift-left quality approach:

### Three-Gate ACTIVE Flow

```
Architects → Gate 1 → Engineers → Gate 2 → Testers → Gate 3
```

| Gate | After | Marker | Validates | Catches |
|------|-------|--------|-----------|---------|
| Gate 1 | Architects | `.design_review_v2` | Design document accuracy, feasibility | Design flaws |
| Gate 2 | Engineers | `.code_reviewed` | Code matches design, no obvious issues | Implementation drift |
| Gate 3 | Testers | `.testers_done` | Tests pass, integration works | Integration issues |

### Gate Responsibilities

**Gate 1: Design Review (after Architects)**

- Reviewers validate the design document for completeness and feasibility
- Check: are requirements clear? are components well-defined? are interfaces specified?
- Output: `.design_review_v2` marker on approval
- Defect caught: design flaws — cheapest to fix at this stage (document edit vs. code rewrite)

**Gate 2: Code Review (after Engineers)**

- Reviewers validate implementation against design document
- Check: does code match the design? are edge cases handled? is error handling present?
- Output: `.code_reviewed` marker on approval
- Defect caught: implementation drift — cheaper to fix now (code adjustment) than after testing (rework + retest)

**Gate 3: Test Verification (after Testers)**

- Reviewers validate that tests pass and integration works
- Check: do tests cover the requirements? are there regressions? does the system work end-to-end?
- Output: `.testers_done` marker (existing behavior)
- Defect caught: integration issues — requires rework across teams if caught here

### Defect Cost by Detection Point

| Defect Type | Caught at Gate 1 | Caught at Gate 2 | Caught at Gate 3 | Caught post-merge |
|-------------|-------------------|-------------------|-------------------|-------------------|
| Design flaw | Document edit | Code rewrite | Rework + retest | Hotfix + rework |
| Implementation drift | — | Code adjustment | Rework + retest | Hotfix + rework |
| Integration issue | — | — | Rework | Hotfix + rework |

### Gate.sh Integration

Gate markers are enforced by `gate.sh --validate-only` (see ADR-0017). The ACTIVE→REVIEW transition requires all three markers:

- `.design_review_v2` — Gate 1 passed
- `.code_reviewed` — Gate 2 passed
- `.testers_done` — Gate 3 passed

Without all three markers, the transition is blocked.

### Token Budget Impact

| Phase | Before ADR-0016 | After ADR-0016 | Change |
|-------|-----------------|----------------|--------|
| ACTIVE review work | 1 Reviewer pass | 3 Reviewer passes | +200% |
| Estimated tokens per ACTIVE | ~2K Reviewer tokens | ~3-3K Reviewer tokens | +30-50% |

This increase is acceptable because the rework reduction outweighs the additional review cost. A single design flaw caught at Gate 1 saves the full implementation + testing + review cycle.

## Consequences

- Positive: Design flaws caught before implementation — cheapest fix point
- Positive: Implementation drift caught before testing — avoids retest cycle
- Positive: Each gate catches a specific defect category at its optimal detection point
- Positive: Gate markers integrate with existing gate.sh validation system
- Positive: Shift-left quality reduces total rework across the ACTIVE phase
- Negative: ~30-50% more Reviewer token budget per ACTIVE cycle
- Negative: Three review passes per ACTIVE phase instead of one — more Reviewer invocations
- Neutral: Gate 3 is unchanged from existing behavior — no disruption to current test verification flow
