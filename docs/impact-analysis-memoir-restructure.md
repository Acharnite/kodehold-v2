# Impact Analysis: ICM Memoir Restructure

**Project:** KodeHold
**Current State:** CLOSED (v0.16.0)
**Target State:** ACTIVE (for config updates)
**Author:** Architects
**Date:** 2026-05-30

---

## 1. Summary of Changes

### What changed in the data layer (already complete)

The ICM memoir structure was consolidated to eliminate fragmentation between shared learnings and team-specific learnings:

| Before | After | Status |
|--------|-------|--------|
| `kodehold` + `learnings` (separate memoirs) | `kodehold-learnings` (63 concepts, 68 links) | ✅ Merged |
| `kodehold-<team>-learnings` (6 per-team learnings memoirs) | **Deleted** — content merged into `kodehold-learnings` | ✅ Deprecated |
| `context-kodehold` topic (201 entries) | Distilled to 1 consolidated entry | ✅ Done |
| `kodehold-arch` (architecture concepts) | Still exists (3 concepts) | ⚠️ Still referenced |
| `kodehold-<team>` (team knowledge memoirs) | Still exist individually | ⚠️ Naming convention needs alignment |
| `kodehold-patterns` | **Never created** — referenced but doesn't exist | ⚠️ Dead reference |

### What the ICM memoir tree looks like now

```
Memoirs (12 total):
├── kodehold-learnings (63 concepts, 68 links)  ← THE merged learnings memoir
├── kodehold-arch (3 concepts)                   ← Architectural concepts (legacy name)
├── kodehold-architects (3 concepts)
├── kodehold-engineers (2 concepts)
├── kodehold-fls (1 concept)
├── kodehold-reviewers (3 concepts)
├── kodehold-scribes (4 concepts)
├── kodehold-testers (11 concepts)
├── lifecycle (8 concepts)
├── principles (8 concepts)
├── teams (7 concepts, 8 links)
└── tools (7 concepts)
```

### Why this matters

The old Knowledge Flow protocol (ADR-0027) instructed teams to search `<team>-learnings` memoirs (Step 2) and store to `kodehold-<project>-<topic>-<team>-learnings` topics (Step 7). Neither of these targets exist anymore. If agents follow their current instructions, Step 2 searches will return empty results and Step 7 stores will go to orphaned topics. All 6 agent files, the SKILL.md, 3 ADRs, scribes.md, and the protocol reference contain stale memoir references that must be updated before teams can operate correctly.

---

## 2. Config File Impact

### 2.1 Agent Files (6 files)

Each agent file has a Pre-task ICM Knowledge Flow section with a stale reference to `kodehold-<team>-learnings`:

| File | Line | Old Reference | New Reference Needed |
|------|------|---------------|---------------------|
| `.opencode/agents/architects.md` | 75 | `kodehold-architects-learnings` | `kodehold-teams` or `kodehold-architects` |
| `.opencode/agents/engineers.md` | 45 | `kodehold-engineers-learnings` | `kodehold-teams` or `kodehold-engineers` |
| `.opencode/agents/testers.md` | 44 | `kodehold-testers-learnings` | `kodehold-teams` or `kodehold-testers` |
| `.opencode/agents/reviewers.md` | 66 | `kodehold-reviewers-learnings` | `kodehold-teams` or `kodehold-reviewers` |
| `.opencode/agents/fls.md` | 65 | `kodehold-fls-learnings` | `kodehold-teams` or `kodehold-fls` |
| `.opencode/agents/scribes.md` | — | (scribes uses Post-task only, no pre-task search) | No change needed |

**Note on target:** The `kodehold-<team>` memoirs (e.g., `kodehold-architects`) already exist with concepts. A combined `kodehold-teams` memoir does NOT currently exist but could be created. The simplest migration is to reference the existing `kodehold-<team>` memoirs directly.

### 2.2 ICM Knowledge Flow SKILL.md

| Line | Current Content | Change Needed |
|------|----------------|---------------|
| 19 | Step 2: search `<team>-learnings` memoir | Update to reference actual memoir names |
| 27 | Step 7: store to `kodehold-<project>-<topic>-<team>-learnings` | Update topic pattern |
| 54 | Example: `memoir="<team>-learnings"` | Update example |
| 86 | Store topic: `kodehold-<project>-<topic>-<team>-learnings` | Update topic pattern |

### 2.3 Scribes Agent (`.opencode/agents/scribes.md`)

| Line | Current Content | Change Needed |
|------|----------------|---------------|
| 87 | `icm_memory_extract_patterns -t kodehold-fls-learnings -m kodehold-fls` | Update topic from `kodehold-fls-learnings` |
| 263 | Comment: `kodehold-architects, kodehold-engineers, etc.` | This is still accurate — `kodehold-<team>` memoirs still exist |
| 272 | `icm_memory_extract_patterns -t kodehold-<project>-learnings -m kodehold-<project>` | This is still accurate |
| 292 | Table: KodeHold itself → `kodehold-arch` \| `kodehold-patterns` | `kodehold-arch` still exists but name is legacy; `kodehold-patterns` doesn't exist |

### 2.4 Protocol Reference (`.opencode/references/kodehold-protocol.md`)

| Line | Current Content | Change Needed |
|------|----------------|---------------|
| 57 | `kodehold-architecture-teams` | Update example topic name (non-critical, just an example) |

### 2.5 CHANGES.md

Need to add entry for the upcoming v0.17.0 release documenting the memoir restructure.

**No changes needed:** `.opencode/agents/director.md`, `.opencode/agents/second-opinion.md`, VERSION.md, TODO.md (these files don't reference stale memoir names).

---

## 3. ADR Impact

### 3.1 ADR-0027 (ICM Knowledge Flow Invocation Modes) — HIGH IMPACT

**Status:** Proposed

ADR-0027 contains a large table (lines 154-161) mapping each team to their learnings topics and concept memoirs. Most entries reference the now-deprecated `kodehold-<team>-learnings` pattern:

```markdown
| Team | ... | Team Learnings Topic | Concept Memoirs |
|------|-----|---------------------|-----------------|
| Architects | ... | `kodehold-architects-learnings` | `kodehold-arch`, ... |
| Engineers  | ... | `kodehold-engineers-learnings`  | ... |
| Testers    | ... | `kodehold-testers-learnings`    | ... |
| Reviewers  | ... | `kodehold-reviewers-learnings`  | ... |
| Scribes    | ... | `kodehold-scribes-learnings`    | ... |
| FLS        | ... | `kodehold-fls-learnings`        | ... |
```

**Specific changes needed:**
1. All 6 entries in the `Team Learnings Topic` column: `kodehold-<team>-learnings` → new convention
2. The `Concept Memoirs` column references `kodehold-arch` (still exists but legacy name)
3. The 8-step table (lines 13-14) references `<team>-learnings` in Step 2
4. Lines 83-84 and 92-93 in the Pre-task/Post-task mode descriptions

**Recommendation:** Rewrite the entire Team Parameters table and update the step descriptions to match the new memoir structure.

### 3.2 ADR-0023 (Semantic Memory Automation) — MEDIUM IMPACT

**Status:** Superseded

ADR-0023 is already marked as Superseded (by ICM plugin hooks). However, it still references `kodehold-arch` (lines 37-39, 93, 95) and `kodehold-architects` (line 40, 94) as target memoirs:

````markdown
| Source | What to Extract | Target Memoir |
|--------|----------------|---------------|
| New ADR | Decision, context, consequences as concepts | `kodehold-arch` |
| Updated ADR | Refined concepts, new relationships | `kodehold-arch` |
| Design doc component | Component role, relationships, dependencies | `kodehold-arch` |
| Team structure | Team roles, responsibilities, interactions | `kodehold-architects` |
````

**Specific changes needed:**
1. Update the "Target Memoir" references from `kodehold-arch` to current naming
2. Update the Memoir Structure table (lines 91-96) similarly

**Note:** Since ADR-0023 is Superseded, the changes are cosmetic for historical accuracy — the ADR documents past decisions but is no longer active policy.

### 3.3 ADR-0009 (ICM MCP Integration) — MEDIUM IMPACT

**Status:** Accepted

ADR-0009 defines the original Layer 2 memoir structure (lines 49-53):

```markdown
| Memoir | Concepts | Purpose |
|--------|----------|---------|
| `kodehold-arch` | Director, Architects, Engineers... | KodeHold's own architecture |
| `kodehold-patterns` | Composable validators... | Reusable patterns |
```

`kodehold-arch` still exists (3 concepts) but `kodehold-patterns` was never created. The memoir names need updating.

**Specific changes needed:**
1. Line 51: `kodehold-arch` → current architecture memoir name
2. Line 52: `kodehold-patterns` → update or remove (never created)

---

## 4. Risk Assessment

### 4.1 Agent Execution Risk (HIGH)

If an agent follows its current instructions verbatim and tries to search `kodehold-architects-learnings` (which doesn't exist), the ICM `memoir_search` will return empty results. This means:
- **Pre-task context loading fails silently** — agents won't get team-specific patterns
- **No hard error** — the search returns empty, the agent continues without the context it needs
- **Patterns are lost** — team-specific knowledge is effectively invisible

**Mitigation:** Update agent files before any new delegation. This is the highest priority item.

### 4.2 ICM Topic Fragmentation Risk (MEDIUM)

If Step 7 stores still use the old `kodehold-<project>-<topic>-<team>-learnings` topic pattern, learnings will go to:
- Topics that no agent searches during Pre-task (orphaned stores)
- Fragmented from the main `kodehold-learnings` memoir

**Mitigation:** Update SKILL.md Step 7 topic convention first, then update agent files.

### 4.3 ADR History Inconsistency Risk (LOW)

ADR-0009 and ADR-0023 are Accepted/Superseded respectively — they document past decisions. Updating them is cosmetic but important for:
- New architects reading ADRs for context
- Avoiding confusion when searching ADRs for memoir names
- Maintaining the ADR index as the single source of truth

**Mitigation:** Update ADRs but keep original decisions intact. Mark updates with a note about the memoir restructure.

### 4.4 `kodehold-arch` still exists but is legacy (LOW)

The `kodehold-arch` memoir (3 concepts) still exists and is referenced in ADR-0009, ADR-0023, and scribes.md. If we plan to deprecate/rename it, we need to:
1. Migrate its 3 concepts to the appropriate new memoir
2. Update all references
3. Then delete the memoir

**Risk:** If we delete `kodehold-arch` before migrating concepts, we lose 3 architectural patterns (Director Delegation Enforcement, ADR acceptance criteria, gzipped logging pattern).

### 4.5 `kodehold-patterns` doesn't exist (LOW)

Referenced in ADR-0009 and scribes.md but was never created. This is a dead reference — no data loss risk since no data was ever stored there.

---

## 5. Migration Plan

### Phase 1: Documentation (impact analysis + design)
| Step | Action | Team |
|------|--------|------|
| 1.1 | Write and approve this impact analysis | Architects |
| 1.2 | Update design doc with memoir restructure section | Architects |
| 1.3 | **Write new ADR-0028** documenting the memoir restructure decision | Architects |
| 1.4 | `.impact_analysis_done` marker created | Architects |

### Phase 2: Config updates (ACTIVE implementation)
| Step | Action | Team | Depends on |
|------|--------|------|------------|
| 2.1 | Update `.opencode/skills/icm-knowledge-flow/SKILL.md` — fix Step 2, Step 7, examples | Engineers | Phase 1 |
| 2.2 | Update 5 agent files (architects, engineers, testers, reviewers, fls) | Engineers | 2.1 |
| 2.3 | Update `.opencode/agents/scribes.md` — extraction pattern example, memoir table | Engineers | 2.1 |
| 2.4 | Update `.opencode/references/kodehold-protocol.md` — ICM topic example | Engineers | — |
| 2.5 | Update ADR-0027 — rewrite team parameters table, update step references | Scribes | Phase 1 |
| 2.6 | Update ADR-0009 — memoir table references | Scribes | Phase 1 |
| 2.7 | Update ADR-0023 — target memoir references (cosmetic, ADR is Superseded) | Scribes | Phase 1 |
| 2.8 | Update CHANGES.md — add v0.17.0 entry | Scribes | Phase 1 |
| 2.9 | Add CHANGES.md recommendation for VERSION.md bump | Scribes | — |

### Phase 3: Verification (REVIEW)
| Step | Action | Team | Depends on |
|------|--------|------|------------|
| 3.1 | Run full test suite to verify no regressions | Testers | Phase 2 |
| 3.2 | Verify no stray references to `kodehold-*-learnings` in config files | Reviewers | Phase 2 |
| 3.3 | Verify ADR-0027 table is consistent with current memoir list | Reviewers | 2.5 |
| 3.4 | Verify all 5 agent files have correct memoir names | Reviewers | 2.2 |

### Phase 4: Cleanup (data layer)
| Step | Action | Team | Depends on |
|------|--------|------|------------|
| 4.1 | Verify no agents reference old memoir names | Any | Phase 3 |
| 4.2 | **Delete deprecated memoirs** (if `kodehold-arch` is renamed, delete old) | Scribes | Phase 3 |
| 4.3 | Extract patterns from `context-kodehold` topic into appropriate memoirs | Scribes | — |

### Phase 5: Gate closure
| Step | Action | Team |
|------|--------|------|
| 5.1 | Run CLOSED→REOPEN gate via `scripts/gate.sh --transition CLOSED_TO_REOPEN` | Director |
| 5.2 | After REOPEN, proceed to ACTIVE per normal lifecycle | Director |
| 5.3 | Run REOPEN→ACTIVE gate after Reviewers approve updates | Director |

---

## 6. Rollback Plan

### Scenario A: Config update causes agent misbehavior

If agents start failing because the new memoir names are wrong:

1. **Revert agent files** to their pre-migration state:
   ```bash
   git checkout HEAD -- .opencode/agents/*.md
   git checkout HEAD -- .opencode/skills/icm-knowledge-flow/SKILL.md
   ```
2. **Revert ADRs** if they were updated:
   ```bash
   git checkout HEAD -- docs/adr/ADR-0027*
   git checkout HEAD -- docs/adr/ADR-0009*
   ```
3. **Keep data layer changes** — the merged `kodehold-learnings` is backward compatible
4. **Delete `.impact_analysis_done`** to return to CLOSED:
   ```bash
   rm .impact_analysis_done
   ```

### Scenario B: `kodehold-arch` deleted prematurely

If `kodehold-arch` is deleted (concepts migrated) and a loss is discovered:

1. **Recreate concepts** from the migration log stored in ICM:
   ```bash
   icm_memory_recall -t kodehold-memoir-migration -i critical high
   ```
2. **Re-add concepts** to the correct target memoir
3. **Restore links** between migrated concepts

### Scenario C: Phase 1 complete but Phase 2 fails

If the impact analysis is approved but config updates cannot be completed:

1. **Keep `.impact_analysis_done`** — it only gates CLOSED→REOPEN, not ACTIVE work
2. **Remain in REOPEN** until Engineers complete the updates
3. **No data loss** — the data layer changes are already committed in ICM

### Scenario D: Complete rollback (worst case)

If the entire restructure needs to be reversed:

1. **Restore ICM memoirs** — ICM is append-only, so old memoirs still exist in backup:
   ```
   # From ICM backup or git history
   ```
2. **Revert all config files** to git baseline
3. **Delete the merged `kodehold-learnings` memoir** (if concepts were re-distributed)
4. **Recreate individual `kodehold-<team>-learnings` memoirs** from extracted patterns

**Note:** ICM has no native "undelete" for memoirs. Rollback depends on git backups of the `.icm/` directory or manual concept recreation.

---

## 7. Detailed File-by-File Change Specifications

### 7.1 SKILL.md Changes

| File | Section | Old Text | New Text |
|------|---------|----------|----------|
| `.opencode/skills/icm-knowledge-flow/SKILL.md` | Pre-task Step 2 | `search <team>-learnings memoir` | Search `kodehold-<team>` memoir for team-specific patterns |
| `.opencode/skills/icm-knowledge-flow/SKILL.md` | Post-task Step 7 | `kodehold-<project>-<topic>-<team>-learnings` | `kodehold-<project>-<topic>-team-learnings` |
| `.opencode/skills/icm-knowledge-flow/SKILL.md` | Step 2 example | `memoir="<team>-learnings"` | `memoir="kodehold-<team>"` |
| `.opencode/skills/icm-knowledge-flow/SKILL.md` | Step 7 example | `topic="kodehold-<project>-<topic>-<team>-learnings"` | `topic="kodehold-<project>-<topic>-team-learnings"` |

### 7.2 Agent File Changes (per-file template)

Each of the 5 agent files needs this change in the ICM Knowledge Flow Pre-task section:

```diff
 ## ICM Knowledge Flow (Pre-task Mode)
 
 Follow the ICM Knowledge Flow skill protocol in **Pre-task mode**:
 1. Search `kodehold-learnings` memoir for relevant patterns before starting work
-2. Search `kodehold-<team>-learnings` memoir for team-specific patterns before starting work
+2. Search `kodehold-<team>` memoir for team-specific patterns before starting work
```

Specific team → memoir mapping:

| Agent File | Old Memoir | New Memoir |
|------------|-----------|------------|
| `architects.md` | `kodehold-architects-learnings` | `kodehold-architects` |
| `engineers.md` | `kodehold-engineers-learnings` | `kodehold-engineers` |
| `testers.md` | `kodehold-testers-learnings` | `kodehold-testers` |
| `reviewers.md` | `kodehold-reviewers-learnings` | `kodehold-reviewers` |
| `fls.md` | `kodehold-fls-learnings` | `kodehold-fls` |

### 7.3 Scribes.md Specific Changes

**Line 87** — extraction pattern example:
```diff
-icm_memory_extract_patterns -t kodehold-fls-learnings -m kodehold-fls
+icm_memory_extract_patterns -t kodehold-fls -m kodehold-fls
```

**Line 292** — memoir target table:
```diff
-| KodeHold itself | `kodehold-arch` | `kodehold-patterns` |
+| KodeHold itself | `kodehold-arch` | `kodehold-learnings` |
```

**Line 263** — already references `kodehold-architects, kodehold-engineers, etc.` which is still accurate (individual team memoirs still exist) — no change needed.

---

## 8. Open Questions

1. **Should we create a `kodehold-teams` combined memoir?** Currently the individual team memoirs (`kodehold-architects`, `kodehold-engineers`, etc.) exist independently. A combined `kodehold-teams` memoir would allow single-query searches across all teams but requires merging existing concepts.

2. **What happens to `kodehold-arch`?** It has 3 architectural concepts that could live in `kodehold-learnings` or a new `kodehold-architecture` memoir. The name is legacy (predates the team memoirs) but still descriptive.

3. **Should ADR-0009 and ADR-0023 be updated or left as historical records?** ADR-0009 is Accepted (still active policy), so it should be updated. ADR-0023 is Superseded — changes are cosmetic only.

4. **Is a new ADR needed?** The memoir restructure represents a significant architectural decision. Consider ADR-0028 to document:
   - Why individual `*-learnings` memoirs were deprecated
   - The new unified `kodehold-learnings` as the single learnings store
   - The team knowledge memoirs (`kodehold-<team>`) as the search targets for team-specific patterns

---

## 9. References

- **ICM memoirs** — verified via `icm_memoir_list`: 12 total, including `kodehold-learnings` (63 concepts, 68 links)
- **Agent files** — 6 files under `.opencode/agents/`:
  - 5 with stale `kodehold-<team>-learnings` references (architects, engineers, testers, reviewers, fls)
  - 1 with extraction pattern references (scribes)
- **ADRs affected** — ADR-0027 (high), ADR-0009 (medium), ADR-0023 (medium)
- **SKILL.md** — `.opencode/skills/icm-knowledge-flow/SKILL.md`
- **Protocol reference** — `.opencode/references/kodehold-protocol.md`
- **CHANGES.md** — `/home/kiffer/project/kodehold/CHANGES.md`
