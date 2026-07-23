---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0003: Design Document Lifecycle

## Status

Accepted

## Context

The design document is the central artifact of KodeHold. It must have a defined lifecycle that governs how it is created, reviewed, updated, and superseded. Without a lifecycle, design documents become stale, are bypassed, or accumulate drift from the actual implementation.

The lifecycle must handle:
- Initial creation before any code exists
- Incremental updates during implementation
- Reviews at defined quality gates
- Supersession when the document is no longer relevant
- Reopening when a closed project gains new requirements

## Decision

### Lifecycle States

```
CREATE → REVIEW → APPROVE → [ACTIVE] → UPDATE → REVIEW → APPROVE → ...
                                  │
                            PROJECT CLOSED
                                  │
                            [REOPEN] → UPDATE → REVIEW → APPROVE → ACTIVE
```

| State | Description | Owner |
|-------|-------------|-------|
| Draft | Initial creation, structure filled in | Architects |
| Review | Submitted for review, feedback collected | Reviewers |
| Approved | Passed review, ready for implementation | Director |
| Active | Being implemented, may receive incremental updates | All teams |
| Updating | Undergoing revision (new sections, changes) | Architects |
| Superseded | Replaced by a newer design document | Director |

### Review Gates

1. **Initial Review** — Before any implementation starts. Architects present, Reviewers approve.
2. **Change Review** — Any modification to an Active document. Reviewers verify consistency.
3. **Impact Review** — When a closed project is reopened. Reviewers + Architects assess scope.
4. **Final Review** — Before project close. Team Meeting (see ADR-0011) replaces solo Director approval. All six teams review collectively.

### Design Document Template

All design documents follow this structure:

```
# Project: [Name]
**Version:** x.y
**Status:** Draft | Active | Updating | Superseded
**Design Authority:** [Architects team ref]
**Last Reviewed:** YYYY-MM-DD

## 1. Purpose & Scope
## 2. Requirements
## 3. Architecture Overview
## 4. Component Design
## 5. Data Model
## 6. API Design
## 7. Implementation Plan
## 8. Testing Strategy
## 9. ADR Index
## 10. Open Questions
## 11. Changelog
```

Documents are stored at `docs/design/<project-name>.md`.

## Consequences

- Positive: Clear process prevents "coding before thinking"
- Positive: Versioned document history via git
- Positive: Review gates catch design issues before they reach implementation
- Negative: Process overhead for small changes — mitigated by allowing minor updates without full review cycle
- Negative: Requires discipline from teams to keep document in sync with code
- Neutral: Superseded documents remain in git for historical reference
