---
status: Superseded
superseded-by: agentmemory (memory_lease + memory_signal)
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0024: Shared Memory (Multi-Agent Alignment)

## Status

Superseded

Replaced by agentmemory memory_lease (action locking) + memory_signal (inter-agent communication). Teams never work simultaneously on the same resources. Marker-based coordination provides sufficient state propagation. GitHub issue #30 closed as over-engineering.

## Context

KodeHold has a central ICM store, but there is no real-time state synchronization between agents. Teams may work on overlapping files without awareness of each other's changes. The Director orchestrates sequentially, but within a delegation round, multiple agents could theoretically access the same resources.

The current approach has these limitations:

- No file-level locking — two teams could modify the same file simultaneously
- No conflict detection when teams work on overlapping areas
- ICM memories are written asynchronously — agents may read stale data
- No verified state propagation — agents assume ICM is current without verification
- The commit protection protocol (design doc §6.4) prevents data loss but not conflicts
- Workspace projects share the central `.icm/` but have no coordination mechanism

The key forces are:

- KodeHold uses sequential delegation (Architects → Engineers → Reviewers), reducing conflict risk
- Within a round, file reads by one team and writes by another could cause inconsistency
- ICM is the single source of truth, but agents may cache or assume freshness
- Locking adds overhead — must be lightweight for the sequential workflow
- Conflict resolution must be automated where possible, manual where not

## Decision

Implement a shared memory layer with agent locking on common resources, conflict detection, and verified state propagation.

### Resource Locking

| Resource | Lock Type | Duration | Who Locks |
|----------|-----------|----------|-----------|
| Design doc | Read lock | During active design work | Architects |
| Design doc | Write lock | During design updates | Architects (exclusive) |
| Code files | Write lock | During implementation | Engineers (per-file) |
| ADR files | Write lock | During ADR creation | Architects (per-file) |
| Test files | Write lock | During test writing | Testers (per-file) |
| ICM topic | Write lock | During memory operations | Scribes (per-topic) |

### Lock Protocol

```
1. Agent requests lock on resource
2. Check if resource is locked by another agent
3. If unlocked → grant lock, record in ICM
4. If locked by same agent → grant (reentrant)
5. If locked by different agent → wait or escalate to Director
6. Agent completes work → release lock
7. Lock expires after 5 minutes (timeout)
```

### Conflict Detection

| Conflict Type | Detection | Resolution |
|---------------|-----------|------------|
| Same file, different teams | Git diff on commit | Manual merge by Director |
| ICM topic, concurrent writes | Lock check before write | Scribes serializes writes |
| Design doc + code mismatch | Reviewers gate check | Reviewers flag in gate review |
| Stale ICM read | Timestamp comparison | Re-fetch from ICM before critical decisions |

### State Propagation

| Event | Propagation | Verification |
|-------|-------------|-------------|
| Design doc updated | Notify Architects team | Check `.design_review_v2` marker |
| Code committed | Notify Engineers team | Check `git log --oneline -1` |
| ADR created | Notify Architects team | Check `docs/adr/` directory |
| ICM memory stored | Notify Scribes team | Check `icm_memory_recall` |
| State transition | Notify all teams | Check `.kodehold-state` |

### Lightweight Implementation

Given KodeLock's sequential delegation model, the shared memory layer is intentionally lightweight:

| Mechanism | Purpose | Overhead |
|-----------|---------|----------|
| Git status check | Detect file conflicts before write | ~10 tokens |
| ICM lock record | Prevent concurrent topic writes | ~20 tokens |
| Timestamp comparison | Detect stale reads | ~5 tokens |
| Director notification | Manual conflict resolution | ~50 tokens |

Total overhead per delegation round: ~85 tokens (acceptable).

### Integration Points

- **Commit protection (design doc §6.4):** Shared memory adds file awareness before commits
- **ADR-0015 (Delegation enforcement):** Locks complement tool permissions
- **ADR-0019 (Session compression):** Lock state is included in session summaries
- **ADR-0020 (Hierarchical memory):** Tier transitions respect lock state

### Implementation Plan

| File | Change |
|------|--------|
| scribes.md | Add lock management, conflict detection, state propagation |
| director.md | Add lock check before delegation, conflict resolution workflow |
| engineers.md | Add file lock acquisition before code writes |
| design doc | Add section 7.10 — Shared Memory Layer |

## Consequences

- Positive: Prevents file conflicts between teams working on overlapping areas
- Positive: Verified state propagation ensures agents work with current data
- Positive: Lightweight implementation fits KodeLock's sequential delegation model
- Positive: Lock timeout prevents deadlocks if an agent crashes
- Negative: Adds ~85 tokens overhead per delegation round for lock operations
- Negative: Manual conflict resolution required for file-level conflicts
- Negative: Lock management adds complexity to ICM operations
- Neutral: Lock timeout (5 minutes) may need tuning based on actual work durations
- Note: agentmemory's memory_lease provides action-level locking; memory_signal provides inter-agent communication.
