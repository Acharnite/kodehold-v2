# Changelog

## 1.24.0 — 2026-07-22

### Added
- **Loop system cleanup + workspace rewrite** — Removed deprecated loop skills (loop-budget, loop-constraints, loop-triage), config files (gate.yaml, workspaces.yaml), and stale documentation (LOOP.md, STATE.md, safety.md). Moved ADR-0058/0059 to inactive/, added ADR-0060. workspace.py rewritten with full workspace management. Added workspace-loop-management skill and discord-notify.py.

### Changed
- **director.md, AGENTS.md, opencode.json** — Updated for new loop integration with workspace-loop-management skill.

### Removed
- **Deprecated loop infrastructure** — 3 loop skills, 2 config files, 4 documentation files, patterns/registry.yaml, loop-run.sh.

## 1.23.0 — 2026-07-22

### Added
- **Discord webhook notifications** — `scripts/discord-notify.py` sends color-coded embeds (green=clean, orange=issues, red=error) to Discord webhook after loop runs.
- **ADR-0059: Workspace as Mini-KodeHold** — Accepted. workspace.py rewritten with 10 commands (init, adopt, list, state, loop, gate, deploy-ready, migrate, deinit, ensure-git). Copy mode default, `--link` for legacy symlinks. YAML registry in `config/workspaces.yaml`.

### Changed
- **ADR-0007 deprecation** — All references to ADR-0007 removed from active operational files (design doc, TODO.md, CHANGES.md).
- **`.opencode/memory/` deprecation** — All references to file-based memory removed from active operational files.

### Removed
- `scripts/loop-run.sh` — replaced by `scripts/loop_runner.py` (backwards-compat wrapper retained).

## 1.22.0 — 2026-07-14

### Changed
- Removed all `.sh` scripts — `gate.sh`, `workspace.sh`, `ship.sh`, `benchmark.sh`,
  `sync-agent-config.sh`, `validate-config.sh`, `token-usage.sh`,
  `detect-test-framework.sh`, and `scripts/lib/output.sh` deleted
- Updated all agent files (director.md, reviewers.md, scribes.md, engineers.md,
  fls.md, testers.md) to reference `.py` scripts directly
- Updated `resume/SKILL.md` and `state-awareness/SKILL.md` to use `gate.py --status`
- Updated `lib/output.py` — removed `.sh` entries from `KODEHOLD_SYSTEM_PATHS`
- All commands now use `python3 scripts/gate.py`, `python3 scripts/workspace.py`,
  `python3 scripts/ship.py`, etc.

## 1.20.0 — 2026-07-14

### Added
- **ADR-0054: OpenCode RAG → Graphify migration** — Replaces the standalone `opencode-rag mcp` MCP server with Graphify knowledge graph for deterministic, tree-sitter-based code retrieval
- **Graphify knowledge graph** — Primary code retrieval tool for structural queries (call graphs, imports, class hierarchies, symbol definitions)
- **Documentation section** per ADR-0048 §3 — mandatory tool documentation added to ADR-0054

### Changed
- **Code retrieval architecture** — Graphify replaces opencode-rag as the sole code retrieval method. Built-in tools (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) are platform-level primitives, not part of KodeHold's documented workflow
- **opencode.json cleanup** — Removed `krypto-agent` MCP server entry and stale `rag-plugin.js`/`rag-tui.js` plugin references
- **Design doc §5, §7.2** — Updated with ADR-0054 entry and Graphify as primary retrieval layer
- **VERSION.md** — Bumped 1.19.0 → 1.20.0

### ADRs
- **ADR-0054** — Accepted: Replace opencode-rag with Graphify Knowledge Graph for Code Retrieval

## 1.19.0 — 2026-07-09

### Changed
- **Design doc Section 8.1 (Bring Your Own Model)** — vLLM documented as recommended local inference provider alongside Ollama. Added concurrent LLM + Embedding serving architecture: dual vLLM instances (LLM on port 8000, embedding on port 8001) for projects requiring simultaneous generation and embeddings (e.g., RAG pipelines). References ADR-0053 (Replace Ollama with vLLM for Concurrent LLM + Embedding Serving).

## 1.12.8 — 2026-06-06

### Added
- **Memory Types tab in viewer:** New tab showing all semantic facts and procedural memories with search/filter capabilities. Dashboard cards now link to full view when >5 items exist.

## 1.12.3 — 2026-06-04

### Added
- **Project slug migration (Phase 4, ADR-0036):** All agentmemory records migrated from full filesystem paths to canonical slugs. Used Python `iii-sdk` to connect directly to iii-engine WebSocket — ensures correct binary serialization of `.bin` files.
- **Migration audit log:** `scripts/migrations/slug-migration-20260604.log` documents every change.

### Changed
- **366 session metadata entries** updated (`project` + `cwd` fields from `/home/kiffer/project/<slug>` → `<slug>`)
- **6,552 observations** recursively migrated (path references in structured data fields)
- **3 profiles merged** (old absolute-path entries merged into slug entries, old entries deleted)
- **5 corrupted entries cleaned** (orphaned keys, markdown in project field, empty projects)

### Fixed
- ***does_not_exist*** orphaned session entry removed from state store
- Session with corrupted project name (markdown output captured as identifier) corrected to `bob`
- Profiler profiles for irrelevant projects deleted (`/tmp/agentmemory-demo`, `/home/kiffer/project`, `.agentmemory-last-project`)

### Architecture
- Discovery: agentmemory stores observations in separate scope `mem:obs:<session_id>` from session metadata (`mem:sessions`)
- Discovery: Direct `.bin` file manipulation corrupts 9-byte binary footer — always use `state::set` via iii-sdk
- ADR-0036 promoted to Accepted (Phase 4 complete)

## 1.12.2 — 2026-06-03

### Added
- **Summarization quality pipeline fixes** — 3 fixes to agentmemory source code (`/home/kiffer/project/agentmemory/src/`):
  - **Anti-pattern detection (prompt-leakage)** — `isPromptLeakage()` function in `src/functions/summarize.ts` detects 6 patterns where the LLM regurgitates prompt text instead of generating content (e.g., "Short session title", "max 100 chars", backtick-prefixed titles, structural keyword overloading). `parseSummaryXml` calls this before returning — if leakage is detected, returns `null` to trigger the retry loop.
  - **QualityScore penalty** — `scoreSummary()` in `src/eval/quality.ts` applies -80 penalty for prompt-leaked titles, clamped to 0. Previously, prompt-regurgitated summaries scored 95-100 because only structural checks (length, non-empty arrays) were performed.
  - **Deduplication** — `registerSummarizeFunction` in `src/functions/summarize.ts` checks `KV.summaries` for an existing summary before running the LLM. Prevents the same session from being summarized 5-6 times due to three independent triggers (idle, compacted, stopped).
- **Build verification** — `npm run build` passed with 0 errors; 1366/1379 tests passing (13 pre-existing failures, unrelated).

## 1.12.1 — 2026-06-03

### Changed
- **Agentmemory upgraded v0.9.24 → v0.9.25** — 5 obsolete patches removed (triggerVoid, summary XML parse, viewer-bind, merged). All upstream bug fixes from our reports now included: triggerVoid migration (PR #773), summary XML markdown fence parsing (PR #791), graph pagination, sharded index persistence, smart-search diagnostics, cross-project memory leakage fix, consolidation auto-enable, and 0 npm audit vulnerabilities.
- **Viewer bind** — now via `AGENTMEMORY_VIEWER_HOST=0.0.0.0` env var instead of direct dist patch. New minimal `agentmemory-viewer-bind-0.9.25.patch` bypasses upstream AGENTMEMORY_SECRET requirement for non-loopback binds.
- **Patches archived** — old v0.9.24 patches moved to `patches-v0.9.24/` for historical reference.

### Fixed
- **CHANGES.md format** — removed `v` prefix from version headers to match ship.sh parsing.

## 1.12.0 — 2026-06-02

### Added
- **YAML-based agent & task configuration** (Issue #34, ADR-0037)
  - Phase 1: `config/agents.yaml` (8 agents), `config/agents.schema.json`, `config/tasks.yaml` (4 workflows, 5 gates)
  - Phase 1: `scripts/validate-config.sh`, `scripts/sync-agent-config.sh`
  - Phase 4: `tests/init/test_yaml_config.py` (46 tests)

## 0.19.2 — 2026-06-02

### Added
- **ADR-0036: Project Slug Convention** (Accepted) — project identifiers migrated from filesystem paths to stable slugs. Director protocol, workspace scripts, and design docs updated.
- **Custom KodeHold viewer server** — `tools/viewer/serve.mjs` binds port 3115, serves custom viewer HTML + proxies agentmemory API.
- **Slots tab** — added to custom KodeHold viewer (pending_items, session_patterns, project_context).
- **`scripts/migrate-project-slugs.sh`** — data migration for path-style → slug project identifiers.
- **`scripts/validate-slugs.sh`** — slug validation script, referenced in ADR-0036 for CI integration.

### Changed
- **Director protocol** — `project` field now uses stable slug instead of `process.cwd()`/filesystem path.
- **Workspace scripts** — `workspace.sh` validates project names as slugs; catalog uses `project: name` instead of `path: filesystem_path`.
- **Design doc** — updated to v1.10.2 with ADR-0036 entry.
- **Viewer** — Slots tab added (pending_items, session_patterns, project_context).
- **`.opencode/agents/director.md`** — project parameter changed to slug.
- **`scripts/workspace.sh`** — added `validate_slug()`, catalog now uses `project` field.
- **`docs/design/README.md`** — ADR-0036 entry + changelog v1.10.2.
- **`docs/adr/README.md`** — ADR-0036 entry.
- **`tools/viewer/index.html`** — Slots tab.

## 0.19.1 — 2026-06-02

### Fixed
- **CI: ADR format smoke test (03-adr-format.sh)** — Replaced `echo "$content" | grep -q` with direct `grep -q ... "$f"` calls, removing unused `content` variable. Fixes `set -euo pipefail` + `grep -q` SIGPIPE antipattern that falsely reported ADR-0029 as missing Status section in CI.
- **CI: Agentmemory health check (02-icm-check.sh)** — Three-way logic: connection refused (`curl` status `000`) → WARN (not FAIL), 2xx → PASS, 4xx/5xx → FAIL. Allows init tests to pass in CI where agentmemory daemon is not running, per ADR-0029.
- **CI: Removed obsolete ICM setup steps** — Removed "Setup ICM" and "Bootstrap ICM database" steps from `.github/workflows/kodehold-ci.yml`. Added comment explaining removal per ADR-0029 (agentmemory migration).
- **Agentmemory summary XML parsing** — Agentmemory v0.9.24 intermittently logged `[agentmemory] warn Failed to parse summary XML` (28 occurrences across 15+ sessions). Root cause: LLM wraps summary XML in markdown code fences (` ```xml...``` `) that the regex-based `getXmlTag` parser can't handle, and the final summarize call had no retry mechanism. Fix: `parseSummaryXml` now strips markdown code fences and extracts raw XML before regex parsing; the final summarize call now has a retry loop (2 attempts). Applied directly to `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/index.mjs`. Service restarted via `systemctl --user restart agentmemory`. **Caveat:** `npm update -g @agentmemory/agentmemory` will overwrite the patch.
- **ADR test scripts** — `03-adr-format.sh` and `03-adr-index.sh` now skip `.original.md` files in their loops, preventing false failures from agentmemory compression backups.

## 0.19.0 — 2026-06-01

### Added
- **ICM → Agentmemory Migration Complete (Phases 1-5)** — Full replacement of ICM memory system with agentmemory throughout KodeHold
- **ADR-0030** — Agentmemory Knowledge Flow (3-mode protocol: Pre-task/Post-task/Full, replaces icm-knowledge-flow skill)
- **ADR-0031** — Actions + Crystals for Director Delegation (10 action types, dependency model, frontier flow, lease management)
- **ADR-0032** — Routine Templates (4 standard flow templates: adr-flow, implement-flow, bugfix-flow, ship-gate)
- **ADR-0033** — Crystals + Signals (auto-crystallize triggers, 5 inter-agent signal types)
- **Action Frontier Protocol** — Director now uses `memory_frontier` + `memory_lease` + `memory_crystallize` instead of manual todowrite
- **Routine Templates** — Director can instantiate standard workflows with single `memory_routine_run` call
- **Auto-Crystallize** — 4 triggers (every 5 actions, state transitions, routine completion, explicit)
- **Inter-Agent Signaling** — 5 signal types (info, request, response, alert, handoff) with routing rules
- **agentmemory-knowledge-flow skill** — New skill replacing icm-knowledge-flow

### Changed
- **director.md** — Todo Sequence Protocol replaced by Action Frontier Protocol with 7-step delegation flow
- **scribes.md** — Added Action Management, Crystal Consumption, and Signal Handling sections
- **All 6 team agents** — ICM Knowledge Flow → Agentmemory Knowledge Flow references updated
- **gate.sh + ship.sh** — ICM health checks replaced with agentmemory curl health checks
- **kodehold-protocol.md + AGENTS.md** — All ICM references updated to agentmemory
- **scripts/benchmark.sh + scripts/consolidate-all.sh** — Deprecated (agentmemory auto-consolidates)

### Deprecated
- **ADR-0027** — ICM Knowledge Flow Invocation Modes (replaced by ADR-0030)
- **scripts/benchmark.sh** — ICM benchmarks, no agentmemory equivalent
- **scripts/consolidate-all.sh** — Agentmemory auto-consolidates
- **icm-knowledge-flow skill** — Replaced by agentmemory-knowledge-flow

## 0.17.2 — 2026-06-01

### Fixed
- **OpenCode startup session errors** — Removed broken gstack skill symlinks (`~/.config/opencode/skills/gstack/`) that caused 11 `ENOENT` errors per startup (P0, FLS fix). Ghost session accumulation (168 sessions) caused by agentmemory daemon v0.9.24 calling removed `sdk.triggerVoid()` from iii-sdk v0.11.2 — resolved via upstream PR #731 (P1, user fix). LLM circuit breaker timeouts handled independently (P2, user fix).

## 0.17.1 — 2026-06-01

### Fixed
- **PR #749 CodeRabbitAI review** — Added runtime validation for `info?.directory` in agentmemory-capture.ts (replaces type assertion with proper `typeof` + length check)

## 0.17.0 — 2026-05-30

### Changed
- **ICM memoir restructure** — 7 team-specific memoirs (`kodehold-architects`, `kodehold-engineers`, `kodehold-testers`, `kodehold-reviewers`, `kodehold-scribes`, `kodehold-fls`, `kodehold-arch`) merged into single `kodehold-teams` memoir (27 concepts, 16 links)
- **Learnings consolidation** — `kodehold` and per-team learnings merged into `kodehold-learnings` (63 concepts, 68 links)
- **ADR-0027, ADR-0023, ADR-0009** — all references to per-team memoirs updated to `kodehold-teams` and `kodehold-learnings`
- **FLS added to kodehold-teams** — `kodehold-teams` memoir now covers all 6 teams plus KodeHold architecture
- **All agent configs** — updated to reference new consolidated memoir structure

### Lifecycle
- CLOSED→REOPEN→ACTIVE lifecycle transition completed

## 0.16.0 — 2026-05-29

### Added
- **Lifecycle simulation document** — `docs/simulations/lifecycle-simulation.md` with 4 complete scenarios: new project (30 steps), bug fix via FLS (6 steps), large feature reopen (26+ steps), adopted project (9+ steps). Includes Actor Reference, State Reference, Gate Reference, Cross-Reference Matrix, and Marker Lifecycle diagrams.
- **ADR-0021 Prospective Memory design** — implementation design for deferred + recurring tasks via ICM memories. Accepted by Reviewers (Gate 1 PASS). Storage in `kodehold-<project>-prospective` topic, session-start task check, Scribes-managed CRUD.

### Changed
- **ADR status cleanup** — 5 ADRs resolved: ADR-0020 Superseded (ICM decay system), ADR-0022 Superseded (ICM plugin hooks), ADR-0023 Superseded (ICM plugin + memoir), ADR-0024 Deprecated (over-engineering), ADR-0025 Deprecated (over-engineering)
- **Director delegation lesson** — Added file modification delegation rule: Architects never modify files directly, all changes via Scribes
- **Architects agent** — Added explicit constraint: "Never directly modify files"

### Fixed
- **OPENCODE_NONINTERACTIVE env leak** — gate marker enforcement test now isolates env with `env -u OPENCODE_NONINTERACTIVE` to prevent false failures when parent env sets the variable
- **Redundant file cleanup** — `docs/icm-knowledge-flow.md` deleted (content preserved in ADR-0027)

## 0.15.2 — 2026-05-29

### Fixed
- **ADR-0027 implementation review fixes** — SKILL.md renumbered to match ADR-0027 step numbering (steps 1-2 Pre-task, steps 4-8 Post-task, step 3 removed from knowledge flow). Fixed `memoit=` typo to `memoir=`. Added Full mode to Mode Selection table.
- **docs/icm-knowledge-flow.md** — updated with ADR-0027 reference, 3 invocation modes documented, Scribes Post-task-only documented, consolidation threshold unified to >7 entries (was >5).
- **second-opinion.md** — updated ICM Knowledge Flow section to declare Post-task mode per standard format.

## 0.15.1 — 2026-05-29

### Changed
- **Agent files updated with ADR-0027 invocation modes** — all 6 agent files now specify Pre-task mode (steps 1-2) for ICM Knowledge Flow. Scribes uses Post-task only (steps 4-8). Fixes token waste and semantic confusion from running search steps immediately before store.

## 0.15.0 — 2026-05-29

### Added
- **ADR-0027** — ICM Knowledge Flow Invocation Modes (Proposed). Defines three invocation modes (Pre-task, Post-task, Full) for the ICM Knowledge Flow skill, replacing the generic "execute each step" approach. Scribes uses Post-task only (no search steps); other teams use Pre-task default. Fixes token waste and semantic confusion from running search steps immediately before store.

### Fixed
- **Consolidation threshold inconsistency in ADR-0027** — step 5 in the Proposed SKILL.md section said ">5 entries" while the Context section said ">7 entries". Unified to ">7 entries" (canonical value per ICM docs).

## 0.14.0 — 2026-05-29

### Added
- **Dedicated second-opinion subagent** — created second-opinion subagent with Google Gemma 3 12B via OpenRouter, OpenRouter configured as provider in opencode.json, Director routes second opinions to dedicated subagent, Reviewers.md cleaned up (protocol removed, delegation pointer added). Fixes [#12](https://github.com/Acharnite/kodehold/issues/12).

### Changed
- `opencode.json` — OpenRouter provider added, second-opinion subagent registered
- `.opencode/agents/reviewers.md` — second opinion protocol removed, delegation pointer added
- `.opencode/agents/director.md` — second opinion routing to dedicated subagent

### Tested
- 12/12 tests PASS, Review: PASS

## 0.13.4 — 2026-05-29

### Fixed
- **ICM Knowledge Flow skill missing frontmatter** — added YAML frontmatter with `name` and `description` fields to `.opencode/skills/icm-knowledge-flow/SKILL.md`. Skill now registers correctly with OpenCode's skill discovery. Fixes #13 follow-up.

## 0.13.3 — 2026-05-29

### Fixed
- **Shipping gate alignment** — AGENTS.md and ship.sh now agree on step count (8 total: 1 manual Team Meeting + 7 automated in ship.sh). Team Meeting (step 0) clarified as manual pre-requisite. CHANGES.md check in ship.sh upgraded from `warn` to `fail` — shipping without a changelog entry is now blocked.

### Changed
- `AGENTS.md` — Shipping Gate section expanded from one-liner to structured table with 8 steps (manual/automated distinction)
- `scripts/ship.sh` — CHANGES.md check: `warn` → `fail`; header comment updated to document 8-step process

## 0.13.2 — 2026-05-29

### Fixed
- **FLS ICM protocol inconsistency** — removed direct `icm_memory_store` call from FLS workflow (step g). FLS now delegates ICM storage to Scribes via Director, consistent with ADR-0010 ("FLS requests Scribes via Director") and director.md delegation patterns ("fls → scribes (post-task)"). Fixes bug #15.

## 0.13.1 — 2026-05-29

### Fixed
- **FLS ICM protocol inconsistency** — removed direct `icm_memory_store` call from FLS workflow (step g). FLS now delegates ICM storage to Scribes via Director, consistent with ADR-0010 ("FLS requests Scribes via Director") and director.md delegation patterns ("fls → scribes (post-task)"). Fixes bug #15.

## 0.13.0 — 2026-05-29

### Added
- **Dependabot configuration** — `.github/dependabot.yml` for weekly automated dependency updates: GitHub Actions (version + security) and npm packages in `.opencode/`

## 0.12.0 — 2026-05-29

### Changed
- **GitHub MCP server upgrade** — replaced deprecated `@modelcontextprotocol/server-github` npm package with official `github/github-mcp-server` Go binary v1.1.2. Binary installed at `/home/kiffer/.local/bin/github-mcp-server`. Resolves intermittent authentication failures ([#25](https://github.com/Acharnite/kodehold/issues/25))

### Closed
- GitHub issue [#22](https://github.com/Acharnite/kodehold/issues/22) — stale test count (fixed in v0.11.0)
- GitHub issue [#25](https://github.com/Acharnite/kodehold/issues/25) — GitHub MCP auth failures (deprecated package replaced)

## 0.11.0 — 2026-05-29

### Fixed
- **Test count** — TODO.md and README.md: corrected "10 tests" → "12 tests" (4 smoke + 3 init + 5 integration)
- **Light mode checkbox** — TODO.md: parent item marked [x] (all 4 sub-items already done)
- **Design doc version consistency** — header bumped from 1.4.8 to 1.4.10 to match changelog (v1.4.9 ADR entry existed but header was stale)
- **README.md ADR range** — updated from "ADR-0001 through ADR-0011" to "ADR-0001 through ADR-0025"

### Closed
- GitHub issue [#4](https://github.com/Acharnite/kodehold/issues/4) — GitHub integration (implemented)
- GitHub issue [#5](https://github.com/Acharnite/kodehold/issues/5) — ICM context summaries (implemented)

## 0.10.0 — 2026-05-29

### Added
- **ADR status fix** — ADR-0015, ADR-0016, ADR-0019 statuses corrected across all docs (Proposed→Accepted)
- **`--project-path` parameter for gate.sh** — gate.sh now accepts `--project-path` flag to check markers in workspace directories instead of main project root, fixing false negatives for workspace/adopted projects ([#8](https://github.com/Acharnite/kodehold/issues/8))

### Changed
- **Commit protection protocol** — Director and Scribes now have explicit rules: new ADR, design, and config files must be committed before session end or checkpoint. Added to Director Core Protocol, Scribes Post-Task workflow, and design doc ([#23](https://github.com/Acharnite/kodehold/issues/23))
- **git clean -fd protection** — added explicit prohibition in Director, AGENTS.md, and design doc: `git clean -fd` must never be used without explicit user permission ([#3](https://github.com/Acharnite/kodehold/issues/3))

## 0.9.0 — 2026-05-27

### Added
- **Gate markers for quality enforcement** — `.design_reviewed` (Reviewers approve design before INIT→ACTIVE), `.impact_analysis_done` (Architects assess scope before CLOSED→REOPEN)
- **User review stop at INIT→ACTIVE** — gate presents design doc + ADR list and asks for confirmation before proceeding
- **Session checkpoint protocol** — Director stores checkpoint in ICM every ~8 delegations. Scribes can resume from checkpoint
- **`--yes` flag** for `scripts/gate.sh` — skip interactive prompts for CI/automation
- **Compaction config** — `opencode.json` now has `compaction: { auto: true, prune: true, reserved: 7000 }`
- **`webfetch`/`websearch` permissions** for Architects agent — research before designing

### Changed
- `scripts/gate.sh` — `init_to_active()` shows design doc + ADRs and prompts user before transition; `closed_to_reopen()` checks `.impact_analysis_done`; `review_to_closed()` cleans up all lifecycle markers
- `.opencode/agents/reviewers.md` — creates `.design_reviewed` after approving design
- `.opencode/agents/architects.md` — step 10: creates `.impact_analysis_done` after impact analysis
- `.opencode/agents/director.md` — trimmed 301→153 lines (49% reduction); added session checkpoint protocol
- `.opencode/agents/scribes.md` — session checkpoint store/resume workflow
- `opencode.json` — compaction config added
- Team Meeting (ADR-0011) exercised on radarr-lang-router — 6 teams, 4 approve + 2 approve-with-concerns

## 0.8.0 — 2026-05-27

### Added
- **FLS-specific tests** — `tests/integration/04-fls-workflow.sh` with 7 test areas (triage criteria, workflow, state restrictions, skill references, ICM docs, permissions), 50+ assertions
- **Shared `.venv`** — KodeHold root `.venv/` with pytest, pyyaml, requests installed. Agents now reference `.venv/bin/pytest` consistently instead of creating temp venvs
- **ADR-0013** — Investigate Skill: Systematic Debugging (Accepted). Documents 4-phase methodology, gstack adaptations, and agent integration

### Changed
- **English-only subagent prompts** — Director Core Protocol rule #6: "ALWAYS write subagent prompts in English only". Bold warning in Delegation Pattern section
- `scripts/workspace.sh` — removed wasteful `find` call in `ws_adopt()` that returned 0 through symlinks
- `.opencode/agents/testers.md` — references KodeHold root `.venv/bin/pytest` instead of "if a venv exists"
- `.opencode/agents/fls.md` — references `.venv/bin/pytest` for test verification; fixed workflow numbering (missing "4. If Major" heading)
- `.opencode/agents/engineers.md` — added step for systematic debugging via investigate skill
- `.opencode/agents/director.md` — added `Investigate / root cause` trigger mapping; `skill: allow` permission
- `.opencode/agents/reviewers.md` — added `skill: allow` permission
- `.gitignore` — added `.venv/`, `__pycache__/`, `*.pyc`
- `docs/design/README.md` — v1.0 → 1.1, Draft→Active, added §7.4 Skills System, updated file layout
- `docs/adr/README.md` — ADR-0013 added to index

## 0.7.0 — 2026-05-27

### Added
- **Investigate skill** — `.opencode/skills/investigate/SKILL.md` with 4-phase systematic debugging adapted from gstack. Iron Law, pattern analysis, 3-strike rule, regression test requirement, structured debug report, ICM storage ([#2](https://github.com/Acharnite/kodehold/issues/2))
- ACTIVE→REVIEW gate enforces Test→Review sequence — checks `.testers_done` marker, blocks if missing
- Design doc discipline — all 7 agents now update the design doc after completing work
- Pytest best practices codified in Testers agent (`.venv/bin/pytest`, `PYTHONPATH=src`, `pytest-asyncio`, no `rtk pytest`)

### Changed
- **Skillified state-awareness** — enhanced `.opencode/skills/state-awareness/SKILL.md` with full 4-step protocol (check, verify, refuse, workspace). All 6 agents now load the skill instead of inlining state checks, replacing ~90 lines of duplication
- **Removed per-project `.icm/`** — workspaces and adopted projects no longer create their own ICM database. All memory uses the central `.icm/` in the kodehold root with topic prefixes (`kodehold-<project>-*`)
- `scripts/workspace.sh` — `ws_adopt()` no longer creates `.icm/` or adds `.icm/` to `.gitignore`
- `scripts/gate.sh` — removed `ICM_DB` variable and per-project `.icm/` check; uses `icm stats` without `--db`; `active_to_review()` checks `.testers_done` marker
- `scripts/ship.sh` — removed `--db .icm/memories.db` flag
- `.opencode/agents/director.md` — all `--db` references removed from ICM protocol, workspace, session lifecycle; added `skill: allow`
- `.opencode/agents/testers.md` — creates `.testers_done` marker on completion
- `.opencode/agents/reviewers.md` — refuses to start without `.testers_done`
- `.opencode/agents/engineers.md` — updates Component Design after implementation
- `tests/init/01-config-valid.sh` — removed `.icm/` gitignore check
- `tests/init/02-icm-check.sh` — removed all `--db .icm/memories.db` references
- `.github/workflows/kodehold-ci.yml` — removed `--db .icm/memories.db`
- `docs/adr/ADR-0012-adopted-projects.md` — removed `.icm/` from directory tree and consequences
- `.opencode/agents/architects.md` — reviews + updates design doc after implementation cycles
- `.opencode/agents/fls.md` — updates design doc if fix changes documented behavior
- `.opencode/agents/scribes.md` — updates design doc Changelog and Version in pre-transition workflow
- `.opencode/agents/director.md` — verifies design doc is updated before any gate transition
- All 7 agents — "update design doc after work" steps added to workflows

### Fixed
- Gate enforcement gap — parallel Test/Review no longer possible (`.testers_done` blocks at gate level)
- `VERSION.md`, `CHANGES.md`, `TODO.md` — bumped to 0.6.0

## 0.5.0 — 2026-05-27

### Added
- ICM docs study — architecture, features, guide, integrations, product read. Key insights stored in kodehold-learnings
- Consolidation threshold awareness — step 4 in ICM Knowledge Flow for all 6 agents: pre-store check at >5 entries
- FLS cross-project recall — searches project-specific topics (`kodehold-<project>`, `kodehold-<project>-fls`) when project name is known
- FLS project discovery — lists workspaces, all topics, and does broad ICM/memoir search when project name is forgotten
- `ICM-Docs-Best-Practices` concept in `kodehold-scribes` memoir

### Changed
- `scribes.md` — ICM Best Practices section (consolidation, store nudge, auto-dedup, pattern extraction, memory lifecycle)
- `director.md` — ICM Protocol extended with consolidate/extract patterns guidance
- `fls.md` — project discovery step before triage, project history recall in minor fix workflow
- All 6 agent files — ICM Knowledge Flow renumbered, step 4 added for pre-store consolidation check
- `VERSION.md` — bumped to 0.5.0
- `TODO.md` — marked 5 items completed, added 3 new items

### Tested
- **Auto-dedup** — identical memory in same topic → same ID (updated). Different memory → new ID.
- **Recall quality** — "authorization" returned 3 relevant results via hybrid search where FTS5 would give 0.
- **Extract patterns** — detected pattern across 2 session-checkpoint memories, created concept in kodehold-learnings.

## 0.4.0 — 2026-05-27

### Added
- FLS (Front Line Support) team — 6th team with triage, hotfix, and escalation protocol
- ADR-0010 — FLS Front Line Support Team (Accepted)
- ADR-0011 — Team Meeting (collective project review, replaces solo Director approval)
- `.opencode/agents/fls.md` — FLS agent definition with triage criteria and workflow
- Team Meeting as step 0 in shipping gate (9 steps total)
- Director as default_agent in opencode.json with mode: primary
- Second opinion now requires cross-provider model (Claude/Codex), not same-family local
- 7 new ICM memoirs: kodehold-architects, kodehold-engineers, kodehold-reviewers, kodehold-testers, kodehold-scribes, kodehold-fls, kodehold-learnings
- ICM Knowledge Flow (6-step) added to all 6 agent files: search shared → search team → execute → store shared → store team → distill/refine

### Changed
- ADR-0002 — Director + 5 teams → Director + 6 teams (FLS added)
- ADR-0003 — Final Review updated to Team Meeting (ADR-0011)
- ADR-0006 — second opinion must use different provider (cross-provider mandated)
- ADR-0008 — FLS triage gateway between CLOSED and REOPEN; Team Meeting in Close Protocol
- ADR-0009 — Status promoted from Proposed to Accepted
- All 6 agent files — ICM Knowledge Flow section added before existing workflows
- Scribes — migrated from CLI (`icm store`/`icm recall`) to MCP tools (`icm_memory_store`/`icm_memory_recall`)
- Reviewers — second opinion stores outcomes in team learnings
- Design doc — organisational structure (6 teams), Review Cadence, REVIEW state, ADR index
- README.md — architecture diagram includes FLS, 6 teams, ADR range to 0011
- VERSION.md — bumped to 0.4.0
- TODO.md — updated with all completed items

### Fixed
- All 11 ADRs now pass Nygaard format smoke test
- `teams` memoir: FLS concept added, kodehold concept updated to 6 teams, scribes refined to MCP
- Frontmatter test: accepts mode: all for director (needs primary + subagent)

## 0.3.0 — 2026-05-26

### Added
- Director as working agent (`.opencode/agents/director.md`) with `task: allow` permission
- Lifecycle state tracking (`.kodehold-state`) — persistent state per project
- Automated quality gates (`scripts/gate.sh`) — 5 transitions with structural checks
- Workspace system (`scripts/workspace.sh`) — init, list, state, gate, deploy-ready
- Project catalog (`workspaces/.catalog`) — JSON registry of all managed projects
- Second opinion protocol — triggers on new ADRs, security-critical, complex decisions
- State awareness in all 5 subagents — check `.kodehold-state` before work, refuse in wrong phase
- Agent refusal/escalation protocol — agents report current vs required state + suggested gate
- Scribes documentation workflow — README, CHANGES, TODO, VERSION creation
- Centralized ICM — all project memory in kodehold `.icm/`, no per-project databases
- Gate support for pytest workspaces (venv detection, PYTHONPATH, `python3` compatibility)
- Architectural decision to remove per-workspace ICM in favour of central store

### Changed
- AGENTS.md simplified to quick reference — full definition in `.opencode/agents/director.md`
- `opencode.json` — director agent registered with `task: allow`
- All 5 subagent files — state awareness sections with allowed phases and escalation paths
- Reviewers — second opinion triggers documented, structured report format
- Architects — design doc status workflow, ADR triggers second opinion
- Scribes — pre-transition storage requirement, documentation file responsibilities
- Testers — state awareness for ACTIVE/REVIEW phases
- `.gitignore` — added `.kodehold-state` and `workspaces/`
- Test suite — adapted for director (frontmatter `task: allow`, transition checks in both AGENTS.md and director.md)

### Fixed
- Gate `scripts/gate.sh` — removed kodehold-infrastructure checks from INIT→ACTIVE (`.opencode/agents`)
- Gate `scripts/gate.sh` — added pytest support for workspace projects (venv, PYTHONPATH, python3)
- Gate `scripts/gate.sh` — made TODO.md optional, made .icm/ optional for workspaces
- Gate `scripts/gate.sh` — dual test runner support (tests/run.sh or pytest)

### Validated
- Full lifecycle run with `lib-validate` workspace: INIT → ACTIVE → REVIEW → CLOSED
- 3 gates passed (INIT_TO_ACTIVE, ACTIVE_TO_REVIEW, REVIEW_TO_CLOSED)
- 3 team delegation rounds (Architects, Reviewers, Engineers, Testers, Reviewers)
- 2 fix rounds (5 design issues + 4 implementation issues)
- Second opinion on ADR-0001 (minor concerns, all addressed)
- 254 tests passing in workspace
- Scribes documented 5 memories in central ICM

## 0.2.0 — 2026-05-25

### Added
- Director orchestrator (`AGENTS.md`) with lifecycle states, quality gates, token budgets, ICM protocol, second opinion triggers
- `opencode.json`: Ollama provider, project permissions (no hardcoded default model)
- 5 team subagents (`.opencode/agents/`): Architects, Engineers, Reviewers, Testers, Scribes
  - Each with YAML frontmatter, responsibilities, workflow, constraints (no model overrides)
- Shared protocol reference (`.opencode/references/kodehold-protocol.md`)
- Test suite: 10 tests across smoke, init, and integration
- CI workflow (`.github/workflows/kodehold-ci.yml`) with 3 parallel jobs
- Shipping gate protocol (8-step process in AGENTS.md + `scripts/ship.sh`)
- ICM knowledge graph: 5 memoirs, 33 concepts, 21 links, 65 memories

### Changed
- ADR-0005 rewritten: bring-your-own-model, light mode optional (KODEHOLD_LIGHT=1), no per-team model config, Ollama as option not default
- Design doc LLM Support section updated to match ADR-0005
- All agent files: model overrides removed — teams inherit user's default OpenCode model
- Protocol reference: light mode marked as optional

### Fixed
- CI: actions/checkout upgraded to v6, setup-python to v6, Node24 opt-in
- CI: ICM download URL corrected to use specific release tag and asset name
- CI: ICM database bootstrap step added before init tests
- Test: ICM init test now auto-creates DB if missing, graceful warn instead of hard-fail
- Test: removed assertion for hardcoded model field in opencode.json
- ADR index: design doc now lists all 8 ADRs with proper references

## 0.1.0 — 2026-05-25

### Added
- Design document (`docs/design/README.md`) — full architecture, team structure, lifecycle
- 8 Architecture Decision Records covering all design constraints:
  - ADR-0001: Foundation and principles
  - ADR-0002: Director + 5 teams (Architects, Engineers, Reviewers, Testers, Scribes)
  - ADR-0003: Design document lifecycle with review gates
  - ADR-0004: ICM and RTK integration strategy
  - ADR-0005: LLM support with Ollama and light mode (32k context)
  - ADR-0006: Second opinion protocol for cross-model validation
  - ADR-0008: Project lifecycle with reopen and archive protocol
- ADR index (`docs/adr/README.md`)
- `.gitignore` excluding `.icm/` directory
- Top-level README, VERSION, TODO, CHANGES
- GitHub repository initialized at `github.com/Acharnite/kodehold`
- ICM persistent memory store initialized
