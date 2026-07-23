---
status: Superseded
superseded-by: ADR-0052 (Structured Durable Execution — Formal Checkpoint Schema and Auto-Checkpoint)
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0019: Session Context Compression via Periodic ICM Summaries

## Status

Superseded

Superseded by ADR-0052 (Structured Durable Execution), which defines a formal YAML frontmatter checkpoint schema with thread-aware storage, auto-checkpoint on every delegation, and machine-validatable incomplete work detection.

## Context

KodeHold targets 32K context models (Ollama) where chat history grows with every delegation round. Each round includes the Director's analysis, Task tool calls, team responses, and file reads — easily 1K-2K tokens per round. After 10-15 rounds, context approaches overflow.

Without context compression:

- Chat history grows monotonically — no mechanism to reclaim context space
- Small context models (32K) hit overflow after ~15 delegation rounds
- The Director must choose between truncating history (losing context) or stopping work
- Existing checkpoint protocol (ADR-0014) preserves state snapshots but does not compress chat
- Session restart is the only way to recover context — expensive and disruptive

The key forces are:

- Compression must preserve decision-relevant context while discarding verbose tool output
- Summaries must be structured so the Director can load them quickly after compression
- The compression frequency must balance context savings against information loss
- The existing ICM infrastructure can store summaries — no new storage layer needed
- The compression must complement, not replace, the existing checkpoint protocol

## Decision

Scribes compresses the running chat every 4 delegation rounds into structured ICM summaries.

### Compression Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Frequency | Every 4 delegation rounds | Balances context savings vs. information loss |
| Summary size | 200-400 tokens each | Small enough to reclaim significant context |
| Original chat | 2K-5K tokens per 4 rounds | Typical delegation round output |
| Savings | 60-80% per compression cycle | Significant context reclamation |
| ICM topic | `kodehold-<project>-session-summary` | Consistent with existing ICM structure |
| Importance | `high` | Persists across sessions but decays faster than critical |

### Summary Template

Each compression produces a structured summary with these sections:

```
## Session Summary — Round <N>

### Completed
- <what was done in this batch of rounds>

### In-Progress
- <what is currently being worked on>

### Decisions
- <key decisions made, with rationale>

### Files
- <files modified, created, or reviewed>

### Teams
- <which teams were involved and their status>

### Blockers
- <any blockers or issues encountered>

### Carry-Forward
- <context needed for next rounds>
```

### Compression Triggers

| Trigger | Condition | Action |
|---------|-----------|--------|
| Round counter | Every 4 delegation rounds | Scribes compresses chat to ICM summary |
| State transition | Any state change | Scribes compresses current phase context |
| Explicit request | User or Director requests | Scribes compresses immediately |

The Director counts delegation rounds in head and resets the counter on state transitions.

### Consolidation

ICM topics accumulate entries over time. To prevent overflow:

- Maximum 10 entries per `kodehold-<project>-session-summary` topic
- At threshold (10 entries), Scribes consolidates oldest 5 entries into a single "session history" entry
- Uses `icm_memory_consolidate` to merge entries without losing information
- Consolidated entry retains all decisions and carry-forward context

### Wake-Up Integration

After `icm_wake_up` loads critical facts, the Director loads the latest session summary:

1. `icm_wake_up` — loads critical/high memories
2. `icm_memory_recall -t kodehold-<project>-session-summary -i high` — loads latest summary
3. Director presents summary to user for context reconstruction

### Relationship to Checkpoint Protocol

| Aspect | Checkpoint (ADR-0014) | Compression (ADR-0019) |
|--------|----------------------|------------------------|
| Purpose | State snapshot for resume | Chat history compression |
| Importance | Critical (never decay) | High (normal decay) |
| Content | Project state, decisions, next steps | Verbatim chat summary |
| Trigger | Before context overflow | Every 4 delegation rounds |
| Frequency | On-demand | Periodic |

Checkpoints are the safety net; compression is the日常 maintenance.

### Implementation Plan

| File | Change |
|------|--------|
| director.md | Add compression protocol, round counter, reset logic |
| scribes.md | Add compression workflow, summary template, consolidation logic |
| design doc | Add section 7.5 — Session Context Compression |

## Consequences

- Positive: 60-80% context savings per compression cycle — extends usable session length
- Positive: Structured summaries preserve decision context while discarding verbose output
- Positive: Complements existing checkpoint protocol — compression for日常, checkpoints for safety
- Positive: ICM storage means summaries persist across sessions — no information loss
- Positive: Consolidation prevents ICM topic overflow (max 10 entries)
- Negative: Compression is lossy — some detail is lost in summarization
- Negative: Director must remember to count rounds and trigger compression (prompt-level enforcement)
- Neutral: Summary template is rigid — may not fit all session types perfectly
- Neutral: Designed but not yet implemented — requires testing on 32K context models to validate savings
