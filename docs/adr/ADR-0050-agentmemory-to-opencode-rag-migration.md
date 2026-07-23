# ADR-0050: Agentmemory → OpenCode RAG Migration

## Status

Accepted

**Date:** 2026-06-27

**Note:** Second opinion (Step 5) skipped per user request — cross-model validation provider not available at time of finalization.

> **Superseded (ADR-0054):** The OpenCode RAG tools adopted in this ADR (§3, OpenCode RAG Knowledge Flow Skill) have been superseded by Graphify knowledge graph per [ADR-0054](ADR-0054-replace-opencode-rag-with-graphify.md). Graphify is now KodeHold's sole documented code retrieval method. This ADR is retained for historical context around the agentmemory removal.
>
> **Superseded (ADR-0051):** §5 (File-Based Persistent Storage) is superseded by [ADR-0051](ADR-0051-opencode-mem-persistent-memory.md) — opencode-mem replaces the file-based `.opencode/memory/` proposal. The file-based storage was never implemented; opencode-mem provides semantic search, auto-capture, and compaction. The rest of ADR-0050 (agentmemory removal, OpenCode RAG adoption) remains valid for its time, though OpenCode RAG has since been replaced by Graphify per ADR-0054.

## Context

### The Problem

KodeHold currently depends on **agentmemory**, an external daemon (`iii`) and npm package (`@agentmemory/agentmemory`), as its persistent memory backend. Agentmemory provides memory storage/retrieval, lesson recall, action management, crystals/signals, routine execution, and session tracking across all 7 team agents, 6 skills, 10+ scripts, and 30+ ADRs.

However, OpenCode now provides **native RAG tools** (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) that cover the same use cases without requiring:

- An external daemon (`iii` engine on port 3111)
- An npm package (`@agentmemory/agentmemory`)
- A REST API with WebSocket dependencies
- Health checks, daemon restarts, patching scripts
- Persistent background processes

The cost of maintaining agentmemory has grown to outweigh its benefits. With 71 files referencing agentmemory across the entire codebase, and with OpenCode RAG offering equivalent or superior functionality via built-in tools, removing agentmemory eliminates an entire class of infrastructure maintenance while simplifying the architecture.

### Key Forces

1. **Infrastructure debt.** Agentmemory requires a running daemon (`iii`), npm dependencies, config files (`~/.agentmemory/iii-config.yaml`), health checks in gate/ship scripts, patching scripts (`scripts/patch-agentmemory.sh`), and a custom viewer (`tools/viewer/`). This adds approximately 10 files and hundreds of lines of operational overhead.

2. **No incremental value over OpenCode RAG.** Every agentmemory function has an OpenCode RAG or stdlib equivalent. `search_semantic` provides semantic search (replacing `memory_lesson_recall`, `memory_recall`, `memory_smart_search`). `find_usages` tracks symbol references. `get_file_skeleton` provides structural file awareness. `grep`/`glob` handle keyword search. File-based storage (`.opencode/memory/`) replaces persistent memory.

3. **Daemon dependency.** Agentmemory's `iii` daemon must be running, healthy, and reachable for agents to function. This adds startup latency, a failure point (daemon crash), and operational complexity (systemd service, port 3111 availability).

4. **71 files to update.** Agentmemory references are spread across agent definitions, skills, scripts, tests, design docs, and ADRs. The migration requires systematic, phased updates across every file.

5. **Routines are the only feature worth preserving.** ADR-0032 defines 4 workflow templates (ADR flow, implement flow, bugfix flow, ship gate) that are currently stored in agentmemory and instantiated via `memory_routine_run`. These templates are valuable but do not require a daemon — they can be embedded as static tables in `director.md`.

6. **Rollback must be possible.** If the migration causes issues, reverting to agentmemory must be straightforward (git revert + daemon restart).

### File Inventory

The migration touches 71 files across 7 categories:

| Category | Files | Approximate agentmemory References |
|----------|-------|------------------------------------|
| Team agent files | 7 (director.md, architects.md, engineers.md, reviewers.md, testers.md, fls.md, scribes.md) | ~137 |
| Skills | 6 (agentmemory-knowledge-flow, ponytail-audit, resume, investigate, skills README) | ~68 |
| Scripts | 10 (gate.sh, ship.sh, patch-agentmemory.sh, benchmark.sh, consolidate-all.sh, migrate-project-slugs.sh, token-dashboard.sh, slug-migration log, tag-lessons.py, migrate-project-scope.py) | ~46 |
| Tests | 2 (02-icm-check.sh, 04-fls-workflow.sh) | ~2 |
| Design docs | 3 (README.md, knowledge-recall.md, actions-crystals-integration.md) | ~30 |
| ADRs | ~30 files (ADR-0028 through ADR-0046 and older) | ~100+ |
| References & config | 5 (kodehold-protocol.md, AGENTS.md, viewer README, opencode.json, .gitignore) | ~12 |

**Total: 71 files, ~395+ agentmemory references.**

### Agentmemory vs. OpenCode RAG Function Mapping

Every agentmemory function has a direct replacement in OpenCode RAG or stdlib:

| agentmemory Function | Replacement | Rationale |
|---------------------|-------------|-----------|
| `agentmemory_memory_lesson_recall()` | `search_semantic(query="<topic>", topK=5)` | Semantic codebase/doc search replaces lesson recall |
| `agentmemory_memory_recall()` | `search_semantic(query="<topic>")` | Same — searches indexed codebase and docs |
| `agentmemory_memory_smart_search()` | `search_semantic(query="<topic>")` | Single unified semantic search |
| `agentmemory_memory_save()` | Write structured markdown to `.opencode/memory/` directory | Files are persistent, version-controlled, and require no daemon |
| `agentmemory_memory_frontier()` | Simple protocol steps in director.md | Director delegates serially — no action queue needed |
| `agentmemory_memory_action_create/update()` | Protocol steps + `todowrite` in director.md | Simplified delegation tracking |
| `agentmemory_memory_action_lease()` | **Remove** — Director delegates serially, no concurrent execution | No lease mechanism needed |
| `agentmemory_memory_routine_run()` | **KEEP** as static template definitions in director.md Templates section | Preserve workflow definitions without agentmemory |
| `agentmemory_memory_crystallize()` | **Remove** — manual session summaries suffice | Crystals were auto-generated digests with marginal value |
| `agentmemory_memory_signal_send/read()` | **Remove** — Director delegates directly via Task tool | No inter-agent signaling needed |
| `agentmemory_memory_diagnose/health()` | **Remove** — no daemon to diagnose | Health checks become unnecessary |
| `agentmemory_memory_consolidate/reflect/patterns()` | **Remove** | Auto-consolidation layer removed with agentmemory |
| `agentmemory_memory_procedural_list()` | `grep` or `search_semantic` for procedure files | Stdlib tools suffice |
| `agentmemory_memory_lesson_save()` | **Remove** — lessons go in `AGENTS.md` or skill markdown files | Static documentation replaces dynamic lessons |
| `agentmemory_memory_graph_query()` | **Remove** — no knowledge graph | Knowledge graph removed with agentmemory |
| Session checkpoints | `.opencode/memory/checkpoints/*.md` structured files | File-based checkpoint storage |
| Prospective memory | `.opencode/memory/prospective/*.md` structured files | File-based task tracking |
| CLOSED distillation | **Remove** — doc files ARE the distillation | Documentation replaces memory summaries |
| Token metrics | Store as JSON files in `.opencode/memory/` (or remove if not needed) | Optional file-based storage |
| Health checks in gate.sh/ship.sh | **Remove** agentmemory health check step | Gate and ship scripts no longer check daemon health |
| Viewer (`tools/viewer/`) | **Remove** agentmemory-specific viewer | No viewer needed for file-based memory |
| `agentmemory-knowledge-flow` skill | **Replace** with `opencode-rag-knowledge-flow` skill | New skill uses `search_semantic` instead of `memory_lesson_recall` |
| All patches/scripts for agentmemory | **Remove or deprecate** | Daemon-specific scripts become obsolete |

> **Data-source note:** `search_semantic` searches the indexed workspace **files**, not a dedicated lesson database. Agentmemory's lesson system stored runtime-generated learnings outside the file system. Any learning that was stored in agentmemory but **never written to a file** (ADR, design doc, agent file, skill) will NOT survive the migration. Phase 9 should include a step to manually migrate critical learnings from the agentmemory archive (`~/.agentmemory/`) to the appropriate file before decommissioning. The ADR's rollback plan preserves the archive, but a specific migration step for vital lessons is recommended.

### What We Keep: The 4 Routine Templates (ADR-0032)

The 4 routine templates are the only agentmemory feature worth preserving. They migrate from agentmemory storage to static definitions in `director.md`:

| Template | Steps | Owner | Preserved As |
|----------|-------|-------|-------------|
| `kodehold-adr-flow` | 5 steps: research + write-adr → design-doc-update → review-adr → [cross-validate] → finalize | Architects | Static table in director.md Action Frontier Protocol section |
| `kodehold-implement-flow` | 6 steps: design → [design-review] → implement → code-review → test → gate-validation | Engineers | Static table in director.md |
| `kodehold-bugfix-flow` | 5 steps with branching: triage → hotfix or REOPEN | FLS | Static table with condition branches |
| `kodehold-ship-gate` | 7 parallel steps: version-check → changelog-check → todo-check → test-suite → agentmemory-check* → git-status → branch-check | Director | Static table (* agentmemory-check replaced with git-status-only or removed) |

These are documented in ADR-0032 and will be embedded as template tables in director.md's workflow section. The Director instantiates them manually by following the table steps rather than via `memory_routine_run`.

### ADRs Affected

#### ADRs to Deprecate (via this ADR)

| ADR | Title | Current Status | Deprecation Rationale |
|-----|-------|---------------|----------------------|
| ADR-0028 | Agentmemory Project Detection Strategy | Accepted | Agentmemory removed; project detection handled by git-based workspace |
| ADR-0029 | ICM → Agentmemory Migration Strategy | Accepted | Superseded by this ADR — reverse migration to OpenCode RAG |
| ADR-0030 | Agentmemory Knowledge Flow | Accepted | Replaced by opencode-rag knowledge flow skill |
| ADR-0031 | Actions + Crystals for Director Delegation | Accepted | Replaced by simplified delegation protocol in director.md |
| ADR-0033 | Crystals + Signals for KodeHold | Accepted | Replaced by direct Task delegation |
| ADR-0035 | Custom KodeHold Viewer | Accepted | Agentmemory viewer removed; no viewer needed |
| ADR-0043 | Agentmemory Slot Integration | Proposed | Slots removed with agentmemory |
| ADR-0044 | Automatic Session Lifecycle Management | Accepted | Agentmemory-capture plugin removed |

#### ADRs to Update (not deprecate)

| ADR | Update Required |
|-----|----------------|
| ADR-0032 | Update template registration from "stored in agentmemory" to "embedded as static tables in director.md" |
| ADR-0038 | Update from agentmemory lesson recall to `search_semantic` in knowledge recall protocol |
| ADR-0039 | Update from agentmemory recall to `search_semantic` for pre-flight knowledge checks |

#### ADRs Already Superseded/Deprecated (no change needed)

| ADR | Status | Notes |
|-----|--------|-------|
| ADR-0004 | Deprecated | Already deprecated — superseded by ADR-0029 |
| ADR-0009 | Deprecated | Already deprecated — ICM MCP |
| ADR-0019 | Superseded | Already superseded — session compression |
| ADR-0020 | Superseded | Already superseded — hierarchical memory |
| ADR-0021 | Superseded | Already superseded — prospective memory |
| ADR-0022 | Superseded | Already superseded — episodic extraction |
| ADR-0023 | Superseded | Already superseded — semantic memory |
| ADR-0024 | Deprecated | Already deprecated — shared memory |
| ADR-0025 | Deprecated | Already deprecated — A2A protocol |
| ADR-0027 | Deprecated | Already deprecated — ICM knowledge flow |

## Decision

### 1. Remove Agentmemory and Replace with OpenCode RAG + File-Based Storage

Adopt a complete replacement of the agentmemory dependency with OpenCode's native RAG tools and structured file-based storage. The replacement mapping defined above governs every substitution.

No external daemon, no npm package, no REST API, no background process. All persistent state lives in version-controlled files under `.opencode/memory/`.

### 2. Adopt the 10-Phase Migration Plan

| Phase | Focus | Key Files | Priority |
|-------|-------|-----------|----------|
| **1** | Replace `agentmemory-knowledge-flow` skill → `opencode-rag-knowledge-flow` | `.opencode/skills/agentmemory-knowledge-flow/SKILL.md`, create `.opencode/skills/opencode-rag-knowledge-flow/SKILL.md` | High |
| **2** | Update 6 team agents: replace agentmemory refs with opencode-rag | `architects.md`, `engineers.md`, `reviewers.md`, `testers.md`, `fls.md`, `scribes.md` | High |
| **3** | Rewrite `director.md` — Action Frontier → simplified protocol; embed routines as static tables | `.opencode/agents/director.md` | High |
| **4** | Rewrite skills: ponytail-audit, resume, investigate → use `search_semantic` | `.opencode/skills/*/SKILL.md` | Medium |
| **5** | Update scripts: gate.sh, ship.sh, patch-agentmemory, benchmark, consolidate-all, migrate-project-slugs, token-dashboard | `scripts/*.sh` | Medium |
| **6** | Update tests: 02-icm-check.sh, 04-fls-workflow.sh | `tests/*.sh` | Medium |
| **7** | Deprecate agentmemory-specific ADRs, update ADR-0032, update ADR index | `docs/adr/*.md` | Large |
| **8** | Update design docs: README.md, knowledge-recall.md, actions-crystals-integration.md | `docs/design/*.md` | Medium |
| **9** | Update references, config, viewer docs, clean up obsolete scripts | `.opencode/references/`, `tools/viewer/`, `opencode.json` | Medium |
| **10** | Final verification — `grep` confirms zero agentmemory references across entire codebase | — | High |

#### Phase Dependencies

Phases 1-3 must execute sequentially (skills → agents → director). Phases 4-9 can parallelize within each category but should complete before Phase 10 (final verification).

```
Phase 1 (Skill replacement)
   │
   ▼
Phase 2 (Agent files)
   │
   ▼
Phase 3 (Director rewrite)
   │
   ├──────────┬──────────┬──────────┬──────────┐
   ▼          ▼          ▼          ▼          ▼
Phase 4    Phase 5    Phase 6    Phase 7    Phase 8    Phase 9
(Skills)   (Scripts)  (Tests)    (ADRs)     (Design    (Config/
                                              docs)     cleanup)
   │
   └─────────────────────────┬─────────────────────────────┘
                             ▼
                       Phase 10
                  (Final verification)
```

#### Verification Gates

| Transition | Gate Criteria |
|------------|---------------|
| Phase 1 → 2 | New skill file exists; old skill file deprecated; `agentmemory` not referenced in new skill |
| Phase 2 → 3 | All 6 agent files updated; zero `agentmemory` references in agent files; `search_semantic` present |
| Phase 3 → 4 | director.md rewritten; routine templates embedded as static tables; Action Frontier Protocol simplified |
| Phases 4-9 → 10 | All category-specific files updated; no agentmemory references in respective categories |
| Phase 10 → Done | `grep -r "agentmemory" --include="*.md" --include="*.sh" --include="*.py" --include="*.json" .` returns zero matches across ALL workspace files |

### 3. OpenCode RAG Knowledge Flow Skill

> **Historical (superseded by ADR-0054):** The opencode-rag-knowledge-flow skill created here has been replaced by `graphify-knowledge-flow` per ADR-0054. Graphify is now the sole code retrieval method. This section is retained for historical context.

Replace `.opencode/skills/agentmemory-knowledge-flow/SKILL.md` with a new skill at `.opencode/skills/opencode-rag-knowledge-flow/SKILL.md`:

**Pre-task Knowledge Retrieval:**
```markdown
## Pre-task Knowledge Retrieval

1. **Search relevant code** — before starting work, call:
   - `search_semantic(query="<task-related-topic>", topK=5)` to find relevant code patterns
   - `find_usages("<key-symbol>")` to locate all references of key functions/variables
   - `get_file_skeleton("<file-path>")` to understand file structure before reading

2. **Search relevant documentation** — call:
   - `search_semantic(query="<topic> design decisions", pathHints=["docs/"])` to find relevant ADRs and design decisions
   - Use `pathHints` parameter to scope searches to specific directories

3. **Search team-specific patterns** — each team searches with its context:
   - Engineers: `search_semantic(query="engineers patterns <domain>", topK=5)`
   - Reviewers: `search_semantic(query="reviewers patterns <domain>", topK=5)`
   - (etc. for each team)
```

### 4. Director Delegation Simplification

Replace the current Action Frontier Protocol (which uses `memory_frontier`, `memory_action_create`, `memory_action_lease`, `memory_crystallize`, `memory_signal_send/read`) with a simplified protocol:

| Component | Current (agentmemory) | New (OpenCode RAG) |
|-----------|----------------------|-------------------|
| Delegation queue | `memory_frontier` | Sequential delegation in director.md — one task at a time via Task tool |
| Action tracking | `memory_action_create/update` | Simple `todowrite` + progress tracking in Task tool |
| Dependencies | `memory_action_lease` with `requires` | Director manually ensures prerequisites via `todowrite` |
| Completion | `memory_crystallize` | Manual session summary |
| Inter-agent signals | `memory_signal_send/read` | Director delegates directly — no agent-to-agent communication |
| Routine instantiation | `memory_routine_run` | Static tables in director.md — Director follows steps manually |

#### Routine Templates as Static Tables (Preserved from ADR-0032)

Embed the 4 routine templates directly in director.md as markdown tables. Example for the ADR flow:

```markdown
### kodehold-adr-flow (5 steps)
| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | architects | research | (none) | 8 | No |
| 2 | architects | write-adr | step 1 | 8 | No |
| 3 | scribes | design-doc-update | step 2 | 5 | No |
| 4 | reviewers | review-adr | step 2 | 7 | No |
| 5 | second-opinion | cross-validate | step 2 | 7 | Yes |
| 6 | scribes | finalize | steps 4, 5* | 5 | No |
```

Full definitions for all 4 templates (adr-flow, implement-flow, bugfix-flow, ship-gate) will be embedded in director.md per ADR-0032.

### 5. File-Based Persistent Storage

Replace all persistent memory writes with structured files:

| Storage Type | Location | Format | Managed By |
|-------------|----------|--------|------------|
| Session checkpoints | `.opencode/memory/checkpoints/<session-id>.md` | Markdown with frontmatter (date, session, project, summary) | Director |
| Prospective tasks | `.opencode/memory/prospective/*.md` | Markdown with YAML frontmatter (priority, status, requires) | Architects |
| Token metrics | `.opencode/memory/metrics/*.json` | JSON | Scribes |
| Team learnings | `AGENTS.md` or skill markdown files | Markdown in the relevant section | Scribes |

### 6. Infrastructure Removals

The following infrastructure is removed as part of Phase 5 (scripts) and Phase 9 (cleanup):

| Infrastructure | File(s) | Removal Action |
|---------------|---------|----------------|
| Agentmemory daemon | `systemctl --user restart agentmemory` | No longer needed |
| npm package | `npm install -g @agentmemory/agentmemory` | Uninstall |
| Config | `~/.agentmemory/iii-config.yaml` | Delete |
| Patch script | `scripts/patch-agentmemory.sh` | Delete |
| Health checks | `scripts/gate.sh` agentmemory check, `scripts/ship.sh` agentmemory check | Remove steps |
| Viewer | `tools/viewer/` (agentmemory viewer) | Delete directory |
| Legacy benchmark | `scripts/benchmark.sh` | Delete or deprecate |
| Consolidation scripts | `scripts/consolidate-all.sh` | Delete |
| Migration scripts | `scripts/migrate-project-slugs.sh`, `scripts/tag-lessons.py`, `scripts/migrate-project-scope.py` | Delete or archive |
| Dashboard queries | `scripts/token-dashboard.sh` agentmemory queries | Replace with file-based queries |

### 7. Design Doc Update Requirements

After Phase 8, `docs/design/README.md` must be updated:

- **Section 7.2 (Agentmemory)** — rewrite to reflect removal. Replace with "7.2 OpenCode RAG" section.
- **ADR Index table** — add ADR-0050 row; update deprecation status for ADRs listed above.
- **Principle table** — add new principle: "No external memory dependencies — use OpenCode RAG for code retrieval, files for persistent storage."
- **Skills table (Section 7.4)** — rename `agentmemory-knowledge-flow` → `opencode-rag-knowledge-flow`.
- **Session compression (Section 7.5)** — remove references to agentmemory summaries; use file-based summaries.
- **CLOSED state (Section 6)** — remove "context stored in agentmemory"; replace with "context stored in design docs and files."
- **Changelog** — add entry for ADR-0050 migration.

## Documentation

> **Historical (superseded by ADR-0054):** The OpenCode RAG tools documented here have been replaced by Graphify knowledge graph. See ADR-0054 for current documentation.

| Field | Value |
|-------|-------|
| **Tool adopted** | OpenCode RAG (built-in: `search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) |
| **Official docs** | N/A — OpenCode RAG is a built-in toolset without external docs. See OpenCode's tool reference for interfaces. |
| **Version documented** | Built into OpenCode — no separate versioning |
| **Key concepts** | `search_semantic`: vector-based semantic search of the indexed codebase. `find_usages`: symbol reference locator — finds every reference to a function, variable, or class. `get_file_skeleton`: structural file overview showing functions, classes, interfaces with line numbers. `describe_image`: vision-model-based image description. All tools require zero configuration — they index the workspace automatically. |
| **Storage replacement** | Structured markdown files in `.opencode/memory/` — version-controlled, no daemon needed, survives restarts and clones. |
| **Known gotchas** | • `search_semantic` only finds indexed workspace content — files outside the workspace or newly created files may not appear until indexing completes. Re-index if search results seem incomplete or stale. |
| | • `find_usages` requires index freshness — after renaming or deleting symbols, the index may briefly return stale results. |
| | • `search_semantic` searches workspace **files**, not a lesson database. Learnings that exist only in agentmemory (never written to a file) will NOT be found. Critical learnings must be manually migrated from the agentmemory archive to the appropriate file before the archive is decommissioned. |
| | • File-based storage in `.opencode/memory/` is not auto-cleaned — manual cleanup or rotation may be needed for checkpoints. |
| | • Agentmemory's `memory_routine_run` auto-resolved dependencies; manual table-following places this burden on the Director. |
| | • The 4 routine templates lose parameter substitution (`title`, `require_second_opinion`, etc.) — these become manual steps. |

## Consequences

### Positive

1. **Zero infrastructure.** No daemon (`iii`), no npm package, no REST API, no port 3111, no health checks, no patching scripts. Everything works with tools already present in OpenCode.

2. **Simplified architecture.** 71 files with agentmemory references reduced to zero. Approximately 10 infrastructure files (daemon config, viewer, patch scripts, health checks) deleted entirely. ~395+ agentmemory function calls replaced with ~50 `search_semantic` calls.

3. **Lower token consumption.** Agentmemory's slot system injected ~8 slots (up to 3000 chars each) into every agent context. Removing slots saves ~8,000+ characters per session start that were consumed by slot auto-injection.

4. **Version-controlled state.** All persistent data moves from agentmemory's opaque database (`~/.agentmemory/`) to version-controlled files under `.opencode/memory/`. This means checkpoints are backed up in git, visible in diffs, and recoverable from any git state.

5. **Faster session startup.** No need to wait for agentmemory daemon health check. No `memory_slot_list` call to load slots. No WebSocket connection to port 3111.

6. **No more patching.** `scripts/patch-agentmemory.sh` and the `iii-config.yaml` workaround (`0.0.0.0` binding) are removed. No more fighting npm upgrade overwrites.

7. **Eliminates a class of failures.** Daemon crash, port conflict, npm version mismatch, binary serialization issues (`state::set` websocket quirks), and corrupted `.bin` files are all removed from the failure domain.

### Negative

1. **Loss of automatic parameter substitution.** The 4 routine templates previously used `memory_routine_run` with parameters (`title`, `feature_description`, `skip_design_review`). In static table form, parameters must be substituted manually by the Director.

2. **Loss of auto-crystallization.** Completed action chains were automatically crystallized into compact digests. This is replaced by manual session summaries, which may be less consistent.

3. **Loss of inter-agent signals.** Agent-to-agent typed messaging via `memory_signal_send/read` is removed. The Director now mediates all cross-agent communication, which adds one delegation round per handoff.

4. **No more cross-session lesson auto-strengthening.** Agentmemory's lesson system would auto-strengthen frequently recalled lessons. File-based lessons are static.

5. **No more auto-consolidation.** Agentmemory's 4-tier consolidation pipeline (working → episodic → semantic → procedural) is removed. All knowledge remains in its original form unless manually distilled.

6. **Loss of knowledge graph.** Agentmemory's `memory_graph_query` for relationship discovery across memories is removed. Relationship tracking becomes manual.

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `search_semantic` is less capable than `memory_lesson_recall` | Medium | Medium | The replacement mapping uses `topK=5` with team-prefixed query format (from ADR-0038). Test coverage in Phase 6 validates recall quality. |
| File-based storage grows without bound | Medium | Low | `.opencode/memory/` checkpoints are lightweight markdown. Manual rotation if needed. Git handles history. |
| Director overload without auto-delegation tools | Medium | Low | The simplified protocol has fewer features but covers 90%+ of delegation patterns. Complex cases fall back to manual `todowrite`. |
| Lost lessons from agentmemory database | Low | Medium | Agentmemory database (`~/.agentmemory/`) is preserved as read-only archive. Specific lessons can be manually migrated if needed. |
| Templates are harder to follow without auto-instantiation | Low | Medium | Static tables are embedded in director.md. Director reads and follows them — same mental model as before, just without the `memory_routine_run` convenience call. |
| `search_semantic` index is stale after file edits — newly added or modified content not yet searchable | Medium | Medium | Add re-index awareness to pre-task workflows: if files were recently modified, agents should re-index or run a second search after a brief delay. The risk diminishes over time as the index catches up. |
| Cross-project knowledge isolation — file-based storage is per-project; no single queryable store across workspaces | Low | Low | Agentmemory's cross-project search was rarely used in practice. If needed, create a shared `.opencode/memory/shared/` directory or document key cross-project learnings in a shared ADR. |
| Rollback needed if migration fails | Low | High | Git revert all changes. Reinstall agentmemory via `npm install -g @agentmemory/agentmemory`. Restart daemon via `systemctl --user restart agentmemory`. This ADR becomes Deprecated. |

### Metrics

| Metric | What It Measures | How to Measure | Target |
|--------|-----------------|----------------|--------|
| Agentmemory references | Total agentmemory function calls remaining | `grep -r "agentmemory\|memory_\|iii " --include="*.md" --include="*.sh" --include="*.py" . \| wc -l` | 0 after Phase 10 |
| Infrastructure files | Files that exist only for agentmemory | Count of: `tools/viewer/`, `scripts/patch-agentmemory.sh`, `~/.agentmemory/` | 0 after cleanup |
| `search_semantic` adoption | Usage of new RAG tool across agent files | `grep -r "search_semantic" .opencode/agents/ \| wc -l` | ≥6 (one per agent) |
| Session startup tokens | Tokens consumed before first delegation | Measure context at startup before/after migration | Reduced by 8,000+ chars (slot content) |
| Delegation cycle time | Time from Director receiving task to first action | Manual timing of 3 delegation rounds | Within 20% of pre-migration |

## ADR References

- **ADR-0028** (Agentmemory Project Detection) — deprecated by this ADR
- **ADR-0029** (ICM → Agentmemory Migration Strategy) — superseded by reverse migration in this ADR
- **ADR-0030** (Agentmemory Knowledge Flow) — replaced by opencode-rag knowledge flow
- **ADR-0031** (Actions + Crystals for Director Delegation) — replaced by simplified protocol
- **ADR-0032** (Routine Templates) — updated to store routines statically
- **ADR-0033** (Crystals + Signals) — replaced by direct Task delegation
- **ADR-0035** (Custom KodeHold Viewer) — viewer removed with agentmemory
- **ADR-0038** (Knowledge Recall Protocol) — updated from agentmemory lesson recall to `search_semantic`
- **ADR-0039** (Pre-Flight Knowledge Check) — updated from agentmemory recall to `search_semantic`
- **ADR-0043** (Agentmemory Slot Integration) — slots removed with agentmemory
- **ADR-0044** (Automatic Session Lifecycle) — agentmemory-capture plugin removed
- **ADR-0048** (Mandatory Documentation Review) — complementary safeguard for tool selection
- **ADR-0049** (Lazy Senior Dev Philosophy) — adds rigor to replacement decisions

## Rollback Plan

If the migration causes critical issues:

1. **Revert all file changes** via `git checkout -- .` (or revert specific commits)
2. **Reinstall agentmemory:** `npm install -g @agentmemory/agentmemory`
3. **Restore agentmemory config:** Copy `iii-config.yaml` from backup
4. **Restart daemon:** `systemctl --user restart agentmemory` or `iii`
5. **Verify:** Run `scripts/gate.sh --validate-only` to confirm health
6. **Mark this ADR Deprecated** — migration reverted, agentmemory restored

The agentmemory database (`~/.agentmemory/`) is intentionally left untouched during migration (no data deletion), ensuring all historical memories, lessons, and sessions remain available if rollback is needed.

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.0 | 2026-06-27 | Initial ADR — Agentmemory → OpenCode RAG Migration |
