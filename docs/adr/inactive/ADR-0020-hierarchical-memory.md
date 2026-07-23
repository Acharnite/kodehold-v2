---
status: Superseded
superseded-by: agentmemory (memory_consolidate tier system)
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0020: Hierarchical Memory (Hot/Warm/Cold)

## Status

Superseded

Replaced by agentmemory's memory_consolidate tier system and memory_save importance field. Agentmemory provides working→episodic→semantic→procedural consolidation tiers, importance-based priority (via the importance parameter), auto-dedup, and health monitoring. GitHub issue #26 closed.

## Context

KodeHold currently stores all ICM memories with static importance levels (critical/high/medium/low) but lacks automatic tiered storage based on access patterns. Every memory retrieval scans the full store regardless of age or relevance, wasting tokens on stale data.

The current approach has these limitations:

- Static importance levels don't reflect actual usage — a `high` memory from 2 weeks ago may be less relevant than a `medium` memory from today
- No automatic promotion/demotion — memories stay at their assigned importance forever (unless manually updated)
- Token budget is wasted retrieving stale memories that haven't been accessed recently
- No distinction between "hot" session context and "cold" historical records
- The session context compression (ADR-0019) addresses chat growth but not memory tier optimization

The key forces are:

- KodeHold targets 32K context models where every token matters
- ICM already has importance-based retrieval, but importance is static
- Access frequency and recency are natural signals for relevance
- OS-style memory management (hot/warm/cold) is a proven pattern for tiered storage
- The existing ICM infrastructure supports metadata tags — no schema changes needed

## Decision

Implement 3-tier memory classification with automatic promotion/demotion based on access frequency and recency.

### Memory Tiers

| Tier | Description | Access Pattern | Auto-Promote | Auto-Demote |
|------|-------------|---------------|--------------|-------------|
| **Hot** | Current session context, actively used | Accessed within last session | On session access | After session ends, if importance < high |
| **Warm** | Recent high-value memories | Accessed in last 7 days | On 3+ accesses in 7 days | After 14 days without access |
| **Cold** | Archived, low-frequency memories | Not accessed in 14+ days | On 5+ accesses in 30 days | Never auto-demoted (manual only) |

### Promotion Rules

| Trigger | From | To | Condition |
|---------|------|----|-----------|
| Session start | Warm | Hot | Memory loaded via `icm_wake_up` |
| 3+ accesses in 7 days | Cold | Warm | Access count threshold |
| 5+ accesses in 30 days | Cold | Warm | For older memories |

### Demotion Rules

| Trigger | From | To | Condition |
|---------|------|----|-----------|
| Session ends | Hot | Warm | Memory not re-accessed in 2 consecutive sessions |
| 14 days no access | Warm | Cold | Recency threshold |
| Manual override | Any | Any | User/Director explicitly sets tier |

### Storage Tags

Memories are tagged with metadata in ICM:

```
tier: hot|warm|cold
last_accessed: <timestamp>
access_count: <int>
promoted_at: <timestamp>
```

### Retrieval Optimization

| Tier | Retrieval Strategy | Token Budget |
|------|-------------------|-------------|
| Hot | Always loaded on session start | ~500 tokens |
| Warm | Loaded via `icm_memory_recall` with recency filter | ~1000 tokens |
| Cold | Loaded on-demand only (explicit search) | As needed |

### Integration Points

- **Session start (ADR-0019):** `icm_wake_up` loads Hot tier; Director loads Warm tier via recall
- **Session end:** Scribes demotes Hot→Warm for non-re-accessed memories
- **Weekly sweep:** Scribes runs demotion pass (Warm→Cold for 14-day stale)
- **Access tracking:** Each `icm_memory_recall` increments access count and updates `last_accessed`

### Implementation Plan

| File | Change |
|------|--------|
| scribes.md | Add tier management workflow, weekly sweep, access tracking |
| director.md | Add tier-aware retrieval at session start |
| icm-knowledge-flow SKILL.md | Add tier metadata to storage steps |
| design doc | Add section 7.6 — Hierarchical Memory |

## Consequences

- Positive: Token savings of ~30-40% by not retrieving stale memories unnecessarily
- Positive: Hot tier ensures session-critical context is always available
- Positive: Automatic promotion/demotion reduces manual memory management burden
- Positive: Cold tier preserves historical context without polluting active retrieval
- Negative: Adds complexity to ICM operations — access counting, tier checks, sweeps
- Negative: Demotion may occasionally remove memories that are still relevant but rarely accessed
- Negative: Requires tracking access metadata (last_accessed, access_count) on all memories
- Neutral: Tier thresholds (3 accesses, 14 days) may need tuning based on actual usage patterns
- Note: agentmemory's memory_consolidate (working→episodic→semantic→procedural) and importance-based tiering cover this completely. No separate hierarchical memory layer needed.
