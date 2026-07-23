# ADR-0055: KodeHold Improvement Opportunities

## Status

**Accepted** — 2026-07-21

## Context

A systematic investigation of the KodeHold project revealed 13 improvement opportunities across structural issues i design doc og agentfiler, kodekvalitet i scripts, og vedligeholdelsesproblemer i konfiguration og skills.

### Structural / Design Doc

1. **Design doc numbering broken** — `## 1.` through `## 11.` is used TWICE. Lines 144-154 are nested under `## 4. Design Document Lifecycle` but reuse top-level numbering (1-11), conflicting with the actual top-level sections.

2. **RTK section still prominent** — Design doc §7.3 devotes significant space to RTK with "40-60% token savings" claims, but RTK is only referenced in 2 of 9 agent files (engineers.md line 84, testers.md line 73). RTK is installed but may not be actively driving workflow.

### Code Quality (scripts/)

3. **gate.py main() is 160 lines** — Does arg parsing, self-mod check, dispatch to transitions, TWO separate reviewer mode output blocks (lines 672-678 and 732-744), result printing, marker cleanup, distillation marker creation. Much of this should be extracted.

4. **gate.py build_parser() is 70 lines** — Very large argument parser for a single script.

5. **Self-modification check duplicated** — Detection logic exists in both `output.py` (used by gate.py) AND `workspace.py` has its own self-mod handling. Only one should exist.

6. **Duplicate reviewer output in gate.py** — Lines 672-678 (inside self-mod block) and lines 732-744 (standard path) both output reviewer mode format. This is duplicated code that will drift apart.

### Agent Definitions

7. **"Memory Tools (opencode-mem)" section duplicated verbatim in 6 agents** — 16 lines × 6 = 96 lines of identical copy-paste in architects.md, director.md, engineers.md, fls.md, reviewers.md, testers.md. Any update to opencode-mem instructions must be made in all 6 files.

8. **director.md is 645 lines** — 34% of all agent file content. Contains 28 sections covering protocols, examples, state transitions, shipping gate, checkpointing. Several sections (Token Budget, Context Window, Shipping Gate) could be extracted to reference files or skills.

9. **CRLF line endings in 7 of 9 agent files** — Only `director.md` and `reviewers.md` use LF. The remaining 7 use Windows CRLF (`\r\n`). This causes noisy git diffs, inconsistent grep behavior, and `file` reporting warnings.

10. **graphify-knowledge-flow skill never loaded** — The skill exists at `.opencode/skills/graphify-knowledge-flow/SKILL.md` (50 lines) but 0 agents reference or load it. It was created per ADR-0054 but never wired into any agent's workflow.

11. **Skills README references deleted directory** — `.opencode/skills/README.md` lines 14-15 and 35 reference `agentmemory-knowledge-flow/` which was deleted but the README was not updated.

### Maintenance

12. **Config duplication** — Agent metadata (model, name, description, permissions) exists in both `.md` frontmatter AND `config/agents.yaml`. A sync script (`sync_agent_config.py`) exists but must be run manually. The two sources can and do drift apart.

13. **ponytail-audit run once, never since** — The skill was run once on Jun 28 (result in `.opencode/memory/metrics/`). It has 0 references in any agent file. Could be run periodically or integrated into CI.

### Key Forces

1. **Maintenance burden** — Every duplicated section (Memory Tools, reviewer output, argparser patterns) multiplies the cost of updates and creates drift risk.
2. **Onboarding friction** — CRLF line endings cause tooling issues for new contributors on Linux/macOS.
3. **Dead code exists** — Unreferenced skills, stale README entries, and duplicated self-mod checks add cognitive load.
4. **Single-file bottlenecks** — gate.py (759 lines) and director.md (645 lines) concentrate too much logic in one place.
5. **No automated drift detection** — Config duplication between .md frontmatter and agents.yaml requires manual sync enforcement.

## Decision

TBD — each finding has a proposed treatment below. This ADR documents all findings so the Director can execute them in priority order. No implementation work begins without a Director delegation cycle.

### Proposed Treatments

| # | Finding | Treatment | Effort | Risk |
|---|---------|-----------|--------|------|
| 1 | Design doc numbering | Flatten: promote nested sections to proper H3 under `## 4.` | Low | None |
| 2 | RTK prominence | Keep RTK section but reduce to note; it's still installed | Low | None |
| 3 | gate.py main() 160 lines | Extract reviewer output to `output.py`, extract self-mod block to `_handle_self_mod()` | Low | Low |
| 4 | build_parser 70 lines | Collapse repetitive flag definitions | Low | Low |
| 5 | Self-mod duplication | Consolidate all self-mod detection in `output.py`, remove from workspace.py | Low | Low |
| 6 | Duplicate reviewer output | Remove the inline block in self-mod path, let standard path handle it | Low | None |
| 7 | Memory Tools duplication | Extract to `.opencode/references/opencode-mem.md`, each agent loads via reference | Medium | Low |
| 8 | director.md 645 lines | Extract Shipping Gate, Token Budget, Context Window to reference files or skills | Medium | Low |
| 9 | CRLF line endings | Run `sed -i 's/\r$//'` on all 7 agent files | Low | None |
| 10 | graphify-knowledge-flow unused | Either wire it into agent workflows or archive it | Low | None |
| 11 | Skills README stale entry | Remove agentmemory-knowledge-flow reference | Low | None |
| 12 | Config duplication | Add a CI job that fails if `.md` and `agents.yaml` models don't match | Medium | Low |
| 13 | ponytail-audit unused | Either add to quality checklist or archive | Low | None |

## Consequences

### Positive

- ✅ **Reduced maintenance burden** across duplicated sections — editing one shared reference instead of 6 copies of Memory Tools
- ✅ **Cleaner git history** — no CRLF noise in diffs after conversion
- ✅ **Smaller, more focused files** — director.md and gate.py split into manageable pieces
- ✅ **Automated drift detection** between config sources catches inconsistencies early
- ✅ **Dead code cleaned up** — stale README references, unreferenced skills removed

### Negative

- ❌ **Structuring changes risk breaking agent loading** if frontmatter references are moved (mitigated by testing each change)
- ❌ **Some extraction work** (director.md sections, Memory Tools) requires updating cross-references across multiple files
- ❌ **CI drift check** adds complexity to the CI pipeline

### Neutral

- CRLF→LF conversion will create a one-time large diff across 7 agent files
- Extraction of shared sections changes file layout but not agent behavior
- Some findings (ponytail-audit, graphify-knowledge-flow) are "use or lose" decisions with no in-between

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CRLF→LF diff conflicts with in-flight PRs | Medium | Low | Schedule conversion when no PRs are open |
| Extracted references break agent loading | Low | Medium | Test that each agent loads after reference extraction |
| CI drift check causes false positives | Low | Low | Make the check a warning, not a hard failure |
| One-time large diff buries meaningful changes | High | Low | Do CRLF conversion as a standalone commit with `.git-blame-ignore-revs` |

## Alternatives Considered

### 1. Do nothing

Current state works. All agents load, gate.py runs, CRLF doesn't prevent operation.

**Rejected because:** Maintenance burden accumulates with every duplicate section edit. Each change to Memory Tools instructions requires touching 6 files. Dead code and stale references increase cognitive load over time.

### 2. Major restructure — split gate.py into `gate/` package

Create `gate/__init__.py`, `gate/transitions.py`, `gate/reviewer.py`, `gate/self_mod.py` — one file per transition plus shared utilities.

**Deferred:** Higher effort, better architecture. The proposed treatments focus on the most impactful extractions first (reviewer output, self-mod). Package restructure can follow if needed.

### 3. Remove RTK entirely

Delete design doc §7.3 and remove RTK references from agent files.

**Deferred:** RTK is still referenced in engineers.md and testers.md, suggesting it may be actively used by some teams. Reduce prominence first, revisit removal if usage drops to zero.

### 4. Auto-generate agent files from YAML

Eliminate config duplication entirely by making `.md` frontmatter the single source of truth and deprecating `config/agents.yaml`, or vice versa.

**Rejected because:** Both formats serve different consumers. `agents.yaml` is machine-readable for CI/tooling; `.md` frontmatter is for OpenCode agent loading. A CI drift check is a lighter-weight mitigation.

## Review Notes

- **2026-07-15:** Initial version. Documents 13 findings from systematic investigation. No implementation — this is a catalog ADR for Director prioritization. KodeHold self-mod — no gates apply.
- **2026-07-21 (Accepted):** All 13 findings implemented. Design doc numbering fixed, gate.py refactored (-43 lines), Memory Tools deduplicated (-85 lines, shared reference), director.md split (-63 lines), ponytail-audit archived, stale README removed. Self-mod protocol used (no gates).

## References

- ADR-0049: The Ladder (avoid over-engineering principle)
- ADR-0051: opencode-mem as KodeHold Persistent Memory Backend
- ADR-0054: Replace opencode-rag with Graphify Knowledge Graph
- `.opencode/agents/director.md` — current agent definitions
- `.opencode/skills/README.md` — stale skill references
- `scripts/gate.py` — 759-line gate script
- `config/agents.yaml` — agent metadata configuration

## Documentation

None required — this is an internal architecture ADR affecting no external tools or APIs. All changes are internal restructuring of KodeHold's own configuration, agent definitions, scripts, and documentation.
