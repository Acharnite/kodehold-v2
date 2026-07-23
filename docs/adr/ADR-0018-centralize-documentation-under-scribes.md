---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0018: Centralize All Documentation Work Under Scribes

## Status

Accepted

## Context

Documentation work was scattered across all teams in KodeHold. Each team updated design docs, CHANGES.md, TODO.md, and ICM memories independently. This led to inconsistent documentation, duplicated effort, and missed updates when teams forgot to document their changes.

Without centralized documentation:

- Architects updated design documents after each design session
- Engineers updated README.md and CHANGES.md after implementation
- Testers updated test documentation independently
- Reviewers stored ICM memories directly (bypassing Scribes)
- FLS stored ICM memories directly (bypassing Scribes)
- No single team owned the documentation lifecycle
- Pre-transition documentation was inconsistent or missing

The key forces are:

- Scribes already existed as the documentation team but had a narrow scope
- Other teams were doing documentation as a side effect of their core work
- ICM memory operations were scattered across multiple teams
- The design document needs to be current before any state transition gate
- Post-task documentation must follow a consistent protocol

## Decision

Scribes owns ALL documentation across all states. Teams do their core work and nothing else.

### Scribes Responsibilities (Expanded)

| Responsibility | Description |
|---------------|-------------|
| Design document maintenance | Keep design doc current across all phases |
| ADR status management | Track ADR lifecycle (proposed → accepted → superseded) |
| CHANGES.md | Changelog with version history |
| TODO.md | Completed checklist + future roadmap |
| VERSION.md | Current version declaration |
| ICM memory operations | All icm_memory_store calls centralized here |
| Pre-transition documentation | Ensure design doc is current before gates |
| Session compression | Periodic ICM summaries (see ADR-0019) |

### Team Post-Task Protocol

All other teams gain a Post-Task Protocol:

> When a team completes its core work, it notifies the Director with a summary. The Director then delegates documentation updates to Scribes.

| Team | Core Work | Post-Task Action |
|------|-----------|-----------------|
| Architects | Design docs, ADRs | Notify Director → Scribes updates design doc |
| Engineers | Implementation | Notify Director → Scribes updates README, CHANGES |
| Testers | Testing | Notify Director → Scribes updates test docs |
| Reviewers | Code review | Notify Director → Scribes stores review results |
| FLS | Hotfix triage | Notify Director → Scribes stores findings |

### What Teams Still Do

Teams still READ design docs — they just don't UPDATE them. This ensures teams have context for their work while keeping documentation ownership with Scribes.

### ICM Memory Centralization

Previously, Reviewers and FLS had direct `icm_memory_store` calls in their agent definitions. These are removed:

- `reviewers.md`: removed direct ICM write calls
- `fls.md`: removed direct ICM write calls

All ICM write operations now go through Scribes. This prevents duplicate memories, inconsistent topics, and scattered knowledge.

### Scribes Workflow additions

**Pre-Transition Workflow:** Before any state transition gate, Scribes ensures the design document is current. This includes:

1. Reading the current design doc
2. Checking for stale sections
3. Updating with latest decisions and progress
4. Storing the updated state in ICM

**Session Compression Workflow:** Periodic ICM summaries compressed from chat history (see ADR-0019).

### Team Order Alignment

With this ADR, the team execution order is formalized:

1. Architects (design)
2. Engineers (implementation)
3. Testers (testing)
4. Reviewers (review)
5. FLS (support/triage)
6. Scribes (documentation — runs throughout)

This order is reflected in: ADR-0011, ADR-0018, director.md, and README.md.

## Consequences

- Positive: Single source of truth for all documentation — no more scattered updates
- Positive: ICM memory operations centralized — no duplicate memories or inconsistent topics
- Positive: Pre-transition workflow ensures design doc is always current at gate time
- Positive: Teams focus on core work — documentation is handled by the team that owns it
- Positive: Post-task protocol creates clear handoff between core work and documentation
- Negative: Scribes becomes a bottleneck if it falls behind — multiple teams waiting for documentation updates
- Negative: Additional delegation round for every team's documentation — ~10-15% more total tokens per cycle
- Neutral: Teams still read design docs for context — only update responsibility changes, not access
