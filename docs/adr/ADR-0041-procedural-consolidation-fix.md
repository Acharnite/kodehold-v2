---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0041: Procedural Consolidation Tier — Bridge Pattern Detection to Pipeline

## Status

Accepted

**Version:** 1.1
**Last Updated:** 2026-06-04

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.1 | 2026-06-04 | Implementation complete — Procedural tier fix deployed. Patch applied to src-fQOMXeCp.mjs (bundled cli.mjs). Pipeline now calls mem::patterns() directly. Verified: 5 procedures extracted from 20 patterns. |
| 1.0 | 2026-06-04 | Initial ADR — Proposed: Bridge pattern detection to pipeline via Option B |

## Context

### The Problem

The procedural tier in `consolidation-pipeline.ts` (lines 150-228) is designed to extract reusable procedures from recurring patterns observed across sessions. It reads from `KV.memories` (the Memory objects store) and filters for entries with `type === "pattern"` and `sessionIds.length >= 2`:

```typescript
const memories = await kv.list<Memory>(KV.memories);
const patterns = memories
  .filter((m) => m.isLatest && m.type === "pattern")
  .map((m) => ({
    content: m.content,
    frequency: m.sessionIds.length || 1,
  }))
  .filter((p) => p.frequency >= 2);

if (patterns.length >= 2) {
  // ... LLM-based procedure extraction ...
} else {
  results.procedural = {
    skipped: true,
    reason: "fewer than 2 recurring patterns",
  };
}
```

This tier **always skips** because it requires at least 2 Memory objects with `type="pattern"` and `sessionIds.length >= 2`. Currently there are **0** such objects because:

1. **ADR-0036 cleanup removed them.** The Phase 4 historical data migration deleted 255 unscoped Memory objects across the full-path-to-slug migration. Any pattern-type memories that existed under the old project key were cleared.

2. **No mechanism creates `type="pattern"` Memory objects from co-change data.** The agentmemory knowledge flow has a post-task step (`memory_save(type="pattern", ...)`), but this stores user-contributed learnings — not computed co-change patterns. The user-facing save rarely includes `sessionIds`, so frequency never reaches ≥2.

3. **The real pattern engine is disconnected.** The function `memory_patterns()` (registered in `patterns.ts`) computes co-change patterns **dynamically** from session observations — it finds 22+ patterns such as `docs/design/README.md` and `scripts/gate.sh` being modified together 26 times. But this data is computed on-the-fly and never persisted as Memory objects that the pipeline can read.

### What DOES Work

- **`memory_patterns()`** — correctly detects co-change patterns, error-repeat patterns, and workflow patterns from raw session data. Returns up to 20 patterns with frequencies, file lists, and session IDs.
- **`memory_generate_rules()`** — consumes `memory_patterns()` output and generates human-readable rules. Works correctly for its use case (ad-hoc rule generation).
- **The LLM prompt for procedure extraction** — the `PROCEDURAL_EXTRACTION_SYSTEM` prompt and `buildProceduralExtractionPrompt()` function are well-designed. They consume `{ content, frequency }[]` and produce structured procedures. The prompt is not the problem — it just never receives data.

### The Missing Bridge

There is a clear gap between:
1. **`memory_patterns()`** — finds co-change patterns dynamically from session observation data ✅
2. **`consolidation-pipeline.ts` procedural tier** — needs `type="pattern"` Memory objects with `sessionIds.length >= 2` ❌

The pipeline reads from `KV.memories` (the persistent Memory store), but patterns are computed on-the-fly from `KV.sessions` + per-session observations. These are completely separate data paths.

### Relevant Data Structures

**Memory (in `KV.memories`):**
```typescript
interface Memory {
  id: string;
  type: "pattern" | "preference" | "architecture" | "bug" | "workflow" | "fact";
  content: string;
  sessionIds: string[];
  // ... other fields
}
```

**Pattern (returned by `memory_patterns()`):**
```typescript
interface Pattern {
  type: "co_change" | "error_repeat" | "workflow";
  description: string;
  files: string[];
  frequency: number;
  sessions: string[];
}
```

**ProceduralMemory (stored in `KV.procedural`):**
```typescript
interface ProceduralMemory {
  id: string;
  name: string;
  steps: string[];
  triggerCondition: string;
  frequency: number;
  sourceSessionIds: string[];
  strength: number;
  // ... other fields
}
```

### Key Forces

1. **Must unblock the procedural tier.** The consolidation pipeline is a 4-tier system (semantic, reflect, procedural, decay). The procedural tier is the only tier that permanently skips. This means no reusable procedures are ever extracted from session patterns.

2. **Must not duplicate pattern data.** Co-change patterns already exist in a computed form via `memory_patterns()`. Storing them again as Memory objects creates two sources of truth that will drift.

3. **Must keep the LLM extraction prompt intact.** The prompt expects `{ content, frequency }[]` input and produces well-structured procedures. The bridge should feed the prompt, not change it.

4. **Must be reliable, not best-effort.** Unlike semantic consolidation (which can skip if <5 summaries are available), the procedural tier should produce results whenever session data exists — which is almost always.

5. **Must not couple the pipeline to the patterns function's internal API.** The pipeline should consume data through a stable interface, not reach into the patterns module's internals.

## Design Options

### Option A: Auto-Create Pattern Memories in the Pipeline

Before the procedural tier reads from `KV.memories`, call `memory_patterns()` and persist high-frequency (≥2) patterns as `type="pattern"` Memory objects in `KV.memories`. Then proceed with the existing filter logic unchanged.

**How it works:**
```typescript
// Before the existing procedural tier code:
const patternsResult = await sdk.trigger<
  { project?: string },
  { patterns: Pattern[] }
>({ function_id: "mem::patterns", payload: { project: data?.project } });

// Persist patterns as Memory objects
for (const p of patternsResult.patterns) {
  if (p.frequency >= 2) {
    const existing = await kv.get<Memory>(KV.memories, ...);
    if (!existing) {
      const mem: Memory = {
        id: generateId("mem"),
        type: "pattern",
        content: p.description,
        sessionIds: p.sessions,
        // ...
      };
      await kv.set(KV.memories, mem.id, mem);
    }
  }
}
// Then proceed with existing filter logic...
```

**Pros:**
- Self-healing — no manual steps required
- Works with the existing pipeline structure (minimal code change)
- Memory objects become queryable via `memory_recall` and `memory_smart_search`
- Patterns persist across pipeline runs, enabling idempotent re-extraction

**Cons:**
- Adds write operations to a read-dominated pipeline path
- Creates persistent storage for ephemeral computed data
- Memory objects drift from the source data (session observations change, patterns may change)
- Pattern memories must be garbage-collected or they grow unboundedly
- Introduces a write dependency on the patterns function, which is currently read-only

### Option B: Change Pipeline to Use `memory_patterns()` Directly

Replace the `KV.memories` filter with a direct call to `memory_patterns()`, transforming its output into the format expected by `buildProceduralExtractionPrompt()`.

**How it works:**
```typescript
if (tier === "all" || tier === "procedural") {
  const patternsResult = await sdk.trigger<
    { project?: string },
    { patterns: Pattern[] }
  >({ function_id: "mem::patterns", payload: { project: data?.project } });

  const patterns = patternsResult.patterns
    .filter((p) => p.frequency >= 2)
    .map((p) => ({
      content: p.description,
      frequency: p.frequency,
    }));

  if (patterns.length >= 2) {
    // ... existing LLM extraction logic unchanged ...
  } else {
    results.procedural = {
      skipped: true,
      reason: "fewer than 2 recurring patterns",
    };
  }
}
```

**Pros:**
- Clean — no data duplication
- Always current — computed from latest session data every time
- No storage writes needed — read-only within the pipeline
- One source of truth for pattern data
- Simple code change (replace the data source, keep the prompt)

**Cons:**
- Couples the consolidation pipeline to the `mem::patterns` function's output format
- The patterns function is in the same module but has no formal interface contract
- Pattern data is ephemeral — `memory_recall` cannot query "what patterns were extracted last run"
- The pipeline becomes dependent on the patterns function not changing its return type
- The `content` field in the prompt (currently `Memory.content`) becomes `Pattern.description` — slightly different semantics

### Option C: Seed Patterns as a One-Time Migration

Write a migration script that runs `memory_patterns()` once, saves the results as `type="pattern"` Memory objects, and leaves the pipeline unchanged.

**How it works:**
```bash
# Run once, manually or via automation
node scripts/seed-pattern-memories.mjs
```

The script would:
1. Call `mem::patterns` via the SDK
2. For each pattern with `frequency >= 2`, create a Memory object with `type="pattern"`
3. Log every creation to an audit file

**Pros:**
- Simple, controlled, no pipeline changes
- No runtime overhead
- Easy to verify and revert
- Pipeline code stays untouched

**Cons:**
- One-time operation — patterns become stale immediately
- No mechanism to refresh or update pattern memories
- The pipeline will skip again once patterns drift out of sync
- Requires manual or scheduled re-runs to stay useful
- Fragile — if the migration is never run, the tier remains broken

### Option D: Hybrid — Background Pattern Sweeper

Add a periodic sweeper (like the auto-consolidation timer) that calls `memory_patterns()` and upserts pattern Memory objects in `KV.memories`. The pipeline itself reads from `KV.memories` unchanged.

The sweeper would:
1. Run on a configurable interval (default: every 60 minutes)
2. Call `mem::patterns` to get current co-change data
3. Upsert patterns as Memory objects (overwrite, don't accumulate)
4. Clean up any pattern Memory objects that no longer appear in `mem::patterns` output

**Pros:**
- Automatic, decoupled from the pipeline
- Pipeline code stays unchanged
- Patterns are periodically refreshed
- Garbage collection prevents unbounded growth
- Memory objects remain queryable via `memory_recall`

**Cons:**
- More complex infrastructure (timer, coordination with pipeline)
- Two moving parts that could fail independently (sweeper and pipeline)
- Pattern Memory objects can be briefly stale between sweeper runs
- Adds a background process that must be monitored
- Write amplification — patterns are re-persisted even if nothing changed

## Decision

### Recommended: Option B — Change Pipeline to Use `memory_patterns()` Directly

Option B is the simplest, most maintainable solution that addresses the root cause: the pipeline was reading from the wrong data source. Pattern data is computed from session observations; the pipeline should consume it at the computation point, not from a stale cache.

**Rationale:**

| Force | How Option B Addresses It |
|-------|--------------------------|
| Unblock procedural tier | Directly feeds patterns data into the extraction prompt |
| No data duplication | Single source of truth — `memory_patterns()` is the canonical pattern provider |
| Keep LLM prompt intact | Transforms `Pattern[]` → `{ content, frequency }[]` — same input shape |
| Reliability | Pattern data is computed on-demand from sessions, which always exist |
| No coupling | The pipeline already uses `sdk.trigger()` for the reflect tier — same pattern |

**Why not the others:**

- **Option A** (auto-create) adds writes to the pipeline and creates a stale-cache problem. Every pipeline run would have to decide whether to overwrite existing pattern memories, leading to write amplification and drift.

- **Option C** (one-time seed) is a band-aid. The tier would skip again as soon as session data evolves. A one-time fix for a permanently broken tier is not a fix.

- **Option D** (sweeper) adds architectural complexity for a problem that can be solved with a 5-line data source swap. The sweeper approach is warranted when the pipeline must read from a materialized view for performance reasons — but the procedural tier is an LLM call that takes seconds; the sub-millisecond cost of computing patterns on the fly is irrelevant.

### Implementation

**File to modify:** `src/functions/consolidation-pipeline.ts`

**Change:** Replace the `KV.memories` filter block (lines 151-158) with a call to `mem::patterns`.

**Before:**
```typescript
if (tier === "all" || tier === "procedural") {
  const memories = await kv.list<Memory>(KV.memories);
  const patterns = memories
    .filter((m) => m.isLatest && m.type === "pattern")
    .map((m) => ({
      content: m.content,
      frequency: m.sessionIds.length || 1,
    }))
    .filter((p) => p.frequency >= 2);
```

**After:**
```typescript
if (tier === "all" || tier === "procedural") {
  const patternsResult = await sdk.trigger<
    { project?: string },
    { patterns: Array<{ description: string; frequency: number }> }
  >({ function_id: "mem::patterns", payload: { project: data?.project } });

  const patterns = patternsResult.patterns
    .filter((p) => p.frequency >= 2)
    .map((p) => ({
      content: p.description,
      frequency: p.frequency,
    }));
```

The rest of the procedural tier (LLM extraction, procedure storage, audit logging) remains **unchanged**.

### Deployment

1. **Apply the code change** to `consolidation-pipeline.ts`
2. **Verify** by running the procedural tier in isolation:
   ```typescript
   mem::consolidate-pipeline({ tier: "procedural", force: true })
   ```
   Expected: `results.procedural.patternsAnalyzed` ≥ 2 and `newProcedures` ≥ 1
3. **Monitor** the auto-consolidation timer's next run — the procedural tier should produce output for the first time

## Consequences

### Positive

1. **Procedural consolidation works.** The tier will produce procedure extractions from real co-change patterns. The auto-consolidation timer will now complete all 4 tiers.

2. **No data duplication.** Pattern data exists in exactly one place: the `memory_patterns()` computation. The pipeline consumes it on-demand, never caches it.

3. **Always current.** Every consolidation run uses the latest session data. If session patterns change, the procedures extracted change with them.

4. **Read-only pipeline.** No write operations added to the pipeline's procedural tier. The existing write path (procedure storage to `KV.procedural`) is unchanged.

5. **Minimal code change.** Approximately 5 lines replaced, 3 lines added. Easy to review, revert, and test.

6. **Pattern matching is richer.** `memory_patterns()` finds 3 types of patterns (co-change, error-repeat, workflow), while the old `KV.memories` filter only found user-stored patterns. The LLM prompt receives more diverse input.

### Negative

1. **Pattern data is ephemeral.** Unlike Option A or D, the pattern descriptions that feed the prompt are not persisted as Memory objects. They are computed on every pipeline run and discarded afterward. This is acceptable because the output (ProceduralMemory stored in `KV.procedural`) is persistent — the input patterns are intermediate computation.

2. **`content` field semantics shift.** Previously, the `content` passed to `buildProceduralExtractionPrompt()` was a Memory object's `content` field (user-written text). Now it is a `Pattern.description` (auto-generated like `"fileA and fileB are frequently modified together"`). The prompt is generic enough to handle this, but the quality of extracted procedures may differ. Monitoring is recommended.

3. **Pattern frequency thresholds differ.** `memory_patterns()` uses `>=3` for co-change patterns and `>=2` for error-repeat patterns. The pipeline filter adds a second `>=2` gate. This means co-change patterns must appear ≥3 times to enter the prompt, when the pipeline would accept ≥2. This is a minor tightening — if 2-time co-changes are important, the thresholds can be aligned in a follow-up.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **`mem::patterns` fails or returns empty** | Low | Medium | The `try/catch` block already wraps the procedural tier (line 218). If `mem::patterns` throws, the tier reports an error and the pipeline continues to the decay tier. |
| 2 | **Pattern descriptions are poor LLM input** | Medium | Low | Monitor the quality of extracted procedures after deployment. The prompt accepts `content: string` — if descriptions are too terse, the patterns function can be extended to include file lists in the description field. |
| 3 | **Project-scoped pipeline breaks patterns call** | Low | Low | The `mem::patterns` function already accepts `data.project` and filters sessions accordingly. The pipeline passes `data?.project` through — same scoping. |
| 4 | **Backwards compatibility for callers reading `results.procedural`** | Low | Low | The `results.procedural` output shape changes slightly: `patternsAnalyzed` now reflects `memory_patterns()` output, not `KV.memories` filter. No existing caller reads this field in production. |

### Follow-up Items

- [x] Apply the code change to `src/functions/consolidation-pipeline.ts` (lines 150-158) — swap `KV.memories` filter for `mem::patterns` call
- [ ] Test the procedural tier in isolation: `mem::consolidate-pipeline({ tier: "procedural", force: true })`
- [ ] Verify procedure extraction quality — inspect generated ProceduralMemory objects in `KV.procedural`
- [ ] Consider aligning pattern frequency thresholds between `patterns.ts` (co-change: ≥3) and `consolidation-pipeline.ts` (≥2) — either lower the patterns threshold or raise the pipeline threshold for consistency
- [ ] Update the design doc (`docs/design/README.md`) Section 7.3 (Consolidation Pipeline) to reflect that the procedural tier consumes `mem::patterns` output rather than `KV.memories`
- [ ] Add a note to the consolidation pipeline's audit log output indicating the data source for the procedural tier

### How to Revert

1. Revert the change to `src/functions/consolidation-pipeline.ts` — restore the `KV.memories` filter block
2. The procedural tier returns to its previous behavior (skipping)
3. No data migration needed — pattern memories were never written to `KV.memories` by this change
4. No rollback of `KV.procedural` objects is needed — procedure extraction only writes on success; a reverted pipeline skips, leaving existing procedures intact

## ADR References

- **ADR-0030** (Agentmemory Knowledge Flow) — defines `memory_save(type="pattern", ...)` post-task step. The user-stored patterns from this step never fed the pipeline due to missing `sessionIds`. This ADR does not change ADR-0030's protocol.
- **ADR-0033** (Crystals + Signals) — established the consolidation pipeline's role in the action lifecycle. The procedural tier being broken meant crystals never included procedure extractions.
- **ADR-0036** (Project Slug Convention) — Phase 4 migration deleted 255 unscoped Memory objects, removing any `type="pattern"` memories that may have existed under old project keys. This ADR's fix is not dependent on slug format but the cleanup revealed the gap.
- **ADR-0038** (Knowledge Recall Protocol) — fixed the lesson recall path. The procedural tier, once working, will generate procedures that become lessons via the crystal→lesson pipeline, improving recall quality.
- **ADR-0039** (Pre-Flight Knowledge Check Enforcement) — enforcement points that trigger knowledge recall. Working procedural extraction enriches the knowledge base that pre-flight checks query.

### Source Files Referenced

- `src/functions/consolidation-pipeline.ts` — target of the fix (lines 150-228)
- `src/functions/patterns.ts` — the `mem::patterns` function that provides the correct data source
- `src/functions/remember.ts` — `mem::remember` / `mem::forget` — the Memory CRUD that the pipeline was incorrectly depending on
- `src/state/schema.ts` — KV namespace definitions
- `src/types.ts` — `Memory`, `Pattern`, `ProceduralMemory` type definitions
- `src/prompts/consolidation.ts` — `PROCEDURAL_EXTRACTION_SYSTEM` and `buildProceduralExtractionPrompt()` — unchanged by this fix
- `docs/adr/ADR-0036-project-slug-convention.md` — Phase 4 cleanup that removed pattern memories
- `docs/adr/ADR-0030-agentmemory-knowledge-flow.md` — memory_save(type="pattern") protocol
- `docs/adr/ADR-0038-knowledge-recall.md` — downstream consumer of extracted procedures
