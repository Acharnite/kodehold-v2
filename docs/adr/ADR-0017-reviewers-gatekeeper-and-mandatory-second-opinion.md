---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0017: Reviewers as Gatekeeper + Mandatory Second Opinion

## Status

Accepted

## Context

KodeHold's Reviewers team currently runs at the end of the ACTIVE phase — they review code and tests but have no formal role in state transitions. The Director executes gate transitions unilaterally, which means quality validation is decoupled from the transition decision. Additionally, there is no cross-model validation for ADRs or significant design changes — a single model can accept its own decisions.

Without Reviewers as gatekeeper:

- The Director can execute state transitions without quality validation
- Gate markers (`.design_reviewed`, `.testers_done`) are not verified before transition
- State transitions can proceed with incomplete or failed quality gates

Without mandatory second opinion:

- ADRs and design documents are reviewed by the same model that created them
- Self-review misses biases, blind spots, and architectural inconsistencies
- Security-critical code changes lack independent validation

The key forces are:

- Reviewers must validate transitions but not block emergency paths (CLOSED→REOPEN)
- The gate validation must be a dry-run (--validate-only) that does not change state
- Cross-model validation must be mandatory for high-impact decisions but optional for routine work
- The existing gate.sh script must support both validation modes (--validate-only, --reviewer-mode)
- Test coverage for the new validation logic must be comprehensive

## Decision

We implement two lifecycle changes: Reviewers become gatekeepers for state transitions, and a mandatory second opinion is required for ADRs and significant design updates.

### Part 1: Reviewers as Gatekeeper

Reviewers validate state transitions via `gate.sh --validate-only` before the Director executes them.

| Transition | Reviewers Gate | Required Markers |
|------------|---------------|------------------|
| INIT → ACTIVE | Yes | `.design_reviewed` + `.second_opinion_done` |
| ACTIVE → REVIEW | Yes | `.code_reviewed` + `.testers_done` |
| REVIEW → CLOSED | Yes | All ACTIVE→REVIEW markers + team meeting |
| REOPEN → ACTIVE | Yes | `.design_reviewed` + `.second_opinion_done` |
| CLOSED → REOPEN | **No** | Director-only (emergency path) |

### Gate Validation Flow

```
Director → Reviewers validate (--validate-only) → PASS → Director executes gate
                                        → BLOCKED → Director delegates fixes to responsible team
```

The Reviewers receive structured output from `gate.sh --reviewer-mode`:

| Flag | Output | Purpose |
|------|--------|---------|
| `--validate-only` | Dry run, no state change | Check if markers exist |
| `--reviewer-mode` | Structured output with GATE_RESULT, CHECKS, MARKERS_REQUIRED | Machine-readable validation |

### Part 2: Mandatory Second Opinion

Every ADR and significant design update requires cross-model validation.

**Mandatory second opinion for:**

- Every new ADR (accepted, proposed, or superseded)
- Design document updates exceeding 20% change
- Security-critical code changes
- Ambiguous design decisions with multiple valid approaches

**Optional second opinion for:**

- Complex bugs with uncertain root cause
- Minor documentation updates
- ICM operations and memory management

The second opinion is stored as a memory with the `.second_opinion_done` marker, integrating with the gate validation system.

### gate.sh Enhancements

Two new flags added to gate.sh:

| Flag | Behavior |
|------|----------|
| `--validate-only` | Dry run — checks markers without executing transition |
| `--reviewer-mode` | Structured output with GATE_RESULT, CHECKS, MARKERS_REQUIRED sections |

**Constraint:** `--yes` flag must be the first flag for proper argument passthrough in gate.sh.

### ADR Index Requirement

Reviewers must verify ADR status acceptance reviews include reference to the ADR index (`docs/adr/README.md`). This ensures cross-referencing and discoverability.

### Test Coverage

The Testers team verified the implementation:

- 34/34 new tests pass
- `--validate-only` blocks transitions without required markers
- `--validate-only` passes transitions with all required markers
- `--reviewer-mode` outputs structured sections (GATE_RESULT, CHECKS, MARKERS_REQUIRED)

## Consequences

- Positive: State transitions now require quality validation — cannot bypass gates
- Positive: Cross-model review catches biases and blind spots in ADRs and designs
- Positive: Emergency path (CLOSED→REOPEN) remains Director-only — not blocked by Reviewers
- Positive: gate.sh --validate-only provides dry-run capability for safe validation
- Positive: Structured output (--reviewer-mode) enables machine-readable gate checks
- Negative: Every state transition requires an additional Reviewer invocation — ~10-20% more Reviewer tokens per cycle
- Negative: `--yes` flag ordering constraint is a footgun — must be documented and tested
- Neutral: CLOSED→REOPEN is intentionally Director-only — emergency situations should not require gate validation
