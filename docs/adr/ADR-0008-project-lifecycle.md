---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0008: Project Lifecycle and Reopening

## Status

Accepted

## Context

KodeHold projects have a lifecycle: they are initiated, developed, reviewed, and closed. But software is never truly finished — bugs are discovered, new features are requested, dependencies need updating. A project must be reopenable months or years after closure with full context restored.

Without a defined lifecycle and reopening protocol:
- Closed projects accumulate "we'll fix it later" debt
- Reopening requires manual context reconstruction
- Design documents drift from the actual codebase over time

## Decision

### Project States

```
                  ┌──────────────────────────────────┐
                  │            INIT                   │
                  │  Design doc created               │
                  │  ADRs drafted                     │
                  │  Scope defined                    │
                  └──────────┬───────────────────────┘
                             │ Director approves
                             ▼
                  ┌──────────────────────────────────┐
                  │           ACTIVE                   │
                  │  Implementation in progress        │
                  │  Design doc updates via review      │
                  │  Continuous testing                 │
                  └──────────┬───────────────────────┘
                             │ All work complete
                             ▼
                   ┌──────────────────────────────────┐
                   │           REVIEW                   │
                   │  Full test suite run by Testers    │
                   │  Final review by Reviewers         │
                   │  Design doc verification            │
                   └──────────┬───────────────────────┘
                             │ Director closes
                             ▼
                   ┌──────────────────────────────────┐
                   │           CLOSED                   │
                   │  Context stored in ICM             │
                   │  Design doc archived               │
                   │  Project inactive                   │
                   └──────────┬───────────────────────┘
                              │ Issue reported
                              ▼
                   ┌──────────────────────────────────┐
                   │    FLS Triage (see ADR-0010)       │
                   │  ┌─ Minor → FLS fixes directly    │
                   │  └─ Major → escalate to REOPEN    │
                   └──────────┬───────────────────────┘
                              │ Major change
                              ▼
                   ┌──────────────────────────────────┐
                   │           REOPEN                   │
                   │  Context loaded from ICM            │
                   │  Impact analysis (Architects)       │
                   │  Design doc updated                  │
                   └──────────┬───────────────────────┘
                              │ → ACTIVE (above)
```

### Close Protocol

When a project is closed, the Director orchestrates:

1. **Team Meeting** (see ADR-0011): All six teams present — Architects, Engineers, Testers, Reviewers, Scribes, FLS. Each team approves or blocks the transition. Replaces solo Director approval.
2. **FLS**: Onboard FLS with project context — design doc, ADRs, key code areas, test patterns. FLS must be ready to triage future issues.
3. **Testers**: Run complete test suite. All tests must pass.
3. **Reviewers**: Final code review. Verify design doc matches implementation.
4. **Architects**: Final ADR review. Any decisions made during implementation that lack ADRs must be documented.
5. **Scribes**:
   - Store full project summary in ICM (memories with high importance)
   - Extract concepts for cross-project knowledge
   - Store design doc and ADR index as permanent memories
   - Log project status, dates, and team composition
   - Tag memories so FLS can recall them rapidly

### FLS Triage Protocol (CLOSED → minor fix or REOPEN)

When an issue is reported on a CLOSED project:

1. **Director** delegates to `fls` subagent with the issue description
2. **FLS** triages per criteria in ADR-0010:
   - **Minor**: Read design doc + ADRs, implement fix, verify, document in ICM
   - **Major**: Prepare escalation summary with impact assessment
3. **If minor**: FLS returns fix summary. No lifecycle transition needed.
4. **If major (ESCALATE)**: Director receives escalation, runs CLOSED → REOPEN gate, then follows the reopen protocol below.

### Reopen Protocol

When a project is reopened (via FLS escalation or direct request):

1. **Director**: Receives request with new requirements (features or bugfixes)
2. **Scribes**: Query ICM for all project memories. Load design doc, ADRs, and final context. Summarize for teams.
3. **Architects**: Perform impact analysis. Update design doc with new requirements. Write new ADRs for significant changes.
4. **Reviewers**: Review updated design doc and new ADRs.
5. **Director**: Transition to ACTIVE state. Assign implementation work to Engineers.

### Conditions for Reopening

- Bug fixes: Any project can be reopened for bug fixes regardless of age
- New features: Only if the design doc can accommodate them without requiring a full rewrite. If > 50% of the design would change, a new project should be created instead.
- Dependency updates: Can be handled as a bug fix (security) or new feature (major version bump)
- FLS escalation: If FLS determines an issue is too comprehensive for a hotfix, it escalates to REOPEN

### Archive

After 12 months in CLOSED state without reopening:
- Scribes archive low-level memories (keep only high-importance summaries)
- Full project code remains in git but ICM stores only essential context
- Reopening an archived project requires a full context rebuild (design doc re-review)

## Consequences

- Positive: Long-running projects maintain full context across months or years
- Positive: ICM stores project history for post-mortem analysis and training
- Positive: Reopen protocol ensures consistent process regardless of who initiates
- Negative: Archiving after 12 months means some context loss for very old projects
- Negative: Reopening requires the same teams to be available (may not be possible if team composition changed)
- Neutral: Projects that require >50% redesign are treated as new — prevents scope creep
