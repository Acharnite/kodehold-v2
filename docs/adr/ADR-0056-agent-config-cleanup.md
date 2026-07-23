# ADR-0056: Agent Configuration Cleanup

## Status

**Accepted** — 2026-07-21

## Context

A comprehensive audit of all 11 agent configuration files (`AGENTS.md`, `director.md`, `architects.md`, `engineers.md`, `fls.md`, `reviewers.md`, `scribes.md`, `testers.md`, `second-opinion.md`, `second-opinion-fallback.md`, `kodehold-protocol.md`) was performed. The audit checked for contradictions, dead references, clarity issues, permission mismatches, cross-file consistency, and structural problems. Additional issues were found via graphify knowledge graph queries and opencode-mem search.

This ADR follows recent cleanup work that removed:
- Token budget protocol (ADR-0007 references)
- KODEHOLD_LIGHT references
- RTK binary and all references (ADR-0004 deprecated)
- YAML agent config system (`config/agents.yaml`, `agents.schema.json`, `sync_agent_config.py`, `validate_config.py`, `test_yaml_config.py`)

### Findings

#### Category A: Permission Contradictions

**A1. architects.md: write/edit permissions contradict constraint**
- Frontmatter (lines 9-10): `write: allow`, `edit: allow`
- Constraint (line 115): "Never directly modify files (design docs, ADRs, TODOs, agent configs). Return specifications as text via the Task tool; the Director delegates file changes to Scribes or Engineers."
- director.md (lines 407-413) reinforces: "Architects DESIGN only — they return specifications as text via the Task tool. The Director MUST delegate all file modifications to the appropriate team... Architects must NEVER directly edit files."
- Risk: An LLM may follow the permission system (which allows writes) over the constraint text.
- Fix: Change `write: deny` and `edit: deny` in architects.md frontmatter.

#### Category B: Missing Permissions

**B1. Three agents missing `skill: allow`**
- `architects.md` — references `state-awareness` skill (line 66) but no `skill: allow`
- `testers.md` — references `state-awareness` skill (line 35) but no `skill: allow`
- `scribes.md` — references `state-awareness` skill (line 64) but no `skill: allow`
- Compare: engineers.md, fls.md, reviewers.md, director.md all have `skill: allow`
- Risk: These agents cannot load skills at runtime. The `state-awareness` skill loading instruction is dead code.
- Fix: Add `skill: allow` to frontmatter of all three files.

#### Category C: Dead References

**C1. architects.md line 104: `workspace.sh adopt`**
- Should be `workspace.py adopt`. The file `workspace.sh` does not exist; `scripts/workspace.py` is the correct script.

**C2. fls.md line 32: truncated sentence**
- `"Document all fixes and decisions via Scribes (stored in )"` — empty parentheses, missing path.
- Should be: `"stored in .opencode/memory/"` or similar.

**C3. reviewers.md line 58: stale token budget reference**
- Checklist item: `"Token usage is within budget"` — token budgets were removed from the system.
- Fix: Remove or replace with qualitative check like "Output is concise."

**C4. Skills README references deleted `agentmemory-knowledge-flow/`**
- `.opencode/skills/README.md` lines 14-15 and 35 reference `agentmemory-knowledge-flow/` which was deleted.
- The directory does not exist on disk.
- Fix: Remove the reference from README.md.

#### Category D: Missing Directories

**D1. `.opencode/memory/lessons/` — referenced but does not exist**
- Referenced in: scribes.md (lines 60, 94, 150, 329), kodehold-protocol.md (line 41)
- Directory does not exist on disk.
- Fix: Create the directory, or update references to use existing directories.

**D2. `.opencode/memory/releases/` — referenced but does not exist**
- Referenced in: director.md (line 462), kodehold-protocol.md (lines 65, 76)
- Directory does not exist on disk.
- Fix: Create the directory, or update references to use existing directories.

#### Category E: Stale Content

**E1. kodehold-protocol.md lines 11-20: Token Budgets table**
- The "Token Budgets (per operation)" table defines specific token limits per operation type.
- Token budgets were removed from the system (director.md no longer has Token Budget Protocol).
- Fix: Remove the table or mark as historical.

**E2. scribes.md line 407: "Token Budget Enforcement" heading**
- The heading says "Token Budget Enforcement" but the content is about prospective memory task limits per priority level.
- Fix: Rename to "Prospective Task Limits" or "Task Budget by Priority".

**E3. scribes.md lines 219, 250, 269: token_usage.py references**
- References `python3 scripts/token_usage.py` for token metrics in checkpoint/compression templates.
- The script exists but is part of the removed token budget system.
- Fix: Remove or update these references.

#### Category F: Naming Mismatches

**F1. scribes.md taxonomy: `bug` vs `bugs/`**
- scribes.md line 48: type `bug` maps to `.opencode/memory/<type>/<slug>.md` → `.opencode/memory/bug/...`
- Actual directory on disk: `.opencode/memory/bugs/` (plural)
- Protocol memory tree (kodehold-protocol.md line 42): shows `bugs/` (plural)
- Fix: Update taxonomy to say `bugs` (plural) to match the actual directory.

#### Category G: Step Numbering Errors

**G1. engineers.md lines 84-85: step numbering jumps from 4 to 7**
- Missing steps 5 and 6. Confusing for LLM parsing numbered instructions.
- Fix: Renumber sequentially.

**G2. testers.md lines 74-75: step numbering jumps from 5 to 7**
- Missing step 6.
- Fix: Renumber sequentially.

#### Category H: Formatting Issues

**H1. CRLF line endings in 7 of 9 agent files**
- Files with CRLF: architects.md, engineers.md, fls.md, scribes.md, second-opinion.md, second-opinion-fallback.md, testers.md (mixed CRLF+LF)
- Files with LF: director.md, reviewers.md
- Risk: Noisy git diffs, inconsistent grep behavior, tooling issues on Linux/macOS.
- Fix: Convert all to LF with `sed -i 's/\r$//'`.

#### Category I: Ambiguity

**I1. director.md line 447: "All 6 teams" in Shipping Gate Phase 0**
- Shipping Gate says "All 6 teams approve or block" but there are 7 teams in the Available Teams table (lines 318-329): Architects, Engineers, Testers, Reviewers, Scribes, FLS, Second Opinion.
- Unclear whether Second Opinion is intentionally excluded.
- Fix: Clarify explicitly (e.g., "All 7 teams" or "All teams except Second Opinion").

**I2. scribes.md section naming inconsistency**
- Scribes uses "Persistent Memory (opencode-mem)" as section title.
- All other agents use "Memory Tools (opencode-mem)".
- Fix: Rename to "Memory Tools (opencode-mem)" for consistency.

#### Category J: graphify-knowledge-flow Skill Never Loaded

**J1. Skill exists but no agent loads it**
- `.opencode/skills/graphify-knowledge-flow/SKILL.md` exists (created per ADR-0054).
- 0 agents reference or load it.
- Fix: Either wire it into agent workflows (e.g., as part of preflight) or archive it.

#### Category L: Dead Skill References

**L1. `kodehold-routines/SKILL.md` line 103: `agentmemory-check` step**
The shipping gate routine table has a step `agentmemory-check` (step 5, director). This references the removed agentmemory system. The step should be removed from the routine table.
- File: `.opencode/skills/kodehold-routines/SKILL.md`
- Line: 103
- Current: `| 5 | director | agentmemory-check | (none) | 9 | No |`
- Fix: Remove this row from the table.

**L2. `skills/README.md`: references deleted `agentmemory-knowledge-flow/` directory**
The skills README still lists `agentmemory-knowledge-flow/` as a directory entry and in the table. The directory was deleted but the README was not updated.
- Already documented as C4 in this ADR, but the specific content needs updating.

#### Category K: `.opencode/memory/` Legacy System

**K1. File-based `.opencode/memory/` is a legacy system superseded by opencode-mem**

The design doc (`docs/design/README.md` line 368) explicitly states:
> "The file-based `.opencode/memory/` storage proposed in ADR-0050 §5 was never implemented and is superseded by opencode-mem per ADR-0051."

However, `.opencode/memory/` is referenced **100+ times** across active agent files, treating it as the primary memory system:

**Most affected files (reference counts):**
- `scribes.md`: ~50 references (entire memory taxonomy built around it)
- `director.md`: ~20 references (session lifecycle, transitions, checkpoints)
- `kodehold-protocol.md`: ~8 references (persistent storage convention, quality gate, shipping gate)
- `investigate/SKILL.md`: ~5 references (store bug findings)
- `resume/SKILL.md`: ~5 references (load checkpoints)
- `ponytail-audit/SKILL.md`: ~4 references (store audit metrics)
- `skills/README.md`: ~2 references
- `AGENTS.md`: 1 reference (Quick Reference)
- `README.md`: ~4 references (feature description, team description)
- `.opencode/commands/remember.md`: ~6 references
- `scripts/ship.py`: 1 reference

**The directory actually exists on disk** with subdirectories: `bugs/`, `checkpoints/`, `decisions/`, `fixes/`, `metrics/`, `patterns/`, `prospective/` — containing actual content from prior sessions.

**The contradiction:** opencode-mem MCP tools (`search_memories`, `add_memory`) are the intended memory system. Every agent already has "Memory Tools (opencode-mem)" sections instructing use of these MCP tools. But the file-based `.opencode/memory/` system is simultaneously described as the PRIMARY storage mechanism, creating a dual memory system where:
1. opencode-mem handles semantic search and auto-capture
2. `.opencode/memory/` handles structured docs (checkpoints, decisions, metrics, prospective tasks)

**The problem:** This dual system creates confusion about where data lives, duplicate storage patterns, and maintenance burden across 100+ reference sites.

**Recommended treatment:** This is a **major refactor** that should be its own ADR (not part of this cleanup ADR). The treatment should be:
1. Create a new ADR (e.g., ADR-0057) for migrating `.opencode/memory/` to opencode-mem
2. Migrate existing content from `.opencode/memory/` subdirectories to opencode-mem via `add_memory`
3. Update all agent files to use `search_memories`/`add_memory` instead of file paths
4. Handle special cases: checkpoints (session resume), prospective tasks (task queue), metrics (time-series)
5. Delete `.opencode/memory/` directory after migration
6. Update design doc to remove file-based storage references

## Decision

### Proposed Treatments

| ID | Finding | Treatment | Effort | Risk |
|----|---------|-----------|--------|------|
| A1 | architects write/edit permissions | Change to `write: deny`, `edit: deny` | Low | None |
| B1 | Missing skill: allow (3 agents) | Add `skill: allow` to architects, testers, scribes | Low | None |
| C1 | workspace.sh reference | Change to `workspace.py` | Low | None |
| C2 | fls.md truncated sentence | Complete with `.opencode/memory/` | Low | None |
| C3 | Token budget checklist item | Remove from reviewers.md | Low | None |
| C4 | Skills README stale entry | Remove agentmemory-knowledge-flow reference | Low | None |
| D1 | Missing lessons/ directory | Create `.opencode/memory/lessons/` | Low | None |
| D2 | Missing releases/ directory | Create `.opencode/memory/releases/` | Low | None |
| E1 | Token Budgets table in protocol | Remove section from kodehold-protocol.md | Low | None |
| E2 | "Token Budget Enforcement" heading | Rename to "Prospective Task Limits" | Low | None |
| E3 | token_usage.py references in scribes | Remove from checkpoint/compression templates | Low | None |
| F1 | bug vs bugs naming | Update taxonomy to `bugs` (plural) | Low | None |
| G1 | engineers.md step numbering | Renumber sequentially | Low | None |
| G2 | testers.md step numbering | Renumber sequentially | Low | None |
| H1 | CRLF line endings | Convert all to LF | Low | None |
| I1 | "All 6 teams" ambiguity | Clarify team count in shipping gate | Low | None |
| I2 | Scribes section naming | Rename to "Memory Tools (opencode-mem)" | Low | None |
| J1 | graphify-knowledge-flow unused | Wire into preflight or archive | Medium | Low |
| K1 | .opencode/memory/ legacy system | Create ADR-0057 for full migration to opencode-mem. This ADR documents the finding; implementation is deferred. | High | Medium |
| L1 | agentmemory-check in routines | Remove row from kodehold-routines/SKILL.md shipping gate table | Low | None |
| L2 | Skills README stale entry | (already C4) — remove agentmemory-knowledge-flow references | Low | None |

### Relationship to ADR-0055

ADR-0055 (KodeHold Improvement Opportunities) documented 13 findings. Several overlap with this ADR:

| ADR-0055 Item | This ADR | Status |
|---------------|----------|--------|
| #2 RTK prominence | Removed in prior cleanup | Done |
| #7 Memory Tools duplication | Not addressed here (extraction to shared ref) | Deferred |
| #8 director.md too long | Not addressed here (extraction to skills) | Deferred |
| #9 CRLF line endings | H1 in this ADR | This ADR |
| #10 graphify-knowledge-flow unused | J1 in this ADR | This ADR |
| #11 Skills README stale | C4 in this ADR | This ADR |
| #12 Config duplication | Removed YAML system in prior cleanup | Done |

Items #1 (design doc numbering), #3-6 (gate.py code quality), #13 (ponytail-audit) are not addressed in this ADR and remain as future work.

**New finding from graphify + opencode-mem search:** Category K (`.opencode/memory/` legacy system) was discovered via graphify knowledge graph queries and opencode-mem search during this audit. The file-based memory system is referenced 100+ times across agent files but contradicts the design doc which states it was never implemented. This finding requires its own ADR (ADR-0057) due to its scope.

## Consequences

### Positive

- All agent files have consistent, non-contradictory permissions
- All agents can load skills they reference
- No dead references remain in active files
- Missing directories are created or references updated
- Consistent line endings across all files
- Clear step numbering in all workflow sections
- `.opencode/memory/` legacy system documented for migration (ADR-0057)

### Negative

- 18 low-effort changes across 9 files (batch operation)
- CRLF conversion may produce noisy git diffs on first commit
- K1 (memory migration) is deferred — creates continued dual-system maintenance burden until ADR-0057 is implemented

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CRLF conversion conflicts with in-flight work | Medium | Low | Convert as standalone commit with `.git-blame-ignore-revs` |
| Permission change blocks architects from legitimate writes | Low | Medium | architect frontmatter change is intentional per director.md constraint |
| Dead reference fixes miss edge cases | Low | Low | Verify with grep after each batch of changes |

## Verification

After implementation, run:
```bash
# No dead references
grep -rn "workspace\.sh\|token budget\|KODEHOLD_LIGHT\|rtk\|RTK\|agents\.yaml" .opencode/agents/ .opencode/references/ --include="*.md" | grep -v "docs/adr/"

# All agents have skill: allow
grep -l "skill: allow" .opencode/agents/*.md

# No CRLF
file .opencode/agents/*.md | grep -i crlf

# Directories exist
ls -d .opencode/memory/lessons/ .opencode/memory/releases/
```

## Review Notes

- **2026-07-18:** Initial version. Documents 18 findings from comprehensive audit of all 11 agent configuration files. All treatments are low-effort except J1 (medium). No gates apply — KodeHold self-mod documentation task.
- **2026-07-18 (update):** Added Category K — `.opencode/memory/` legacy system discovered via graphify knowledge graph queries and opencode-mem search. This is a major finding (100+ reference sites across 12+ files) requiring separate ADR-0057 for migration. Total findings now: 19 (18 original + 1 major). Treatment table updated with K1 row.
- **2026-07-18 (update 2):** Added Category L — Dead Skill References from graphify knowledge graph queries. L1: `agentmemory-check` step in shipping gate routine (kodehold-routines/SKILL.md). L2: Skills README stale `agentmemory-knowledge-flow` entry (already C4). Total findings now: 21. Treatment table updated with L1, L2 rows.
- **2026-07-21 (Accepted):** All 21 findings implemented. A1 (permissions), B1 (skill:allow), C1-C4 (dead refs), D1-D2 (directory refs cleaned), E1-E3 (stale content removed), F1 (taxonomy), G1-G2 (step numbering), H1 (CRLF), I1 (team count clarified), I2 (heading consistency), J1 (graphify-knowledge-flow archived), K1 (.opencode/memory/ migrated), L1-L2 (skill refs cleaned).

## References

- ADR-0049: The Ladder (lazy senior dev philosophy)
- ADR-0050: Memory system proposal (file-based, never implemented)
- ADR-0051: opencode-mem adoption (supersedes file-based memory)
- ADR-0054: Replace opencode-rag with Graphify Knowledge Graph
- ADR-0055: KodeHold Improvement Opportunities (overlapping findings)
- ADR-0057: (planned) Migrate `.opencode/memory/` to opencode-mem
- `.opencode/agents/` — all 11 agent configuration files
- `.opencode/skills/README.md` — stale skill references
- `.opencode/skills/graphify-knowledge-flow/SKILL.md` — unused skill
- `docs/design/README.md` line 368 — design doc states file-based memory was never implemented

## Documentation

None required — this is an internal cleanup ADR affecting no external tools or APIs. All changes are internal restructuring of KodeHold's own agent configuration files.
