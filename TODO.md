# TODO

## Completed

- [x] **ADR-0054: Replace opencode-rag with Graphify Knowledge Graph** — ADR written, reviewed, cross-validated, and Accepted. 7-phase migration plan defined:
  - [x] Phase 1: ADR written and Accepted
  - [x] Phase 2: Clean up opencode.json (config cleanup — remove krypto-agent MCP, plugin refs)
  - [x] Phase 3: Install Graphify (uv tool install + graphify install)
  - [x] Phase 4: Update AGENTS.md instructions
  - [x] Phase 5: Update opencode-rag-knowledge-flow skill (replaced by graphify-knowledge-flow)
  - [x] Phase 6: Update agent files (director.md + team agents)
  - [x] Phase 7: Final verification (completed)

- [x] **Issue #34: YAML-Based Agent & Task Configuration (ADR-0037)** — Phase 1-4 complete:
  - [x] `config/agents.yaml` — all 8 agents defined with name, role, model, and delegations
  - [x] `config/agents.schema.json` — JSON Schema for validating agents.yaml
  - [x] `config/tasks.yaml` — 4 workflows (adr-flow, implement-flow, bugfix-flow, ship-gate) + 5 gates
  - [x] `scripts/validate-config.sh` — validates config/agent.yaml against JSON Schema
  - [x] `scripts/sync-agent-config.sh` — syncs frontmatter between .md agent files and agents.yaml
  - [x] `tests/init/test_yaml_config.py` — 46 schema validation tests
  - [x] ADR-0037 promoted from Proposed → Accepted
> **Historical (ADR-0050, ADR-0054):** The ICM→agentmemory migration (ADR-0029) was superseded by ADR-0050 (Agentmemory → OpenCode RAG Migration), which was itself superseded by ADR-0054 (OpenCode RAG → Graphify Knowledge Graph). Pre-task knowledge retrieval is now handled by Graphify knowledge graph queries and opencode-mem persistent memory.

- [x] **ICM → Agentmemory Migration Complete** — Phases 1-5 (ADR-0029): ADR-0030 through ADR-0033, Action Frontier Protocol, routine templates, auto-crystallize, inter-agent signaling. Complete replacement of ICM with agentmemory across all agents, scripts, and references. *(Superseded by ADR-0050)*
- [x] ADR-0030: Agentmemory Knowledge Flow Skill — unified knowledge flow via agentmemory replacing ICM Knowledge Flow *(Superseded by ADR-0050)*
- [x] ADR-0031: Action Frontier + Crystals — action dependency frontier with automatic crystallization of completed chains *(Superseded by ADR-0050)*
- [x] ADR-0032: Routine Templates — frozen workflow routines instantiable via agentmemory_routine_run *(Superseded by ADR-0050)*
- [x] ADR-0033: Inter-Agent Signals — typed messaging between agents via agentmemory signals *(Superseded by ADR-0050)*
- [x] **director.md Action Frontier Protocol** — agentmemory-driven action frontier with dependency resolution, sentinel gating, and auto-crystallization at delegation boundaries *(Superseded by ADR-0050)*
- [x] **scribes.md action management + signals** — Scribes manages actions via agentmemory (frontier, actions, leases, sentinels, crystals) and signals (typed messaging, threading, team feed) *(Superseded by ADR-0050)*
- [x] **gate.sh/ship.sh agentmemory update** — gate.sh and ship.sh updated to check agentmemory instead of ICM for pre-flight and readiness *(Superseded by ADR-0050)*
- [x] **agentmemory-knowledge-flow skill** — dedicated `.opencode/skills/agentmemory-knowledge-flow/SKILL.md` replacing old ICM Knowledge Flow *(Kept as archival, replaced by opencode-rag-knowledge-flow per ADR-0050)*
- [x] **Auto-crystallize on delegation** — completed action chains auto-crystallized into compact digests with narrative, outcomes, and lessons *(Superseded by ADR-0050)*
- [x] **Inter-agent signal wiring** — all 6 teams wired with signal send/read for typed coordination via Director *(Superseded by ADR-0050)*

- [x] ADR-0011: Team Meeting — Collective Project Review
- [x] **Team meeting feature** — replaces solo Director final approval
- [x] Design document with architecture, team structure, lifecycle
- [x] 10 ADRs covering all design constraints (0001–0010), all Accepted
- [x] Director orchestrator (AGENTS.md + `.opencode/agents/director.md`) with lifecycle states, quality gates, ICM protocol, second opinion, shipping gate
- [x] 6 team subagents (Architects, Engineers, Reviewers, Testers, Scribes, FLS) with YAML frontmatter
- [x] opencode.json with Ollama provider (no hardcoded default model — user's global OpenCode model used)
- [x] AGENTS.md, README.md, VERSION.md, TODO.md, CHANGES.md, .gitignore
- [x] ICM memories, memoirs, concepts, links — 144 memories, 7 memoirs
- [x] Test suite: 12 tests (4 smoke + 3 init + 5 integration) with GitHub Actions CI (3 jobs)
- [x] Shipping gate protocol with scripts/ship.sh (8-step process)
- [x] Director as working agent with `task: allow`
- [x] Lifecycle state tracking (.kodehold-state) with gate automation (scripts/gate.sh)
- [x] Workspace system (scripts/workspace.sh) with project registry (.catalog)
- [x] State awareness in all 6 subagents — check state before work, refuse in wrong phase
- [x] Second opinion protocol — implemented and tested on ADR-0001
- [x] Scribes documentation workflow — README, CHANGES, TODO, VERSION per project
- [x] Centralized ICM — no per-project databases
- [x] Proof-of-concept: lib-validate through full INIT→ACTIVE→REVIEW→CLOSED lifecycle
- [x] Agent refusal/escalation protocol with state guidance
- [x] Setup ICM MCP server — respects local databases
- [x] `docs/decisions/` directory for working notes
- [x] `.opencode/skills/` directory for future skills
- [x] FLS (Front Line Support) team — agent definition, triage criteria, escalation protocol
- [x] ADR-0010: FLS Front Line Support Team
- [x] ADR-0009 promoted from Proposed to Accepted
- [x] ADR-0002 updated: Director + 6 teams (FLS added)
- [x] ADR-0008 updated: FLS triage gateway between CLOSED and REOPEN
- [x] All ADRs follow Nygaard format (previously validated by smoke tests — removed in v1.19.0, tests are now functionality-only)
- [x] **Default Director agent** — hardcoded as default_agent in opencode.json with mode: primary
- [x] ADR-0006 updated: cross-provider second opinion (Claude/Codex), fallback til brugerprompt
- [x] Second opinion cross-provider krav — implementeret i Director og Reviewers agenter
- [x] Scribes agent opdateret til MCP tools (icm_memory_store, icm_memory_recall)
- [x] ICM naming convention — 7 nye memoirs, 6 team learnings topics, konsistent navnestruktur
- [x] ICM Knowledge Flow (6-step) — implementeret i alle 6 agenter
- [x] Feedback tool references — tilføjet til Director og Reviewers (icm_feedback_record/icm_feedback_search)
- [x] Initial memoirs for KodeHold architecture — 12 team/domain memoirs oprettet

## High Priority

- [x] **Exercise FLS flow** — FLS triaged + fixed 2 crashes (regex(-1), validate(-1)) on lib-validate. 254 tests passed.
- [x] **Exercise reopen flow** — FLS escalation (async validators) → CLOSED→REOPEN→ACTIVE→REVIEW→CLOSED. ADR-0002, 48 async tests, 302 total.
- [x] **Test auto-dedup behavior** — verified: >85% hybrid similarity i samme topic → opdaterer eksisterende (samme ID returneres). Forskellig memory → nyt ID.
- [x] **Measure recall quality** — hybrid search (70% vector + 30% BM25) fanger semantisk mening. "authorization" matcher login/auth/JWT selvom ordet ikke findes i teksten. FTS5-only ville give 0 resultater.
- [x] **Study ICM docs** — architecture, features, guide, integrations, product read. Key insights recorded in kodehold-learnings

- [x] **Use icm_memory_extract_patterns** — testet mod kodehold-session-checkpoint (4 entries). Detected 1 pattern, created concept i kodehold-learnings. Auto-name = første keyword. Bedst når topics har 5+ entries.
- [x] **Consolidation threshold awareness** — added as step 4 (pre-store check) i ICM Knowledge Flow i alle 6 agenter. Tjekker >5 entries, konsoliderer før lagring.
- [x] **OpenCode ICM plugin** — run `icm init --mode hook` to install auto-extraction hooks for session.created, tool.execute.after

## Medium Priority

- [x] **Remove all .sh scripts** — 9 bash scripts deleted (gate.sh, workspace.sh, ship.sh, benchmark.sh, sync-agent-config.sh, validate-config.sh, token-usage.sh, detect-test-framework.sh, output.sh). All references updated to .py equivalents.
- [x] **Forbyd git clean -fd uden brugertilladelse** — git clean -fd må ALDRIG bruges uden bruger tilladelse. Kommandoen sletter alle untracked filer og kan forårsage tab af data. Dette bør tilføjes som en regel i Director protokollen og i agent dokumentationen. ([#3](https://github.com/Acharnite/kodehold/issues/3)) ✅ Implemented 2026-05-29
- [x] **Brug GitHub til issue tracking og generel integration** — KodeHold bør bruge GitHub Issues til at holde styr på projekter og generelt integrere GitHub features (PRs, releases, branch protection, etc.) i workflowet. Dette forbedrer projektstyring og samarbejde. ([#4](https://github.com/Acharnite/kodehold/issues/4))
- [x] **Design doc discipline** — agents must read design doc before work and update it with results when done (enforce via agent files)
- [x] **Implement reopen protocol** — end-to-end with ICM context restoration (Architects impact analysis → design update → new ADRs). Exercised on lib-validate async validators.
- [x] **Investigate skill** — adapted from gstack: 4-phase systematic debugging (investigate→analyze→hypothesize→implement) with Iron Law, pattern analysis, 3-strike rule, regression test requirement, structured debug report, and ICM storage ([#2](https://github.com/Acharnite/kodehold/issues/2))
- [x] **Implement light mode** (KODEHOLD_LIGHT=1) with collapsed Quality team (Reviewers + Testers). Related strategies for Ollama 32K ctx limit:
  - [x] **Trim director.md** — 301 → 153 lines (49% reduction). Core identity preserved. Removed: duplicate Second Opinion section, FLS Escalation Pattern, Test→Review duplicate, Gate Blockers, detailed adopted projects. Condensed: Gate Enforcement, Workspace, Session Lifecycle. Saves ~8K tokens per Director call
  - [x] **Session checkpoint + reload** — Director gemmer checkpoint i ICM efter ~8 kald. Scribes har store/resume workflow. Director kan foreslå fresh session for små context modeller (Ollama 32K)
  - [x] **ICM context summaries** — Scribes komprimerer chat-historik til ICM summaries jævnligt i stedet for at beholde alt i context ([#5](https://github.com/Acharnite/kodehold/issues/5)) ✅ Implemented 2026-05-29
  - [x] **Auto-compaction threshold** — tilføjet `compaction: { auto: true, prune: true, reserved: 7000 }` til opencode.json. Efterlader 7K buffer så compaction starter tidligere end default
- [x] **All subagent prompts must be in English** — added rule #6 to Director's Core Protocol ("ALWAYS write subagent prompts in English only") and a bold warning in the Delegation Pattern section. Also translate remaining Danish in TODO.md (lines 48, 61, 62, 75).
- [x] **Fjern per-project .icm/** — KodeHold opretter ikke længere `.icm/` i workspace/adopted projekter. Alt bruger central `.icm/` i kodehold roden med topic prefixes (`kodehold-<project>-*`)
- [x] **Test adopted project** — end-to-end: adopt → design → feature → gates → CLOSED. ✅ All 3 gates passed, full lifecycle INIT→ACTIVE→REVIEW→CLOSED, central ICM brugt, ingen `.icm/` i workspace. 1 cosmetic issue: `find` i symlink rapporterer 0 files (non-blocking).
- [x] **Token budget tracking** — per team/phase with alerts when exceeded ([#6](https://github.com/Acharnite/kodehold/issues/6)) ✅ Implemented v0.4.0 — scripts/token-usage.sh
- [x] **Architects: research** — added `webfetch: allow` + `websearch: allow` permissions and "Research before designing" step in workflow
- [x] **Team Meeting (ADR-0011)** — exercised on radarr-lang-router. 6 teams presented, 4 approve + 2 approve-with-concerns. Final: APPROVED-WITH-CONCERNS. Protocol validated — single Task call, 8K budget
- [x] **Initial design review gate** — added `.design_reviewed` marker (Reviewers approves design) to INIT→ACTIVE gate. Also added `.impact_analysis_done` marker (Architects impact assessment) to CLOSED→REOPEN gate
- [x] **Memoir distillation at CLOSED** — ADR-0009 phase 4: Scribes should distill project memories into memoirs after each CLOSED transition. Currently manual ([#7](https://github.com/Acharnite/kodehold/issues/7)) ✅ Implemented v0.4.0 — docs/icm-knowledge-flow.md
- [x] **Fix team rækkefølge: Testers før Reviewers** — Rækkefølgen Testers→Reviewers er korrekt i ACTIVE-fasen (director.md, README.md), men forkert i ADR-0011 team meeting rækkefølgen ("Architects→Engineers→Reviewers→Testers→Scribes→FLS") og generelle opremsninger i agent-filer. Testers skal ALTID være før Reviewers: Testers verificerer → Reviewers godkender. Rettes i: ADR-0011, agent-.md filer, og enhver anden forekomst.
- [x] **Sikre at design/ADR-status opdateres FØR implementering** — ADR-0015 og design-dokumentet blev ikke sat til "Accepted" inden engineers begyndte implementering. Dette er et brud på processen. Overvej at tilføje et gate-check i INIT→ACTIVE transitionen der verificerer at tilhørende design-dokument og ADR har status "Accepted" før tilladelse gives. Dette bør også gælde for ADR-0015 retroaktivt — opdater status til "Accepted" i begge dokumenter.

- [x] **Run KodeHold against another real project** — validated full lifecycle on radarr-lang-router: adopt → design → tests → gates → CLOSED. 50 tests, ADR-0001, 3 gates passed. FLS hotfixed a KeyError bug.
- [x] **ADR format CI** — ADR Nygaard validering kørte tidligere i CI via smoke tests (removed in v1.19.0 — tests are now functionality-only)
- [x] **Udvid Scribes rolle til alt dokumentationsarbejde** — Scribes skal håndtere AL dokumentation: design docs, ADR'er, CHANGES.md, README, status-opdateringer, ICM-lagring, changelog. De øvrige teams skal KUN køre state-awareness protokol og udføre deres kerneopgave (Architects designer, Engineers implementerer, Reviewers reviewer, Testers tester, FLS triager). Ingen team-medlemmer skal skrive dokumentation selv. Dette kræver opdatering af alle 6 agent-.md filer, director.md delegation-sektionen, og Scribes' workflow-beskrivelse.
- [x] **Fix gate-script workspace directory check** — gate.sh checks for markers (.design_reviewed, .second_opinion_done) in main project directory instead of workspace directory. Causes false negatives for workspace/adopted projects. Manual state updates needed as workaround (see qbit-migrate adoption). Fix: gate.sh should accept project path parameter or check workspaces/<name>/ when validating workspace projects. ([#8](https://github.com/Acharnite/kodehold/issues/8)) ✅ Implemented 2026-05-29
- [x] **Investigate symlink/rtk command issues** — Commands on symlinked workspace directories have challenges (e.g., test collection fails with FileNotFoundError when importing modules). User suspects rtk-related root cause. Investigate and fix. Example: qbit-migrate tests fail because LOG_FILE path calculation resolves incorrectly when imported via symlink. Related to ADR-0012 (adopted projects). ([#9](https://github.com/Acharnite/kodehold/issues/9)) ✅ Implemented 2026-05-29 — symlink resolution in gate.sh and workspace.sh
- [x] **Agenter skal være opmærksomme på at alle adopterede projekter er symlinks** — Når KodeHold adopterer et projekt, oprettes der et symlink fra workspaces/<name>/ til det oprindelige projekt. Dette betyder at filstiberegninger, imports og kommandoer kan opføre sig anderledes end forventet. Agent dokumentation skal informere om dette. Relateret til ADR-0012 (adopterede projekter). ([#10](https://github.com/Acharnite/kodehold/issues/10)) ✅ Implemented v0.4.0 — engineers.md, scribes.md updated
- [x] **Bruger-accept checkpoint før final review** — Inden REVIEW→CLOSED overgangen skal der stoppes op og gives bruger accept for at forhindre at et projekt går i CLOSED state bare for at blive genåbnet igen for at tilføje nye features. Dette kræver en ny gate-check eller et interaktivt prompt i REVIEW→CLOSED transitionen. ([#11](https://github.com/Acharnite/kodehold/issues/11)) ✅ Implemented v0.4.0 — gate.sh REVIEW→CLOSED requires user confirmation
- [x] **Second opinion bruger samme model — bias problem** — Second opinion (cross-model validation) bruger samme model som den oprindelige design/proces. Dette skaber bias og reducerer effektiviteten af second opinion. Skal undersøges og fixes. Mulige løsninger: Brug en anden model til second opinion, eller implementer et andet valideringsmekanisme. ([#12](https://github.com/Acharnite/kodehold/issues/12)) ✅ Implemented v0.14.0 — Dedicated second-opinion subagent with Google Gemma 3 12B via OpenRouter, OpenRouter provider configured, Director routes to dedicated subagent, Reviewers.md cleaned up.
- [x] **Hvad er ICM Knowledge Flow?** — Nogle agenter kalder "ICM Knowledge Flow" men det er ikke klart hvad dette er eller hvor det er defineret. Skal undersøges og dokumenteres. Muligvis en protokol der skal tilføjes til agent dokumentationen. ([#13](https://github.com/Acharnite/kodehold/issues/13)) ✅ Implemented v0.4.0 — docs/icm-knowledge-flow.md (179 lines)
- [x] **Ryd state filer op ved REVIEW→CLOSED** — Når projekter går fra REVIEW til CLOSED, skal alle state filer ryddes op (.design_reviewed, .testers_done, .code_reviewed, .second_opinion_done, .impact_analysis_done, .team_meeting_done). Dette bør automatiseres i gate-scriptet eller workspace.sh. ([#14](https://github.com/Acharnite/kodehold/issues/14)) ✅ Implemented v0.4.0 — gate.sh cleanup on REVIEW→CLOSED
- [x] **Commit untracked filer før session slut** — ADR-0015 til ADR-0019 blev oprettet men aldrig committed, og forsvandt ved session reload. Tilføj regel i Director/Scribes protokol: nye ADR-, design- eller konfigurationsfiler skal commits før session afsluttes. ([#23](https://github.com/Acharnite/kodehold/issues/23)) ✅ Implemented 2026-05-29
- [x] **Upgrade GitHub MCP server** — replaced deprecated `@modelcontextprotocol/server-github` npm package with official `github/github-mcp-server` Go binary v1.1.2. Resolves intermittent auth failures. ([#25](https://github.com/Acharnite/kodehold/issues/25)) ✅ Implemented 2026-05-29
- [x] **Fix stale test count** — TODO.md and README.md corrected "10 tests" → "12 tests". ([#22](https://github.com/Acharnite/kodehold/issues/22)) ✅ Implemented 2026-05-29
- [x] **consolidate-all script** — Opret script der automatisk consolidere alle ICM topics med >5 entries. Kører `icm_memory_health`, konsolidere store topics via `icm_memory_consolidate`, rapporterer resultat. ([#24](https://github.com/Acharnite/kodehold/issues/24))

## Low Priority

- [x] **FLS→Scribes protocol fix** — ADR-0010 says FLS requests Scribes via Director for ICM storage. FLS currently stores directly. Resolve inconsistency ([#15](https://github.com/Acharnite/kodehold/issues/15))
- [x] Expand test suite beyond 12 tests — edge cases, failure modes, workspace stress tests ([#16](https://github.com/Acharnite/kodehold/issues/16))
- [x] **Performance benchmarks** — token usage with/without RTK, with/without ICM summaries ([#17](https://github.com/Acharnite/kodehold/issues/17)) ✅ Implemented v0.4.0 — scripts/benchmark.sh
- [x] FLS-specific tests — `tests/integration/04-fls-workflow.sh` med 7 test areas (triage criteria, workflow, state restrictions, skill references, ICM docs, permissions), 50+ assertions, 11/11 total suite
- [x] **Auto-generate CHANGES.md entries from git log on ship** ([#18](https://github.com/Acharnite/kodehold/issues/18)) ✅ Implemented v0.4.0 — ship.sh integration
- [x] **Workshop: dokumentér KodeHold-metoden så nye teams kan onboardes** ([#19](https://github.com/Acharnite/kodehold/issues/19)) ✅ Implemented v0.4.0 — docs/workshop.md (189 lines)

## Memory Stack Features (from AI Agent patterns)

- [x] ADR-0020: Hierarchical Memory (Hot/Warm/Cold) — **Superseded** by ICM decay system ([#26](https://github.com/Acharnite/kodehold/issues/26))
- [x] ADR-0021: Prospective Memory (Task Queue & Scheduler) — Accepted & implemented ([#27](https://github.com/Acharnite/kodehold/issues/27)) ✅ Implemented 2026-05-29
- [x] ADR-0022: Automated Episodic Extraction — **Superseded** by ICM plugin hooks ([#28](https://github.com/Acharnite/kodehold/issues/28))
- [x] ADR-0023: Semantic Memory Automation — **Superseded** by ICM plugin + memoir system ([#29](https://github.com/Acharnite/kodehold/issues/29))
- [x] ADR-0024: Shared Memory (Multi-Agent Alignment) — **Deprecated** (sequential delegation eliminates conflicts) ([#30](https://github.com/Acharnite/kodehold/issues/30))
- [x] ADR-0025: A2A Protocol (Agent-to-Agent Coordination) — **Deprecated** (Director already serves as coordinator) ([#31](https://github.com/Acharnite/kodehold/issues/31))

## Known Infrastructure Patches

> **Obsolete (ADR-0050):** All `@agentmemory/agentmemory` patches below are no longer applicable — the agentmemory npm package has been removed per ADR-0050. Patches are retained for historical reference only.

| Package | Patch | Location | Status |
|---------|-------|----------|--------|
| `@agentmemory/agentmemory` | Viewer bind `127.0.0.1` → `0.0.0.0` (port 3113) | `scripts/patch-agentmemory.sh` | 🗑️ Obsolete per ADR-0050 |
| `@agentmemory/agentmemory` | Summary XML parsing — strip markdown fences, add retry loop | Direct patch to `dist/index.mjs` | 🗑️ Obsolete per ADR-0050 |
| `@agentmemory/agentmemory` | `triggerVoid()` → `trigger()` + `TriggerAction.Void()` | `patches/agentmemory-merged.patch` | 🗑️ Obsolete per ADR-0050 |
| `@agentmemory/agentmemory` | Agentmemory-capture project detection fix | `patches/agentmemory-capture-project-detection.patch` | 🗑️ Obsolete per ADR-0050 |

## Prospective Tasks

_This section is maintained by Scribes. Active deferred/recurring tasks are tracked via add_memory with tag "prospective" (see ADR-0057)._

- 0 deferred tasks tracked via opencode-mem

## ADR-0050: Agentmemory → OpenCode RAG Migration

> **Status:** Implementation complete (2026-06-28). The agentmemory daemon and npm package have been fully removed. Pre-task knowledge retrieval now uses OpenCode's built-in RAG tools (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) and opencode-mem.

- [x] ADR-0050 written and Accepted
- [x] Design doc updated (Section 7.2, Section 10, ADR index, Changelog)
- [x] `.opencode/skills/agentmemory-knowledge-flow/SKILL.md` marked as deprecated (archival only)
- [x] `opencode-rag-knowledge-flow` skill updated with full tool references
- [x] Scripts and CI cleaned of agentmemory references
- [x] All agent definitions and configs updated
- [x] Memory system migrated to opencode-mem per ADR-0057
- [x] Remaining agentmemory references in non-ADR, non-design files cleaned up (this batch)
