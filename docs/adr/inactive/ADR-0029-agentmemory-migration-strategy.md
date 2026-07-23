---
phase:
  current: 6
  total: 6
  names:
    1: "Awareness — dual-write ICM+agentmemory"
    2: "Infrastructure — replace ICM tools"
    3: "Frontier — replace todowrite"
    4: "Routines — standard flow templates"
    5: "Crystals + Signals — auto-crystallize"
    6: "Cleanup — remove legacy references"
  status:
    1: done
    2: done
    3: done
    4: done
    5: done
    6: done
---

# ADR-0029: ICM → Agentmemory Migration Strategy

## Status

Deprecated

> All 5 migration phases completed. ICM fully replaced by agentmemory across all agents, scripts, and references.

## Context

### The Problem

KodeHold has two memory systems operating in parallel:

1. **ICM (Infinite Context Memory)** — the original memory system. Used by all 8 agent definitions, 2 skills, multiple scripts, and referenced across 9 ADRs. ICM provides: memory store/recall, memoir/concept storage, session tracking, topic-scoped queries, and consolidation.

2. **Agentmemory** — a newer, more capable memory system that provides all ICM capabilities natively plus: semantic search (`memory_smart_search`), lessons (`memory_lesson_recall`/`memory_lesson_save`), pattern detection (`memory_patterns`), provenance verification (`memory_verify`), and an orchestration layer (actions, crystals, routines, signals, sentinels, sketches).

**The core problem:** Dual-maintenance on two memory systems creates confusion, increases token consumption, and prevents adoption of agentmemory's advanced orchestration features (actions, crystals, routines). ADR-0028 established the pattern of migrating from ICM to agentmemory, but no overarching strategy exists.

### Key Forces

1. **Zero downtime.** The migration cannot break existing workflows. Every phase must maintain backward compatibility.
2. **31+ files affected.** Agent definitions, skills, design docs, ADRs, scripts, config files, and reference files all contain ICM references.
3. **Dual-write safety.** Phase 1 establishes dual-writes (ICM + agentmemory) so agentmemory can be validated before ICM is removed.
4. **Coexistence required.** ICM data in `.icm/` remains valuable read-only archive. Agentmemory cannot read ICM's SQLite database directly.
5. **Rollback must be possible.** If agentmemory has critical failures, KodeHold must be able to fall back to ICM.

### Tool Mapping Foundation

Every ICM tool has a direct agentmemory equivalent:

| ICM Tool | Agentmemory Equivalent | Confidence |
|----------|----------------------|------------|
| `icm_memory_store` | `memory_save` | High — same semantics (content, project, type) |
| `icm_memory_recall` | `memory_recall` | High — semantic + keyword hybrid search |
| `icm_memory_update` | `memory_save` (same id) | High — overwrite existing memory |
| `icm_memory_forget` | `memory_governance_delete` | Medium — requires `reason` param |
| `icm_memory_consolidate` | `memory_consolidate` | High — same concept, tier parameter |
| `icm_memory_list_topics` | `memory_slot_list` + `memory_recall` | Low — no direct equivalent |
| `icm_memory_stats`/`health` | `memory_diagnose` | High — health check |
| `icm_memoir_search` | `memory_lesson_recall` | High — query lessons |
| `icm_memoir_search_all` | `memory_smart_search` | High — cross-source hybrid |
| `icm_memoir_refine` | `memory_lesson_save` (same content) | High — update lesson |
| `icm_memory_extract_patterns` | `memory_patterns` | Medium — similar concept |
| `icm_memoir_link` | `memory_facet_tag` | Medium — tagging provides linking |
| `icm_memoir_inspect` | `memory_verify` | High — provenance tracking |

### Prior Art

- **ADR-0028** (Agentmemory Project Detection) — established the full filesystem path as canonical project name, removed the last blocker for agentmemory migration
- **ADR-0004** (ICM and RTK Integration) — foundational ICM integration that this strategy supersedes
- **ADR-0009** (ICM MCP Integration) — layered ICM architecture that this strategy replaces
- **Impact analysis** (`docs/impact-analysis-icm-to-agentmemory.md`) — detailed file-by-file inventory of all 31+ affected files across 6 phases

## Decision

### Overall Strategy: 6-Phase Migration with Dual-Write Safety

Adopt the 6-phase migration strategy defined in the impact analysis (`docs/impact-analysis-icm-to-agentmemory.md`, Section 3) and the Actions+Crystals design doc (`docs/design/actions-crystals-integration.md`, Section 11):

| Phase | Name | Goal | Key Deliverable |
|-------|------|------|-----------------|
| 1 | **Awareness** | Add agentmemory alongside ICM (dual-write). No behavioral change. | Director creates `memory_action_create` + Scribes dual-writes `memory_save` |
| 2 | **Infrastructure** | Replace all ICM tools with agentmemory equivalents. Rename skills. Remove dual-write. | All 8 agents migrated, `agentmemory-knowledge-flow` skill replaces `icm-knowledge-flow`, scripts use agentmemory health checks |
| 3 | **Frontier** | Replace todowrite with `memory_frontier` + `memory_lease`. | Director's delegation loop uses actions/frontier; ADR-0004, ADR-0009 deprecated |
| 4 | **Routines** | Standard flows as `memory_routine_run` templates. | 4 templates: ADR, implement, bugfix, ship |
| 5 | **Crystals + Signals** | Auto-crystallize completed chains. Inter-agent signaling. | `memory_crystallize` + `memory_signal_send/read` operational |
| 6 | **Cleanup** | Verify fallback, remove legacy references, validate light mode. | Zero ICM references remain; `.icm/` archived |

### Phase Dependency Mapping

Each phase has explicit prerequisites that must be stable before execution:

| Phase | Name | Prerequisites | Must Be Stable Before Start | Parallelizable |
|-------|------|--------------|----------------------------|----------------|
| 1 | Awareness | None (initial phase) | N/A — first phase | No (foundation) |
| 2 | Infrastructure | Phase 1 complete (dual-write verified) | Agentmemory daemon (`iii`) running reliably; dual-write data consistent | No (requires Phase 1) |
| 3 | Frontier | Phase 2 complete (agents migrated, ICM removed) | All 8 agents using agentmemory knowledge flow; skill rename done | Partially with 4,5 |
| 4 | Routines | Phase 3 actions model operational | `memory_action_create`/`memory_frontier` working; Director using frontier | Partially with 3,5 |
| 5 | Crystals + Signals | Phase 3 actions + crystals operational | `memory_crystallize` verified; action chains stable | Partially with 3,4 |
| 6 | Cleanup | Phases 1-5 complete | All agentmemory features validated; no critical bugs | No (final phase) |

**Dependency Graph:**

```
Phase 1 (Awareness)
   │
   ▼
Phase 2 (Infrastructure)
   │
   ▼
Phase 3 (Frontier) ───► Phase 4 (Routines) ───► Phase 5 (Crystals + Signals)
   │                                                │
   └────────────────────────────────────────────────┘
   │
   ▼
Phase 6 (Cleanup)
```

**Circular Dependency Analysis:**

The following dependency chains were reviewed for circularity risk:

| Chain | Circular Risk | Mitigation |
|-------|---------------|------------|
| Routines (4) ↔ Signals (5) | **Low.** Routines create actions; signals trigger between actions. No mutual dependency — signals are additive to routines, not required. | Split instantiation: resolve Phase 4 action creation first, add signal wiring in Phase 5. |
| Signals (5) ↔ Crystals (3) | **None.** Crystals compress completed action chains. Signals are used during execution, not for post-hoc compression. | Independent features — can be developed and tested separately. |
| Frontier (3) ↔ Routines (4) | **Low.** Routines create actions that frontier reads. If routines break, frontier has no actions to read. | Phase 4 templates fall back to manual `memory_action_create` if `memory_routine_run` fails. |
| Sentinels (5) → Rollback (6) | **None.** Sentinels are event triggers on actions. Rollback reverts to git state, which removes sentinels. | Rollback safety is independent of sentinel correctness. |

**Transition Checkpoints:**

Before advancing to the next phase, the following must be verified:

| Transition | Verification Gate |
|------------|-------------------|
| Phase 1 → 2 | Dual-write consistency: agentmemory writes match ICM writes for ≥24h. No data divergence. |
| Phase 2 → 3 | All 8 agents migrated. ICM is no longer required. Knowledge flow skill renamed. Gate/ship scripts pass without ICM. |
| Phase 3 → 4 | `memory_frontier` returns correct results for ≥5 real delegation cycles. No fallback to todowrite triggered. |
| Phase 4 → 5 | All 4 routine templates instantiated and executed successfully at least once. |
| Phase 5 → 6 | All signal/sentinel patterns operational for ≥7 days. No signal storms. Crystal extraction tested. |

### Data Migration Plan

#### Dual-Write Strategy (Phase 1)

During Phase 1, all memory writes are dual-targeted:

```
Agent writes → memory_save (agentmemory) + icm_memory_store (ICM)
Agent reads  → memory_recall (agentmemory, primary) with ICM fallback
```

**Write path:**
1. Primary write to agentmemory via `memory_save` / `memory_lesson_save` / `memory_action_create`
2. Secondary write to ICM via `icm_memory_store` / `icm_memoir_add_concept`
3. If primary write succeeds but secondary fails → log warning, continue (agentmemory is source of truth)
4. If primary write fails → log error, attempt ICM-only write as emergency fallback, alert Director

**Read path:**
1. Primary read from agentmemory via `memory_recall` / `memory_lesson_recall`
2. If agentmemory returns results → return them (with confidence score)
3. If agentmemory returns empty → fall back to ICM `icm_memory_recall`
4. If both fail → return empty with warning

#### Historical Data Preservation

Existing ICM data in `.icm/` is handled as follows:

| Data Type | Location | Preservation Strategy |
|-----------|----------|----------------------|
| Memories (store/recall) | `.icm/memory.db` | Read-only archive. No bulk import to agentmemory. Accessible via `icm serve` if running. |
| Memoirs (lessons) | `.icm/memoirs.db` | Read-only archive. New lessons use agentmemory `memory_lesson_save`. |
| Concepts (extracted) | `.icm/concepts.db` | Read-only archive. Agentmemory `memory_patterns` handles future extraction. |
| Session data | `.icm/sessions.db` | Read-only archive. New sessions stored in agentmemory. |
| Consolidation artifacts | `.icm/tier.db` | Read-only archive. Agentmemory auto-consolidates. |

**Why no bulk import:**
1. **Cost/benefit.** Bulk importing thousands of historical memories into agentmemory provides marginal benefit (old context is rarely needed) compared to the engineering effort of writing and validating an ETL pipeline.
2. **Schema mismatch.** ICM's schema (topics, memoirs, concepts) does not map cleanly to agentmemory's (project, tags, types). A migration would require field-level transformation rules for each data type, introducing translation errors.
3. **Dual-source complexity.** If historical data is imported, every query must decide whether to search native agentmemory data, imported ICM data, or both. This doubles query complexity for uncertain benefit.
4. **Archive accessibility.** The `.icm/` directory is preserved as SQLite databases. If a specific historical memory is needed, it can be queried directly with `sqlite3` or by temporarily starting `icm serve`.

#### Validation and Consistency Checks

During Phase 1 dual-write, the following checks run at configurable intervals:

| Check | Frequency | What It Validates | Action on Failure |
|-------|-----------|-------------------|-------------------|
| Write consistency scan | Every 100 writes | For a sample of writes, verify agentmemory contains matching data (by querying both systems) | Log warning, increment `dual_write_mismatch` counter |
| Read fallback rate | Continuous | Track how often agentmemory primary read returns empty (triggering ICM fallback) | If >5% over 1 hour: alert Director, pause migration |
| Data loss scan | Daily | Compare count of memories in agentmemory vs. ICM for the last 24h | If agentmemory count < ICM count by >10%: investigate, potential rollback |
| Tool availability check | Every delegation | Verify `memory_save`, `memory_recall`, `memory_lesson_recall` respond within timeout | If timeout: fall back to ICM, log alert |
| Schema integrity | Weekly | Spot-check 10 random agentmemory entries for correct `project`, `type`, `tags` fields | If >20% malformed: pause migration, fix schema mapping |

#### ICM Phase-Out Timeline

| Phase | ICM Status | Reads | Writes |
|-------|-----------|-------|--------|
| 1 | Running | Dual (agentmemory primary) | Dual (agentmemory + ICM) |
| 2 | Running but unused | No reads (agentmemory only) | No writes (agentmemory only) |
| 3 | Optional | `.icm/` as read-only archive | None |
| 4-5 | Optional | `.icm/` as read-only archive | None |
| 6 | Stopped | `.icm/` directory preserved | None — daemon stopped |

### Implementation Rules

1. **Phases execute sequentially** (1→2→3→6 critical path; 4 and 5 can parallel with 3).
2. **Each phase is independently testable.** No phase breaks existing functionality.
3. **Dual-write is temporary.** Phase 1 adds agentmemory writes alongside ICM. Phase 2 removes ICM writes. Dual-write exists only during Phase 1.
4. **ICM remains installed but unused** after Phase 2. The `.icm/` directory is kept as read-only archive.
5. **No data migration from ICM to agentmemory.** ICM memories remain in `.icm/` for reference. New agentmemory sessions accumulate fresh data.
6. **Rollback:** Any phase can be rolled back by restoring the previous phase's agent files from git. Phase 1's dual-write ensures no data loss during rollback.

### Coexistence Rules

During and after migration:
- Agentmemory daemon (`iii`) is the **primary** memory system
- ICM (`icm serve`) is **optional** — if running, it receives dual-writes in Phase 1; after Phase 2, ICM is not required
- The `.icm/` directory is **read-only archive** — no new writes after Phase 2
- `KODEHOLD_LIGHT=1` uses agentmemory summaries (replacing "ICM summaries")

### Testing Strategy

Each phase has specific test requirements covering unit, integration, and end-to-end verification.

#### Phase 1 (Awareness) Tests

| Test Type | Scope | What It Validates | Pass Criteria |
|-----------|-------|-------------------|---------------|
| Unit | Dual-write function | `memory_save` + `icm_memory_store` both called | Both calls succeed |
| Unit | Read fallback | `memory_recall` empty → `icm_memory_recall` called | Fallback triggers correctly |
| Integration | Dual-write consistency | Same data exists in both systems after write | Data matches within 10s |
| E2E | Full delegation cycle | Director delegates → agent writes to both → Director reads | Cycle completes without error |

**Failure states during Phase 1:**

| State | Detection | Action |
|-------|-----------|--------|
| Agentmemory write fails, ICM write succeeds | Error from `memory_save` | Log warning, continue (ICM has data) |
| ICM write fails, agentmemory write succeeds | Error from `icm_memory_store` | Log warning, continue (agentmemory has data) |
| Both writes fail | Both calls return error | Alert Director, pause delegation |
| Read from agentmemory returns wrong data | Compare result to ICM query | Increment `dual_write_mismatch`, investigate if >1% |

#### Phase 2 (Infrastructure) Tests

| Test Type | Scope | What It Validates | Pass Criteria |
|-----------|-------|-------------------|---------------|
| Unit | SKILL.md tool mapping | All 11+ ICM tool replacements call correct agentmemory tool | Each mapping verified |
| Unit | Agent definition references | All 8 agent files reference "Agentmemory Knowledge Flow" | Zero "ICM" references remain |
| Integration | Skill rename atomicity | `agentmemory-knowledge-flow/` exists, `icm-knowledge-flow/` removed | Directory rename complete |
| Integration | Gate/ship scripts | Scripts pass without ICM running | `scripts/gate.sh` exits 0 with only agentmemory |
| E2E | Full delegation cycle (no ICM) | Agent can complete task using only agentmemory tools | Task completes, no ICM calls made |

**Failure states during Phase 2:**

| State | Detection | Action |
|-------|-----------|--------|
| Agent file misses one ICM reference | CI scan finds "ICM" in agent file | Block merge, fix reference |
| Skill rename incomplete | Both directories exist | Fail gate, delete old directory |
| Gate script fails without ICM | `scripts/gate.sh --validate-only` exits non-zero | Pause Phase 2, fix script, retry |

#### Phase 3 (Frontier) Tests

| Test Type | Scope | What It Validates | Pass Criteria |
|-----------|-------|-------------------|---------------|
| Unit | Action CRUD | Create, update, list, delete actions | All operations return expected results |
| Unit | Frontier ordering | Higher priority/unblocked actions returned first | Frontier order matches priority sort |
| Integration | Lease acquire/release | Exclusive lock acquired and released | No concurrent leases on same action |
| Integration | Frontier + todowrite dual-write | Both systems show same pending work | Match across systems |
| E2E | Full delegation via frontier | Director creates actions → reads frontier → delegates → updates | 5 consecutive cycles without todowrite fallback |

**Failure states during Phase 3:**

| State | Detection | Action |
|-------|-----------|--------|
| Frontier returns blocked action | Action with unsatisfied `requires` in frontier results | Pause, check dependency model |
| Lease fails to acquire | `memory_lease` returns error for available action | Fall back to todowrite, log error |
| Action chain has cycle | `requires` creates circular dependency | Cancel cycle, flag for manual resolution |
| Crystalization produces no digest | `memory_crystallize` returns thin content | Continue, flag for improvement |

#### Phase 4 (Routines) Tests

| Test Type | Scope | What It Validates | Pass Criteria |
|-----------|-------|-------------------|---------------|
| Unit | Template registration | Template stored and retrievable | `memory_routine_run` returns action IDs |
| Unit | Parameter substitution | Parameters correctly applied to actions | Actions have correct descriptions/dependencies |
| Integration | Partial instantiation failure | Error at step 3 of 6 — check steps 1-2 are cleaned up | Steps 1-2 cancelled, no orphaned actions |
| E2E | Full ADR flow via template | 6-step ADR flow completes successfully | All 6 actions created, dependencies correct |

**Failure states during Phase 4:**

| State | Detection | Action |
|-------|-----------|--------|
| Template not registered | `memory_routine_run` returns "template not found" | Fall back to manual action creation |
| Wrong parameter type | Template creates actions with wrong descriptions | Cancel created actions, re-instantiate |
| Version mismatch | Template version outdated | Use fallback, flag template for update |

#### Phase 5 (Crystals + Signals) Tests

| Test Type | Scope | What It Validates | Pass Criteria |
|-----------|-------|-------------------|---------------|
| Unit | Signal send/receive | Agent A sends, Agent B receives | Message content matches |
| Unit | Sentinel trigger | Sentinel fires, gated action unblocks | Action becomes unblocked within TTL |
| Integration | Crystal from action chain | Completed chain → crystal contains narrative + outcomes | Crystal digest is non-empty and accurate |
| Integration | Sketch lifecycle | Create → work → promote/expire | Ephemeral actions isolated from permanent graph |
| E2E | Cross-agent workflow via signals | Design complete → signal reviewers → reviewers act | End-to-end without Director mediation |
| E2E | Signal storm prevention | Single agent sends 200 signals in 5 minutes | Rate-limited or blocked after threshold |

#### Phase 6 (Cleanup) Tests

| Test Type | Scope | What It Validates | Pass Criteria |
|-----------|-------|-------------------|---------------|
| Integration | Zero ICM references | Scan all files for "icm" (case-insensitive) | Zero matches in agent/skill/script/config files |
| Integration | Light mode operation | `KODEHOLD_LIGHT=1` works with agentmemory summaries | No ICM fallback attempted |
| E2E | Full KodeHold workflow | All 6 team types can work using only agentmemory | 10 delegation cycles without ICM |
| Regression | No functional regression | Compare outcomes of same task with agentmemory vs. historical ICM | Equivalent quality results |

#### Global Test Coverage Requirements

| Metric | Minimum Target | Measured By |
|--------|---------------|-------------|
| Phase transition gates | 100% pass before next phase | Gate script execution |
| Unit test coverage (new code) | ≥80% | Per-phase test suite |
| E2E scenarios per phase | ≥3 successful cycles | Automated or manual run log |
| Rollback drills | ≥1 per phase before advancing | Documented rollback exercise |
| Failure state coverage | ≥90% of documented failure states | Test scenario matrix |

#### Rollback vs. Pause vs. Continue Decision Matrix

| Condition | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 | Phase 6 |
|-----------|---------|---------|---------|---------|---------|---------|
| Agentmemory write failure > 3 consecutive | Rollback | Pause | Pause | Pause | Pause | Continue |
| Dual-write mismatch > 1% | Pause | N/A | N/A | N/A | N/A | N/A |
| Read fallback > 5% in 1h | Rollback | Rollback | Pause | Pause | Pause | Investigate |
| Frontier empty with actions | N/A | N/A | Rollback | Pause | Pause | N/A |
| Template instantiation fails | N/A | N/A | N/A | Pause | Continue | N/A |
| Signal storm detected | N/A | N/A | N/A | N/A | Rollback | N/A |
| Single tool timeout | Continue | Continue | Continue | Continue | Continue | Continue |
| Daemon down > 60s | Rollback | Rollback | Rollback | Rollback | Rollback | Investigate |

### Monitoring & Observability

#### Key Metrics

| Metric | Source | What It Tracks | Target |
|--------|--------|---------------|--------|
| `agentmemory_write_latency_ms` | `memory_save` call timing | Time to write to agentmemory | <500ms p95 |
| `agentmemory_read_latency_ms` | `memory_recall` call timing | Time to read from agentmemory | <300ms p95 |
| `dual_write_success_rate` | Phase 1 write tracking | % of dual-writes that succeed on both systems | >99.9% |
| `dual_write_mismatch_count` | Consistency scan | Number of data inconsistencies detected | <0.1% of writes |
| `read_fallback_rate` | Read tracking | % of reads that fall back to ICM | <1% after Phase 2 |
| `frontier_response_time_ms` | `memory_frontier` timing | Time to return frontier results | <200ms p95 |
| `frontier_empty_count` | Frontier result tracking | Count of empty frontier responses when actions exist | 0 |
| `action_create_count` | Action tracking | Actions created per session | Track for trend |
| `lease_acquire_success_rate` | Lease tracking | % of lease acquisitions that succeed | >99% |
| `signal_volume_per_hour` | Signal tracking | Signals sent per agent per hour | <50/agent/hour |
| `crystal_quality_score` | Crystal content analysis | % of crystals with non-empty narrative | >80% |
| `agentmemory_daemon_uptime` | Health check | Agentmemory daemon availability | >99.9% |
| `memory_count_growth_rate` | Agentmemory stats | Rate of memory accumulation | Monitor for bloat |
| `memory_bucket_count` | Agentmemory stats | Number of buckets (internal storage units) | Monitor for fragmentation |
| `migration_phase_duration` | Phase tracking | Time spent in each phase | Track against estimate |

#### Memory Consumption Monitoring

Agentmemory uses storage on disk and potentially RAM. Monitor the following:

| Resource | Warning Threshold | Critical Threshold | Action |
|----------|-------------------|--------------------|--------|
| Agentmemory data directory size | >500MB | >1GB | Investigate bloat, run consolidation |
| Agentmemory daemon RSS (RAM) | >256MB | >512MB | Restart daemon, check for leaks |
| Number of memories | >10,000 | >50,000 | Run consolidation, archive old data |
| Number of actions (incomplete) | >100 | >500 | Clean up stale/abandoned actions |
| Number of signals (unread) | >200 | >1,000 | Reader agent may be stuck, investigate |
| Number of sentinels (active) | >50 | >100 | Sentinel proliferation, review necessity |

**Automated actions:**
- When warning threshold hit: log alert, continue
- When critical threshold hit: trigger consolidation, pause non-essential writes, alert Director
- Memory usage trend is tracked daily; if growth rate exceeds 10%/day, investigate before continuing to next phase

#### Alerting Thresholds

| Alert | Severity | Threshold | Response |
|-------|----------|-----------|----------|
| Dual-write failure | Critical | 3 consecutive write failures | Roll back to Phase 0 |
| Agentmemory daemon down | Critical | 60s of unavailability | Fall back to ICM, restart daemon |
| Read fallback elevated | Warning | >5% over 1h | Investigate agentmemory health |
| High write latency | Warning | >1s p95 over 10 min | Check daemon load, consider restart |
| Frontier returning empty | Warning | Empty while actions exist | Investigate dependency graph |
| Signal storm | Critical | >100 signals/hour from one agent | Disable agent signal permissions |
| Memory bloat | Warning | Data directory >500MB | Schedule consolidation |
| Phase duration exceeded | Info | Phase taking >2x estimated sessions | Review blockers, adjust plan |
| Tool permission missing | Critical | `memory_*` tool denied at runtime | Update opencode.json immediately |

#### Transition-Period Dashboard

During the migration (Phases 1-5), the following dashboard should be available for monitoring:

```
┌─────────────────────────────────────────────────┐
│  KodeHold Migration Dashboard — Phase N          │
├─────────────────────────────────────────────────┤
│  Agentmemory:  ● UP (99.9% uptime, 4.2d)        │
│  ICM:          ○ RUNNING (Phase 1 only)          │
│                 ● ARCHIVE (Phase 2+)             │
├─────────────────────────────────────────────────┤
│  Dual-Write Consistency:   99.97% (12 mismatches │
│                            out of 42,341 writes) │
│  Read Fallback Rate:       0.3% (Phase 1 only)   │
│  Average Write Latency:    142ms p95             │
│  Average Read Latency:      87ms p95             │
├─────────────────────────────────────────────────┤
│  Memory Count:             3,847 (agentmemory)   │
│                            12,401 (ICM archive)  │
│  Actions (active):         23                    │
│  Signals (unread):         5                     │
│  Sentinels (active):       3                     │
├─────────────────────────────────────────────────┤
│  Phase Progress:                                  │
│  ████████████░░░░░░░░░░  48%                     │
│                                                   │
│  Phase 1: ✓ Complete (2 sessions)                 │
│  Phase 2: ✓ Complete (3 sessions)                 │
│  Phase 3: ▸ In progress (session 2 of 3)          │
│  Phase 4: ☐ Pending                              │
│  Phase 5: ☐ Pending                              │
│  Phase 6: ☐ Pending                              │
└─────────────────────────────────────────────────┘
```

The dashboard is checked:
- At the start of each session (Director reads metrics)
- After each phase transition
- On alert trigger (automated notification to Director)
- Periodically via cron (optional — every 6 hours collects and logs metrics)

#### Data Collection Method

Metrics are collected via agentmemory's own tooling:

| Metric | Collection Method | Implementation |
|--------|-------------------|----------------|
| Latency | Client-side timing | Wrap each agentmemory call with `Date.now()` |
| Success/failure | Call return status | Count successes vs. failures per call type |
| Data sizes | `memory_diagnose` | Periodic health check |
| Memory count | Agentmemory stats API | Daily snapshot |
| Dual-write consistency | Custom scan script | 1% sample of writes verified against both systems |
| Signal volume | `memory_signal_read` (all agents) | Hourly count |
| Daemon uptime | Health endpoint ping | Every minute |

Metrics are stored as agentmemory memories (type="metric") for traceability and trend analysis.

### What This Changes

- **Agent files (8):** Replace `icm *` commands with `memory_*` MCP tools. Replace "ICM Knowledge Flow" references with "Agentmemory Knowledge Flow".
- **Skills (2):** Rename `icm-knowledge-flow` → `agentmemory-knowledge-flow`. Update `investigate/SKILL.md`.
- **Scripts (4):** Replace ICM health checks with agentmemory equivalents. Deprecate consolidate-all.sh (agentmemory auto-consolidates).
- **ADRs (9):** Deprecate ADR-0004, ADR-0009, ADR-0027. Supersede ADR-0019, ADR-0021. Re-evaluate ADR-0025.
- **Design doc:** Update Section 7.2 (ICM → Agentmemory). Bump to v2.0.0.

## Consequences

### Positive

1. **Unified memory system.** One system to learn, maintain, and debug. No dual-maintenance burden.
2. **Better tooling.** Agentmemory provides semantic search, lessons, patterns, provenance — capabilities ICM lacked.
3. **Orchestration layer unlocked.** Actions, crystals, routines, signals are only available in agentmemory. Migration is a prerequisite for those capabilities.
4. **No data loss.** Dual-write in Phase 1 ensures ICM data remains intact. `.icm/` directory stays as read-only archive.
5. **Gradual adoption.** 6 phases mean teams can adapt incrementally. No "big bang" migration.
6. **Rollback safety.** Each phase is independently revertible via git.

### Negative

1. **31+ files must change.** Every agent definition, skill, script, and config file with ICM references needs updating. This is a large, cross-cutting change.
2. **ICM legacy data becomes cold storage.** Old memories in `.icm/` are not indexed by agentmemory. They remain available as SQLite databases but are not searchable via agentmemory tools.
3. **Agentmemory daemon dependency.** ICM could run as a CLI tool (`icm serve`). Agentmemory requires the daemon (`iii`) to be running. This is an operational constraint.
4. **Tool mapping not 1:1.** `icm_memory_list_topics` has no direct equivalent. Some ICM patterns (topic-scoped queries) must adapt to agentmemory's `project`-scoped model.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Director delegation regression** — `memory_frontier` returns unexpected results | Medium | Critical | Phase 1 dual-writes actions + todowrite. Fallback to todowrite if frontier fails. |
| 2 | **Scribes migration scope** — ~55 ICM references in a single file | Medium | Medium | Split scribes.md into sub-tasks (Knowledge Flow, Database, Workflows, CRUD). |
| 3 | **Script breakage** — gate.sh/ship.sh fail without ICM | Medium | High | Phase 2 adds agentmemory health check before removing ICM check. |
| 4 | **Permission gaps** — `"icm *": allow` removed but `"memory_*": allow` not in opencode.json | Low | High | Update opencode.json in Phase 2 before removing ICM references. |
| 5 | **Phase overrun** — 6 phases estimated at 6-8 sessions | Medium | Low | Each phase is independently deliverable. Ship partial progress per phase. |

### Follow-up Items

- [ ] Create ADR-0030 (Agentmemory Knowledge Flow) — Phase 2 prerequisite
- [ ] Create ADR-0031 (Actions + Crystals) — Phase 3 prerequisite
- [ ] Create ADR-0032 (Routine Templates) — Phase 4 prerequisite
- [ ] Create ADR-0033 (Signals + Sentinels) — Phase 5 prerequisite
- [ ] Deprecate ADR-0004, ADR-0009, ADR-0027
- [ ] Supersede ADR-0019, ADR-0021
- [ ] Update design doc to v2.0.0
- [ ] Rewrite consolidate-all.sh or deprecate
- [ ] Build migration dashboard script — collects and displays monitoring metrics per session
- [ ] Write consistency scan script — validates dual-write data integrity in Phase 1
- [ ] Document rollback drill procedures — one drill per phase before advancing
- [ ] Define alert notification channel — Director signal or external notification on critical alerts
- [ ] Create per-phase test harness — automated test suite for each phase's test matrix

### Rollback Strategy

Rollback decisions are determined by which phase is active and what failure mode is detected.

#### Failure Detection Criteria

| Signal | Detects | Severity | Recommended Action |
|--------|---------|----------|-------------------|
| `dual_write_mismatch > 1%` | Data inconsistency between ICM and agentmemory | High | Pause Phase 1, investigate mapping, continue if fixable |
| `read_fallback_rate > 5%` | Agentmemory primary reads failing too often | Critical | Roll back to Phase 0 (ICM-only) if in Phase 1; pause if in Phase 2+ |
| `agentmemory_daemon_down > 60s` | Agentmemory daemon (`iii`) unavailable | Critical | Fall back to ICM immediately (any phase) |
| `frontier_empty_for_N_delegations` | Frontier returns no actions when actions exist | High | Fall back to todowrite, flag for investigation |
| `action_consistency_error` | Action dependency has invalid requires reference | Medium | Cancel action chain, recreate manually |
| `signal_storm_detected` | >100 signals/hour from single agent | Medium | Disable signal permissions for that agent, investigate |
| `crystal_empty_for_N_chains` | Crystallization produces no useful digest | Low | Continue, log for improvement |

#### Rollback Steps Per Phase

**Phase 0 (Pre-migration — no changes applied yet):**
- No rollback needed. ICM is the only memory system.

**Phase 1 (Awareness — dual-write active):**
- **Safe to rollback?** Yes — 100% safe. ICM has complete data.
- **Steps:**
  1. Remove `memory_*` calls from agent files (restore from git)
  2. Stop agentmemory daemon (`iii`) if no other services depend on it
  3. Verify ICM reads/writes work as before migration
  4. Delete any agentmemory data created during Phase 1 (optional — data is stale but harmless)
- **Detection:** Dual-write mismatch >1% or agentmemory unavailability.
- **Revert command:** `git restore .opencode/agents/director.md .opencode/agents/scribes.md`

**Phase 2 (Infrastructure — ICM replaced):**
- **Safe to rollback?** Conditional — safe if the agentmemory daemon is still functional and ICM tools are still available.
- **Steps:**
  1. Restore old `icm-knowledge-flow/` skill directory: `git restore .opencode/skills/icm-knowledge-flow/`
  2. Revert all 8 agent files to reference ICM knowledge flow: `git restore .opencode/agents/`
  3. Reactivate ICM health checks in gate.sh/ship.sh: `git restore scripts/gate.sh scripts/ship.sh`
  4. Verify ICM daemon starts and responds: `icm serve --check-health`
  5. Test one delegation cycle with full ICM path
- **Detection:** Agentmemory tool failures on >3 consecutive delegations, or knowledge flow skill returns errors.
- **Max safe rollback window:** Until Phase 3 changes are applied. Once Phase 3 migrations start, Phase 2 rollback requires restoring the original (non-action) delegation logic.

**Phase 3 (Frontier — actions replace todowrite):**
- **Safe to rollback?** Yes — todowrite was never removed. Director switches back to manual mode.
- **Steps:**
  1. Stop calling `memory_frontier` in Director's delegation loop
  2. Restore "Todo Sequence Protocol" section in director.md: `git restore .opencode/agents/director.md`
  3. Existing actions in agentmemory become orphaned but cause no harm
  4. Reactivate ADR-0004, ADR-0009, ADR-0021
- **Detection:** `memory_frontier` returns empty when actions exist, or frontier returns unexpected results for ≥2 cycles.
- **Max safe rollback window:** Until Phase 4 templates are in use. Active action chains are lost on hard rollback.

**Phase 4 (Routines):**
- **Safe to rollback?** Yes — fall back to manual `memory_action_create` per action.
- **Steps:**
  1. Stop calling `memory_routine_run` in Director's delegation loop
  2. Fall back to Phase 3-style manual action creation
  3. Templates remain registered but unused
  4. This ADR becomes Deprecated
- **Detection:** Template instantiation fails >50% of the time, or template DAGs don't match actual workflows.

**Phase 5 (Crystals + Signals):**
- **Safe to rollback?** Yes — remove signal permissions, stop crystalization.
- **Steps:**
  1. Remove `memory_signal_send`/`memory_signal_read` from all agent tool permissions
  2. Remove `memory_crystallize` from Director's post-completion flow
  3. Revert to Director-mediated cross-team communication
  4. ADR-0033 becomes Deprecated
- **Detection:** Signal storms detected, or signal routing causes delegation errors.

**Phase 6 (Cleanup):**
- **Safe to rollback?** No — this is the terminal phase. ICM has been fully removed.
- **Steps:**
  1. Full restore from git: revert all migration commits since Phase 1
  2. Reinstall ICM daemon if uninstalled
  3. Reactivate all deprecated ADRs
  4. Verify full ICM pipeline with test delegation

#### Contingency Measures (Minimum Through Phase 2)

| Measure | Phase 1 | Phase 2 | Phase 3+ |
|---------|---------|---------|----------|
| ICM daemon remains running | Required | Recommended | Optional |
| Dual-write to ICM | Active | Stopped | Not available |
| Git revert of agent files | Single file restore | All agents + scripts | Agents + ADRs |
| Todowrite fallback | N/A (not yet migrated) | N/A | Available |
| Emergency stop | Remove `memory_*` calls | Restore icm-knowledge-flow | Restore todowrite |

**Phase 2 contingency guarantee:** If agentmemory fails critically during Phase 2, the following sequence restores full ICM operation within 5 minutes:
1. `git restore .opencode/skills/icm-knowledge-flow/` — restore old skill
2. `git restore .opencode/agents/` — restore all 8 agent definitions
3. `git restore scripts/gate.sh scripts/ship.sh` — restore health checks
4. `icm serve` — restart ICM daemon
5. Verify: one delegation cycle

After Phase 2, ICM can still be restored but the `.icm/` database will lack data that was written only to agentmemory during Phases 1-2 dual-write.

## ADR References

- **ADR-0004** (ICM and RTK Integration) — foundational ICM ADR; to be deprecated by this migration
- **ADR-0009** (ICM MCP Integration) — layered ICM architecture; to be deprecated
- **ADR-0027** (ICM Knowledge Flow Invocation Modes) — to be replaced by ADR-0030
- **ADR-0028** (Agentmemory Project Detection) — established full-path project names, removing the last blocker for agentmemory migration
- **ADR-0030** (Agentmemory Knowledge Flow) — replaces ADR-0027, defines 3-mode knowledge flow using `memory_*` tools
- **ADR-0031** (Actions + Crystals for Director Delegation) — Phase 3 implementation
- **ADR-0032** (Routine Templates) — Phase 4 implementation
- **ADR-0033** (Inter-Agent Signals + Sentinels) — Phase 5 implementation
- **Impact analysis** (`docs/impact-analysis-icm-to-agentmemory.md`) — file inventory, tool mapping, risk assessment
- **Actions+Crystals design doc** (`docs/design/actions-crystals-integration.md`) — 6-phase migration plan
- **Design doc** (`docs/design/README.md`) — Section 7.2 to be updated for agentmemory

### Source Files Referenced

- All `.opencode/agents/*.md` files (8 agent definitions with ICM references)
- `.opencode/skills/icm-knowledge-flow/SKILL.md` (skill to be renamed/replaced)
- `.opencode/skills/investigate/SKILL.md` (3 ICM references to migrate)
- `scripts/gate.sh` (ICM health checks)
- `scripts/ship.sh` (ICM checks in shipping gate)
- `scripts/benchmark.sh` (ICM benchmarks)
- `scripts/consolidate-all.sh` (ICM consolidation)
- `opencode.json` (ICM bash permissions)
- `.opencode/references/kodehold-protocol.md` (ICM topic convention)
- `AGENTS.md` (ICM references in quick reference)
- `docs/impact-analysis-icm-to-agentmemory.md` (this analysis)
- `docs/design/actions-crystals-integration.md` (migration design)
