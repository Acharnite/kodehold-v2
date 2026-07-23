---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0032: Routine Templates for KodeHold

## Status

Accepted

**Note (ADR-0050):** The routines defined in this ADR are preserved but now stored as static template tables in `.opencode/agents/director.md` instead of agentmemory. See ADR-0050 for the migration.

**Phase:** Phase 4 (Routine Templates) — builds on Phase 3's action model to define standard flow templates that can be instantiated with a single `memory_routine_run` call.

## Context

### The Problem

KodeHold executes several multi-step workflows repeatedly. Each time, the Director must manually create the full action dependency chain:

**Example — ADR creation flow:**
1. `memory_action_create(type="design", requires=[], priority=8)` — architects research
2. `memory_action_create(type="design", requires=["action-001"], priority=8)` — architects write ADR
3. `memory_action_create(type="document", requires=["action-002"], priority=5)` — scribes update design doc
4. `memory_action_create(type="review", requires=["action-002"], priority=7)` — reviewers review
5. `memory_action_create(type="second-opinion", requires=["action-002"], priority=7)` — cross-validate
6. `memory_action_create(type="document", requires=["action-004", "action-005"], priority=5)` — scribes finalize

This is 6 `memory_action_create` calls, 6 `requires` chain constructions, and 6 manual priority assignments — every time the Director needs an ADR created.

The same pattern applies to feature implementation, bug fixes, and the shipping gate.

### Key Forces

1. **Templates must be parameterized.** Not every ADR needs a second opinion. Templates must support optional steps.
2. **Templates must be versioned.** As workflows evolve, templates must be updatable without breaking in-flight flows.
3. **Templates are action DAGs, not scripts.** A routine template is a graph of actions with dependencies.
4. **Graceful degradation.** If `memory_routine_run` fails, fall back to manual `memory_action_create`.
5. **Transactional creation.** If a template fails mid-way, already-created actions must be cleaned up.

### Prior Art

- **ADR-0031** (Actions + Crystals) — established the action model that templates build on
- **ADR-0029** (Migration Strategy) — Phase 4 of the migration plan
- **Actions+Crystals design doc** (Section 8) — defines the 4 template DAGs
- **`scripts/ship.sh`** — the 7 automated shipping steps

## Decision

### Template 1: `kodehold-adr-flow` (5 steps)

**Version:** 1.1 | **Category:** design | **Author:** Architects

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | architects | research + write-adr | (none) | 8 | No |
| 2 | scribes | design-doc-update | step 1 | 5 | No |
| 3 | reviewers | review-adr | step 1 | 7 | No |
| 4 | second-opinion | cross-validate | step 1 | 7 | Yes |
| 5 | scribes | finalize | steps 3, 4* | 5 | No |

*If step 4 is skipped, step 5 depends only on step 3.

**Parameters:** `title` (required), `require_second_opinion` (boolean, default true)

### Template 2: `kodehold-implement-flow` (6 steps)

**Version:** 1.0 | **Category:** implement | **Author:** Architects

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | architects | design | (none) | 8 | No |
| 2 | reviewers | design-review | step 1 | 7 | Yes |
| 3 | engineers | implement | step 2* | 8 | No |
| 4 | reviewers | code-review | step 3 | 7 | No |
| 5 | testers | test | step 3 | 6 | No |
| 6 | reviewers | gate-validation | steps 4, 5 | 9 | No |

*If step 2 is skipped, step 3 depends on step 1 directly. Steps 4-5 are parallel (fan-out). Step 6 is fan-in.

**Parameters:** `feature_description` (required), `skip_design_review` (boolean, default false)

### Template 3: `kodehold-bugfix-flow` (5 steps, branching)

**Version:** 1.0 | **Category:** bugfix | **Author:** FLS

| Step | Team | Action | Depends On | Priority | Condition |
|------|------|--------|------------|----------|-----------|
| 1 | fls | triage | (none) | 7 | Always |
| 2a | fls | hotfix | step 1 | 8 | severity < threshold |
| 3 | scribes | document | step 2a | 5 | minor path |
| 4 | reviewers | verify | step 2a | 7 | minor path |
| 2b | — | → REOPEN + implement-flow | step 1 | — | severity ≥ threshold |

The Director evaluates the triage result and chooses the branch. The template cannot auto-branch.

**Parameters:** `issue_ref` (required), `severity_threshold` (int, default 7)

### Template 4: `kodehold-ship-gate` (7 steps)

**Prerequisite:** Step 0 (Team Meeting, ADR-0011) must be completed before instantiation.

| Step | Team | Action | Priority | Description |
|------|------|--------|----------|-------------|
| 1 | director | version-check | 9 | VERSION.md exists + parses |
| 2 | director | changelog-check | 9 | CHANGES.md entry exists |
| 3 | director | todo-check | 9 | TODO.md exists |
| 4 | testers | test-suite | 9 | Full test suite passes |
| 5 | director | agentmemory-check | 9 | Daemon accessible |
| 6 | director | git-status | 9 | Git clean, changes staged |
| 7 | director | branch-check | 9 | Correct branch |

All 7 steps have no dependencies — they can run in parallel (fan-out).

**Parameters:** `version` (required), `project` (required)

### Template Registration

Templates are stored in agentmemory via `memory_routine_run` or, as fallback, as structured JSON in a slot (`slot_create(label="routine_templates")`).

### Fallback

If `memory_routine_run` fails:
- **Before creation**: Fall back to manual `memory_action_create` per action
- **Partial creation**: Cancel created actions, fall back to manual
- **Template not found**: Try slot-based storage, then manual

## Consequences

### Positive
1. **Single-call workflow instantiation.** 6 `memory_action_create` calls → 1 `memory_routine_run` call
2. **Consistent dependency graphs.** No manual errors in `requires` chains
3. **Automatic fan-out/fan-in.** Parallel steps handled by the template
4. **Parameterized optional steps.** Skip second opinion/design review via parameters
5. **Versioned workflows.** Templates evolve without breaking in-flight flows
6. **Reduced Director logic.** Flow logic moves from agent definition to templates

### Negative
1. **Template maintenance overhead.** 4 templates × N steps = ongoing maintenance
2. **Rigidity for edge cases.** Non-standard work uses manual creation
3. **Parameter explosion danger.** Mitigation: keep to 2-3 parameters per template
4. **Agentmemory dependency.** Falls back to slot storage if `memory_routine_run` unavailable

### Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Template drift | Medium | Medium | Version template IDs |
| Partial instantiation failure | Low | High | Cancel created actions on failure |
| No native `memory_routine_run` | Medium | Medium | Slot-based fallback |
| Template misuse | Low | Low | Descriptive template IDs |

### Follow-up
- [ ] Register 4 templates in agentmemory
- [ ] Update director.md with routine trigger detection
- [ ] Test each template with at least one instantiation

## ADR References
- **ADR-0031** (Actions + Crystals) — action model foundation
- **ADR-0029** (Migration Strategy) — Phase 4 implementation
- **ADR-0011** (Team Meeting) — ship gate prerequisite
- **`scripts/ship.sh`** — shipping gate steps
