---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0010: FLS — Front Line Support Team

## Status

Accepted

## Context

KodeHold projects follow a structured lifecycle (INIT → ACTIVE → REVIEW → CLOSED → REOPEN). Once a project reaches CLOSED, the full lifecycle must be invoked even for trivial bug fixes or small changes. This creates unnecessary ceremony for minor issues.

Without a dedicated support function:
- Minor bugs in CLOSED projects accumulate as "we'll fix it later" debt
- Small changes (typos, config tweaks, CSS fixes) require the same heavyweight process as major features
- There is no defined triage function to distinguish minor fixes from comprehensive changes
- The REOPEN lifecycle is invoked for every issue regardless of scope, wasting tokens and time

The key forces are:
- A support team must know the codebase and design docs to fix issues quickly
- There must be a clear escalation path when an issue exceeds the support scope
- All fixes must be documented in ICM for traceability
- The support function should not bypass quality gates for significant changes

## Decision

We introduce a 6th team — **FLS (Front Line Support)** — as the first line of defense for minor bugs and small changes.

### Team Role

FLS operates as a lightweight support layer between CLOSED and REOPEN:

```
CLOSED project → issue reported
    ├── Minor fix → FLS handles directly
    │                ├── Read design doc + ADRs
    │                ├── Implement fix
    │                ├── Verify with tests
    │                └── Document in ICM
    │
    └── Major change → FLS escalates
                         ├── Impact analysis
                         ├── ESCALATE to Director
                         └── Director runs CLOSED → REOPEN gate
```

### Triage Criteria

**Minor (FLS fixes directly):**
- Typo fixes, label changes, error message improvements
- Small CSS/UI tweaks with no layout restructuring
- Configuration value changes (environment variables, constants)
- Single-file changes with clear root cause and low blast radius
- No schema, data model, or API contract changes

**Major (FLS escalates to REOPEN):**
- Spans multiple files or modules
- Schema or data model changes
- New feature requests
- Security impact
- Performance regression
- Architectural changes
- Uncertain root cause requiring investigation

### Agent Definition

FLS is defined as an OpenCode subagent at `.opencode/agents/fls.md` with read/write/edit permissions and the following triggers: `support`, `hotfix`, `triage`, `escalate`, `minor-change`.

The Director delegates to FLS via the Task tool with `subagent_type: fls`.

### Integration with Existing Teams

| Interaction | Protocol |
|-------------|----------|
| Director → FLS | Delegate issue via Task tool with description and context |
| FLS → Director | Return fix summary, or `ESCALATE:` prefix for major issues |
| FLS → Scribes | All fixes documented in ICM (FLS requests Scribes via Director) |
| FLS → Testers | Verify fixes using existing test suite; no new test authoring |
| FLS → Architects | Not directly — escalation goes through Director to Architects |

## Consequences

- Positive: Minor fixes skip the full lifecycle — faster response, lower token cost
- Positive: Clear triage criteria prevents scope creep and gate bypass
- Positive: FLS maintains deep project knowledge across sessions, enabling rapid fixes
- Positive: Escalation path ensures major changes still receive proper design review
- Negative: Additional team increases orchestration surface area
- Negative: FLS must stay current with all CLOSED projects — knowledge maintenance overhead
- Negative: Risk of FLS fixing issues that should go through proper design review if triage is too permissive
- Neutral: FLS is a natural extension of the existing team model — same subagent pattern as all other teams
