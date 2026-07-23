# ADR-0057: Migrate File-Based Memory to opencode-mem

## Status

**Accepted** — 2026-07-21

## Context

KodeHold currently operates a dual memory system:

1. **opencode-mem** (MCP tools: `search_memories`, `add_memory`) — the intended memory system per ADR-0051. Provides semantic search, auto-capture, and persistent storage across sessions.

2. **`.opencode/memory/`** (file-based markdown) — a legacy system with 17 files across 7 subdirectories. The design doc (line 368) explicitly states: "The file-based `.opencode/memory/` storage proposed in ADR-0050 §5 was never implemented and is superseded by opencode-mem per ADR-0051." However, all agent files treat it as the primary memory system with 100+ references.

### Current State of `.opencode/memory/`

```
.opencode/memory/
├── bugs/           (1 file)  — equity-curve-stale-max-2026-07-10.md
├── checkpoints/    (5 files) — session summaries from Jul 3-11
├── decisions/      (5 files) — ADR decisions, test coverage review
├── fixes/          (1 file)  — warmup-shutdown-dashboard-freeze.md
├── metrics/        (1 file)  — ponytail-audit-kodehold-2026-06-28.md
├── patterns/       (3 files) — RSS, price cache, Alpine.js patterns
└── prospective/    (1 file)  — demo search memories task
```

### Reference Sites (100+)

| File | References | Content |
|------|-----------|---------|
| `scribes.md` | ~50 | Entire memory taxonomy, storage rules, checkpoint/compression workflows |
| `director.md` | ~20 | Session lifecycle, transitions, checkpoints, prospective tasks |
| `kodehold-protocol.md` | ~8 | Persistent storage convention, quality gate, shipping gate |
| `investigate/SKILL.md` | ~5 | Store bug findings |
| `resume/SKILL.md` | ~5 | Load checkpoints for session resume |
| `ponytail-audit/SKILL.md` | ~4 | Store audit metrics |
| `skills/README.md` | ~2 | Resume skill description |
| `AGENTS.md` | 1 | Quick Reference |
| `README.md` | ~4 | Feature description, team description |
| `.opencode/commands/remember.md` | ~6 | /remember command writes to `.opencode/memory/` |
| `scripts/ship.py` | 1 | Release note storage |
| `design/README.md` | ~15 | Architecture description (historical) |

### Why opencode-mem Is Superior

| Aspect | `.opencode/memory/` | opencode-mem |
|--------|-------------------|--------------|
| Search | `ls` + `grep` (manual) | Semantic search via `search_memories` |
| Storage | Markdown files on disk | MCP server (abstracted) |
| Scope | Project-specific (manual paths) | Scoped via `scope: "project"` |
| Retrieval | File path knowledge required | Keyword/concept search |
| Auto-capture | None | Built-in |
| Cross-session | File reads | Persistent MCP server |

### What Is NOT Memory (No Migration Needed)

- `.kodehold-state` — lifecycle state machine (INIT/ACTIVE/REVIEW/CLOSED/REOPEN). Gate enforcement, not memory.
- Marker files (`.design_reviewed`, `.testers_done`, `.code_reviewed`, `.second_opinion_done`, `.impact_analysis_done`) — gate enforcement. Stay as files.
- `graphify-out/graph.json` — code/docs knowledge graph. Code understanding tool, not memory.
- OpenCode SQLite DB (`~/.local/share/opencode/opencode.db`) — external infrastructure.

## Decision

### Migration Plan

**Phase 1: Migrate Existing Content (17 files → opencode-mem)**

- Read each `.opencode/memory/` file
- Store via `add_memory(content="...", scope="project")` with appropriate tags
- Preserve timestamps in the memory content
- Categories to migrate: bugs, decisions, patterns, fixes, metrics, checkpoints, prospective

**Phase 2: Update Agent Files (100+ Reference Sites)**

| File | Action |
|------|--------|
| `scribes.md` | Replace entire memory taxonomy with opencode-mem instructions. Remove file-based storage rules. **Remove** "Session Checkpoints" and "Session Compression Workflow" sections entirely. |
| `director.md` | Replace `.opencode/memory/` references with `search_memories`/`add_memory`. Update session lifecycle, transitions, prospective task checking. **Remove** "Session Checkpoint Protocol" and "Session Compression Protocol" sections entirely. |
| `kodehold-protocol.md` | Replace "Persistent Storage Convention" section with opencode-mem instructions. |
| `investigate/SKILL.md` | Replace file-based bug storage with `add_memory`. |
| `ponytail-audit/SKILL.md` | Replace file-based metrics storage with `add_memory`. |
| `skills/README.md` | **Remove** resume skill entry and description. |
| `AGENTS.md` | Update Quick Reference. |
| `README.md` | Update feature description. |
| `scripts/ship.py` | Update release note storage. |

**Phase 3: Delete Removed Files**

| Item | Action |
|------|--------|
| `.opencode/commands/remember.md` | Delete — writes to `.opencode/memory/` (removed) |
| `.opencode/commands/recall.md` | Delete — uses legacy `agentmemory` MCP tools |
| `.opencode/skills/resume/` | Delete directory — depends on checkpoints (removed) |
| `.opencode/skills/graphify-knowledge-flow/` | Archive or delete — superseded by preflight skill |
| `.opencode/skills/README.md` | Update to remove agentmemory-knowledge-flow and graphify-knowledge-flow entries |

**Phase 4: Update Design Doc**

- `docs/design/README.md`: Remove file-based storage architecture description. Update to reflect opencode-mem as sole memory system. Remove checkpoint/compression protocol references.

**Phase 5: Delete Legacy System**

- Delete `.opencode/memory/` directory and all contents
- Verify no remaining references

### Additional Removals

The following systems are **removed entirely**, not migrated. They are redundant with opencode-mem's capabilities or depend on the legacy `.opencode/memory/` filesystem.

| Item | Type | Reason |
|------|------|--------|
| Session Checkpoint Protocol | Protocol section in `director.md` | Redundant with opencode-mem auto-capture |
| Session Compression Protocol | Protocol section in `director.md` | Redundant with opencode-mem auto-capture |
| Session Checkpoints | Section in `scribes.md` | Redundant with opencode-mem auto-capture |
| Session Compression Workflow | Section in `scribes.md` | Redundant with opencode-mem auto-capture |
| `.opencode/commands/remember.md` | Command file | Writes to `.opencode/memory/` (removed) |
| `.opencode/commands/recall.md` | Command file | Uses legacy `agentmemory` MCP tools |
| `resume` skill (`/.opencode/skills/resume/`) | Skill directory | Depends on checkpoints (removed) |
| `graphify-knowledge-flow` skill (`.opencode/skills/graphify-knowledge-flow/`) | Skill directory | Superseded by `preflight` skill (more comprehensive). 0 agents load it. |

**Checkpoint System — REMOVE:**

The checkpoint system (session checkpoints + session compression) is redundant with opencode-mem's auto-capture. When a session ends, opencode-mem already captures context. There is no need for a separate file-based checkpoint system.

Files to remove:
- `director.md` lines 520-578: "Session Checkpoint Protocol" and "Session Compression Protocol" sections
- `director.md` lines 504, 508: checkpoint references in Session Lifecycle
- `director.md` line 477: "To load session context: read .opencode/memory/checkpoints/"
- `scribes.md` lines 181-293: "Session Checkpoints" and "Session Compression Workflow" sections
- `scribes.md` lines 30, 96, 151, 163, 217, 435: checkpoint references throughout
- `resume/SKILL.md`: entire skill depends on checkpoints — **REMOVE the skill entirely**
- `skills/README.md`: remove resume skill entry
- `.opencode/skills/resume/` directory: delete

**Impact on `director.md` session lifecycle:**

Current:
```
1. Load context via graphify query + read design doc + ADRs + check state
1.5. Check prospective tasks
2. Load latest session summary: read .opencode/memory/checkpoints/<latest>.md
3. Listen for requests, map to trigger → team, delegate
4. Before transitions: Scribes store context, run gate, update state
5. On agent refusal: verify state, run gate, re-delegate
6. End: store checkpoint in .opencode/memory/checkpoints/, summarize
```

New:
```
1. Load context via graphify query + read design doc + ADRs + check state
1.5. Check prospective tasks
2. Search recent context: search_memories(query="<project> recent", scope="project")
3. Listen for requests, map to trigger → team, delegate
4. Before transitions: Scribes store context, run gate, update state
5. On agent refusal: verify state, run gate, re-delegate
6. End: summarize session (opencode-mem auto-captures context)
```

**`/remember` Command — REMOVE:**

The `/remember` command writes to `.opencode/memory/` subdirectories. Since `.opencode/memory/` is being removed, this command has no purpose. opencode-mem's `add_memory` replaces it.

**`/recall` Command — REMOVE:**

The `/recall` command uses `memory_smart_search` and `memory_lesson_recall` MCP tools — these are `agentmemory` MCP tools, not opencode-mem. They are legacy.

**`resume` Skill — REMOVE:**

The entire `resume/SKILL.md` skill depends on `.opencode/memory/checkpoints/`. Without checkpoints, the skill has no purpose. opencode-mem's `search_memories` replaces the functionality.

**`graphify-knowledge-flow` Skill — ARCHIVE/DELETE:**

The `graphify-knowledge-flow` skill exists but:
1. No agent loads it (0 references in any agent file)
2. Its functionality is fully covered by the `preflight` skill, which is more comprehensive (4 steps including cross-reference between graphify and opencode-mem)
3. The preflight skill is already loaded by director.md

The skill should be archived (moved to `docs/adr/archived/` or deleted) since preflight supersedes it.

**`preflight` Skill — NO CHANGES NEEDED:**

The `preflight` skill is the correct replacement for both:
- `graphify-knowledge-flow` (code context retrieval)
- The old agentmemory knowledge flow

It uses:
- Step 1: `graphify query` (code/structural context)
- Step 2: `search_memories` (runtime learnings from opencode-mem)
- Step 3: Cross-reference (merge graphify filenames into memory search)
- Step 4: Context assembly (include in Task prompt)

No changes needed to preflight — it's already correct.

## Consequences

### Positive

- Single memory system (opencode-mem) — no dual-system confusion
- Semantic search across all project knowledge
- Reduced maintenance: 100+ file-path references replaced with MCP tool calls
- Consistent memory interface across all agents
- No more missing directories (lessons/, releases/)
- Session resume now handled by opencode-mem search — no separate checkpoint system needed
- Removed redundant protocols: checkpoint, compression, `/remember`, `/recall`, `resume` skill

### Negative

- Migration effort: 17 files to migrate, 100+ reference sites to update
- Removed files/commands may require user retraining
- Design doc requires significant update

### Risks

- Data loss during migration if files are deleted before opencode-mem storage is verified
- Users familiar with `/remember` or `/recall` commands need to adopt `add_memory`/`search_memories`

## Effort

| Phase | Description | Effort |
|-------|-------------|--------|
| Phase 1 | Content migration (17 files) | Low — mechanical |
| Phase 2 | Agent file updates (100+ sites, remove checkpoint/compression sections) | High — 12 files |
| Phase 3 | Delete removed files (remember.md, recall.md, resume/, graphify-knowledge-flow/) | Low — deletion |
| Phase 4 | Design doc update | Medium — architecture section rewrite |
| Phase 5 | Cleanup | Low — delete directory |

**Total:** High effort, should be broken into sub-tasks.

## Verification

After migration:

```bash
# No remaining references to .opencode/memory
grep -rn "\.opencode/memory" .opencode/agents/ .opencode/skills/ .opencode/references/ AGENTS.md README.md scripts/ --include="*.md" --include="*.py"

# Legacy directory removed
ls .opencode/memory/ 2>&1  # should fail: "No such file or directory"

# Removed files/directories do not exist
ls .opencode/commands/remember.md 2>&1  # should fail
ls .opencode/commands/recall.md 2>&1   # should fail
ls .opencode/skills/resume/ 2>&1       # should fail
ls .opencode/skills/graphify-knowledge-flow/ 2>&1  # should fail (archived or deleted)

# Checkpoint/compression sections removed from agent files
grep -n "Session Checkpoint" .opencode/agents/director.md  # should return nothing
grep -n "Session Compression" .opencode/agents/director.md # should return nothing
grep -n "Session Checkpoint" .opencode/agents/scribes.md   # should return nothing
grep -n "Session Compression" .opencode/agents/scribes.md  # should return nothing

# opencode-mem has content (manual verification)
search_memories(query="checkpoint", scope="project")  # should return migrated checkpoints
search_memories(query="decision", scope="project")    # should return migrated decisions
```

## Relationship to Other ADRs

- **ADR-0050**: Proposed file-based `.opencode/memory/` storage — this ADR supersedes it
- **ADR-0051**: Established opencode-mem as memory system — this ADR completes the migration
- **ADR-0056**: Documented the dual-system finding (Category K) — this ADR provides the fix

## References

- ADR-0050: Agentmemory to OpenCode RAG Migration (proposed file-based storage)
- ADR-0051: opencode-mem Persistent Memory (superseded file-based storage)
- ADR-0056: Agent Configuration Cleanup (documented the finding, Category K)
- Design doc §7.2: Persistent Memory & Knowledge Retrieval
- `.opencode/commands/remember.md`: /remember command

## Review Notes

- **2026-07-18:** Initial version. Documents migration plan from `.opencode/memory/` to opencode-mem. Includes 5-phase migration, additional removals (checkpoint system, /remember, /recall, resume skill), and verification steps.
- **2026-07-18 (update):** Added `graphify-knowledge-flow` skill to Additional Removals (superseded by preflight). Added preflight skill validation note. Updated Phase 3 to include graphify-knowledge-flow deletion and skills/README.md update. Updated verification to check for graphify-knowledge-flow removal.
