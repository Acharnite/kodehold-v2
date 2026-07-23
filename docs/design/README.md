# KodeHold — Coding Orchestrator Design Document

**Version:** 1.25.0  
**Status:** Active  
**Last Updated:** 2026-07-22

---

## 1. Overview

KodeHold is a coding orchestrator that applies conscious team-based software engineering methodology to AI-assisted development. It simulates a structured software organization where specialized AI agents collaborate under a Director to produce high-quality code.

The orchestrator is design-document-centric: every project begins with a design document, all teams work from it, and it is continuously reviewed and updated throughout the project lifecycle. Projects can be closed and reopened for new features or bug fixes, with full context preserved via persistent memory.

---

## 2. Principles

| # | Principle | Description |
|---|-----------|-------------|
| 1 | **Design-First** | Every project starts with and revolves around a living design document |
| 2 | **Separation of Concerns** | Distinct teams handle design, implementation, review, testing, and memory |
| 3 | **Token-Conscious** | Every operation is evaluated for token cost; efficient output through native tools |
| 4 | **Persistent Memory** | opencode-mem stores all project context, decisions, and rationale across sessions |
| 5 | **LLM-Agnostic** | Core works with any LLM; Ollama is primary; second-opinion cross-check supported |
| 6 | **Traceable Decisions** | All architectural decisions are recorded as ADRs in git |
| 7 | **Project Lifecycle** | Projects can be opened, closed, and reopened without losing context |
| 8 | **Safe Operations** | `git clean -fd` must never be executed automatically — it deletes all untracked files and requires explicit user approval |

---

## 3. Organizational Structure

```
┌─────────────────────────────────────────────────┐
│                   DIRECTOR                       │
│  Orchestrates lifecycle, assigns work, gates     │
│  quality, triggers second opinions               │
└─────────────────────────────────────────────────┘
          │            │           │           │           │
          ▼            ▼           ▼           ▼           ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│ARCHITECTS│  │ENGINEERS │  │REVIEWERS │  │ TESTERS  │  │ SCRIBES  │  │   FLS    │
├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤
│Design    │  │Implement │  │Code      │  │Write     │  │Agent     │  │Triage    │
│Documents │  │Features  │  │Review    │  │Tests     │  │Memory    │  │Hotfix    │
│ADRs      │  │Refactor  │  │Design    │  │Verify    │  │Doc       │  │Escalate  │
│Tech      │  │Bugfixes  │  │Review    │  │Regression│  │Changelog │  │Support   │
│Decisions │  │          │  │Standards │  │Perf Test │  │Extract   │  │          │
└──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘
```

### 3.0 Loop Engineering Operational Framework

KodeHold adopts **Loop Engineering** as its operational framework for workspace loop management (ADR-0060). All three phases are now complete:

| Phase | Goal | Status | Key Deliverables |
|-------|------|--------|-----------------|
| **Phase 1: Foundation** | loop-init integration + loop subcommand | ✅ Complete | `workspace.py init/adopt` calls `loop-init`, `workspace.py loop list` |
| **Phase 2: Automation** | enable/disable/run + cron commands | ✅ Complete | `workspace.py loop enable/disable/run`, `workspace.py cron install/remove/list` |
| **Phase 3: Monitoring** | audit + cost + sync commands | ✅ Complete | `workspace.py audit/cost/sync` |

Loop Engineering provides 7 production patterns, CLI tools (loop-init, loop-audit, loop-cost, loop-sync), and safety mechanisms (loop-gate, loop-context, loop-constraints). All automation starts at L1 (report-only) and graduates to higher autonomy only after proven reliability.

See ADR-0060 for the full integration specification.

**Available Commands:**

```bash
# Loop management
workspace.py loop <name> list              # List active loops
workspace.py loop <name> enable <pattern>  # Enable a pattern
workspace.py loop <name> disable <pattern> # Disable a pattern
workspace.py loop <name> run <pattern>     # Run a loop manually

# Cron management
workspace.py cron install                  # Install crontab entries
workspace.py cron remove                   # Remove crontab entries
workspace.py cron list                     # Show crontab entries

# Monitoring
workspace.py audit <name>                  # Run loop-audit
workspace.py cost <name> <pattern>         # Estimate token cost
workspace.py sync <name>                   # Check STATE.md ↔ LOOP.md drift
```

**Supported Patterns:**

| Pattern | Cadence | Risk | Token Cost | Level |
|---------|---------|------|------------|-------|
| Daily Triage | 1d–2h | Low | Low (~50k/run) | L1 |
| PR Babysitter | 5–15m | Medium | High (~250k/run) | L1→L2 |
| CI Sweeper | 5–15m | Medium | Very high (~200k/run) | L2 |
| Dependency Sweeper | 6h–1d | Medium | Medium (~300k/run) | L2 |
| Changelog Drafter | 1d | Low | Low (~35k/run) | L1 |
| Post-Merge Cleanup | 1d–6h | Low | Low (~40k/run) | L1 |
| Issue Triage | 2h–1d | Low | Low (~30k/run) | L1 |

### 3.1 Director

The Director is the top-level orchestrator. Responsibilities:
- Project lifecycle management (init, develop, review, close, reopen)
- Task assignment to teams based on current phase
- Quality gate enforcement
- Triggering second opinions on critical decisions
- Token budget management

### 3.2 Architects

Design authority for the project. Responsibilities:
- Author and maintain the project Design Document
- Write Architecture Decision Records (ADRs)
- Evaluate technology choices and trade-offs
- Ensure design coherence across the project

### 3.3 Engineers

Implementation team. Responsibilities:
- Generate code from design specifications
- Refactor existing code
- Fix bugs
- Implement features per design doc

### 3.4 Reviewers

Quality assurance through review. Responsibilities:
- Code review against design doc and standards
- Design review and feedback
- Coordinate second opinion requests with Director
- Verify ADR compliance

### 3.5 Testers

Verification team. Responsibilities:
- Write and execute unit, integration, and e2e tests
- Run regression suites
- Performance testing
- Edge case analysis

### 3.6 Scribes

Memory and documentation team. Responsibilities:
- Manage persistent memory and documentation (store/retrieve project context via opencode-mem)
- Generate and update project documentation
- Maintain CHANGELOG
- Extract knowledge from completed work for future reuse

### 3.7 FLS (Front Line Support)

First line of defense for minor bugs and small changes. Responsibilities:
- Triage incoming issues — determine minor (fix directly) vs major (escalate)
- Apply hotfixes to CLOSED and ACTIVE projects
- Escalate comprehensive issues to REOPEN with impact summary
- Maintain deep knowledge of completed projects for rapid response

See ADR-0010 for full FLS specification.

---

## 4. Design Document Lifecycle

```
[Create] → [Review] → [Approve] → [Implement] → [Update] → [Review] → ...
                                                                    │
                                                          ┌─────────┘
                                                          ▼
                                                    [Project Closed]
                                                          │
                                                    [Reopen for
                                                     new feature/bugfix]
                                                          │
                                                          ▼
                                                    [Update Design Doc]
                                                          │
                                                          ▼
                                                    [Implementation]
```

### 4.1 Design Document Structure

Each design document follows this structure:

```markdown
# Project: [Name]
**Version:** x.y
**Status:** Draft | Active | Updating | Superseded
**Design Authority:** Architects
**Last Reviewed:** YYYY-MM-DD

### 4.1 Purpose & Scope
### 4.2 Requirements
### 4.3 Architecture Overview
### 4.4 Component Design
### 4.5 Data Model
### 4.6 API Design
### 4.7 Implementation Plan
### 4.8 Testing Strategy
### 4.9 ADR Index (links to relevant ADRs)
### 4.10 Open Questions
### 4.11 Changelog
```

### 4.2 Review Cadence

| Phase | Review Type | By | Marker |
|-------|------------|-----|--------|
| Before INIT→ACTIVE | Design Review | Reviewers approve design | `.design_reviewed` |
| Before INIT→ACTIVE | Mandatory Second Opinion | Cross-model validation | `.second_opinion_done` |
| ACTIVE — after Architects | Design Review Gate (Gate 1) | Reviewers | `.design_review_v2` |
| ACTIVE — after Engineers | Code Review Gate (Gate 2) | Reviewers | `.code_reviewed` |
| ACTIVE — after Testers | Comprehensive Review (Gate 3) | Reviewers | `.testers_done` |
| Before REVIEW→CLOSED | Team Meeting (ADR-0011) | All 6 teams — collective review | — |
| Before CLOSED→REOPEN | Impact Assessment | Architects assess scope | `.impact_analysis_done` |

Reviewers serve as **gatekeepers** for lifecycle transitions (ADR-0017). Before the Director runs `gate.py --transition`, Reviewers validate that all transition requirements are met via `gate.py --validate-only`. This ensures independent quality validation — the Director orchestrates, Reviewers validate.

See ADR-0016 for the full specification of early review gates in ACTIVE phase.
See ADR-0017 for the full specification of Reviewers as gatekeeper and mandatory second opinion.

For automation/CI, INIT→ACTIVE confirmation can be bypassed with `--yes` or
`OPENCODE_NONINTERACTIVE=true`. Marker cleanup must still occur only after the
gate fully passes.

> **Note:** `--yes` must be the **first flag** for proper argument passthrough.
> Example: `bash scripts/gate.py --yes --validate-only ACTIVE_TO_REVIEW` works,
> but `bash scripts/gate.py --validate-only --yes ACTIVE_TO_REVIEW` does not.

> **Reviewer Mode:** When `--reviewer-mode` is used, gate.py outputs structured results including `GATE_RESULT`, `CHECKS`, `MARKERS_REQUIRED`, `MARKERS_CLEANUP`. The `CHECKS` field lists individual check results (e.g., `design_reviewed:PASS,second_opinion_done:FAIL`). The `MARKERS_REQUIRED` field lists markers that must exist for the transition.

---

## 5. Architecture Decision Records (ADRs)

All significant decisions are recorded as ADRs in `docs/adr/` following the Nygard format:

```
ADR-NNNN: Title
Status: Proposed | Accepted | Deprecated | Superseded
Context: Why this decision is needed
Decision: What was decided
Consequences: Trade-offs and follow-ups
```

### ADR Index

Only **active** ADRs (Accepted + Proposed) are listed here. Inactive ADRs (Superseded, Deprecated, Archived) have been moved to `docs/adr/inactive/`.

| ADR | Title | Status |
|-----|-------|--------|
| ADR-0001 | KodeHold Foundation and Principles | Accepted |
| ADR-0002 | Organizational Structure — Director + Teams | Accepted |
| ADR-0003 | Design Document Lifecycle | Accepted |
| ADR-0005 | LLM Support and Light Mode | Accepted |
| ADR-0006 | Second Opinion Protocol | Accepted |
| ADR-0008 | Project Lifecycle and Reopening | Accepted |
| ADR-0010 | FLS — Front Line Support Team | Accepted |
| ADR-0011 | Team Meeting — Collective Project Review | Accepted |
| ADR-0012 | Adopted Projects — Existing Codebases in KodeHold | Accepted |
| ADR-0013 | Investigate Skill — Systematic Debugging | Accepted |
| ADR-0015 | Director Delegation Enforcement via Tool Permissions | Accepted |
| ADR-0016 | Early Review Gates in ACTIVE Phase | Accepted |
| ADR-0017 | Reviewers as Gatekeeper + Mandatory Second Opinion | Accepted |
| ADR-0018 | Centralize All Documentation Under Scribes | Accepted |
| ADR-0021 | Prospective Memory (Task Queue & Scheduler) | Accepted |
| ADR-0032 | Routine Templates for Standard Flows | Accepted |
| ADR-0034 | Workflow Monitor Interface | Accepted |
| ADR-0036 | Project Slug Convention — Stable Canonical Identifiers | Accepted |
| ADR-0037 | YAML-Based Agent and Task Configuration | Accepted |
| ADR-0038 | Knowledge Recall Protocol | Accepted |
| ADR-0039 | Pre-Flight Knowledge Check Enforcement | Accepted |
| ADR-0041 | Procedural Consolidation Tier — Bridge Pattern Detection to Pipeline | Accepted |
| ADR-0042 | ADR Implementation Phase Board | Accepted |
| ADR-0045 | Patch mem::remember to Create KV.relations Entry on Supersede | Accepted |
| ADR-0046 | Automatic Git Repository Initialization for Workspace Management | Accepted |
| ADR-0047 | Universal Test Execution Standard | Accepted |
| ADR-0048 | Mandatory Tool Documentation Review Before Implementation | Accepted |
| ADR-0049 | Lazy Senior Dev Philosophy | Accepted |
| ADR-0050 | Agentmemory → OpenCode RAG Migration | Accepted |
| ADR-0051 | opencode-mem as KodeHold Persistent Memory Backend | Accepted |
| ADR-0052 | Structured Durable Execution — Formal Checkpoint Schema and Auto-Checkpoint | Accepted |
| ADR-0053 | Replace ollama with vllm for Embeddings | Accepted |
| ADR-0054 | Replace opencode-rag with Graphify Knowledge Graph for Code Retrieval | Accepted |
| ADR-0060 | Loop-Engineering Integration for Workspace Management | Accepted |
| ADR-0055 | KodeHold Improvement Opportunities | Accepted |
| ADR-0056 | Agent Configuration Cleanup | Accepted |
| ADR-0057 | Migrate File-Based Memory to opencode-mem | Accepted |


See `docs/adr/README.md` for full details.

---

## 6. Project Lifecycle

### 6.1 States

```
INIT → ACTIVE → REVIEW → CLOSED → (REOPEN → ACTIVE)
```

| State | Description |
|-------|-------------|
| INIT | Design doc created, ADRs drafted, project scoped |
| ACTIVE | Implementation in progress, teams working |
| REVIEW | All work completed, Team Meeting review, testing |
| CLOSED | Project complete, context archived in opencode-mem |
| REOPEN | Project resurrected for new feature or bugfix |

### 6.2 Quality Gates (Markers)

Each state transition requires a quality marker before the gate can pass:

| Gate | Required Marker | Created By | Purpose |
|------|----------------|-----------|---------|
| INIT → ACTIVE | `.design_reviewed` | Reviewers | Design quality approved |
| INIT → ACTIVE | `.second_opinion_done` | Reviewers | Cross-model validation completed |
| ACTIVE — Gate 1 | `.design_review_v2` | Reviewers | Design reviewed before implementation |
| ACTIVE — Gate 2 | `.code_reviewed` | Reviewers | Code reviewed before testing |
| ACTIVE → REVIEW | `.testers_done` | Testers | Tests complete before review |
| REVIEW → CLOSED | Team Meeting | All 6 teams | Collective sign-off |
| CLOSED → REOPEN | `.impact_analysis_done` | Architects | Impact assessed before reopening |
| REOPEN → ACTIVE | `.second_opinion_done` | Reviewers | Cross-model validation for updated design |

Markers are deleted by the gate only after a successful transition pass path
(including INIT→ACTIVE confirmation when interactive). All lifecycle markers
are queued and cleaned after REVIEW→CLOSED passes.

**Gatekeeper authority:** Reviewers validate transitions before the Director runs gate.py (ADR-0017). Reviewers run `gate.py --validate-only` and return PASS/BLOCKED. The Director runs the actual gate only after Reviewers approve.

The ACTIVE phase enforces sequential flow: **Architects → Gate 1 → Engineers → Gate 2 → Testers → Gate 3**. Each gate marker must exist before the next team starts work.

See ADR-0016 for the full early review gate specification.

### 6.3 Reopening

When a project is reopened:
1. Director loads project context via `search_memories(query="<project> recent", scope="project")`
2. Design doc is updated with new requirements
3. Impact analysis is performed by Architects → `.impact_analysis_done`
4. New ADRs are written for significant changes
5. CLOSED→REOPEN gate passes → marker cleaned
6. Implementation proceeds as normal lifecycle

### 6.4 Commit Protection Protocol

To prevent data loss (inspired by ADR-0015 through ADR-0019 being lost when sessions ended before committing), KodeHold enforces a commit protection protocol:

- Before any checkpoint, state transition, or session end, untracked files in `docs/adr/`, `docs/design/`, and `.opencode/agents/` must be identified via `git status --short`
- The Director prompts the user for approval before committing any uncommitted files
- Commits use structured messages with conventional commit prefixes:
  - `docs(adr): ADR-00XX - <title>` for new ADR files
  - `docs(design): <description>` for design document changes
  - `config: <description>` for agent configuration changes
- If the user declines, the Director logs the warning via `add_memory` and continues — data loss risk is acknowledged
- Scribes verify file persistence before storing pre-transition context and escalate untracked files to the Director

---

## 7. Integration

### 7.1 OpenCode Compatibility

KodeHold runs as an OpenCode agent or set of agents. All interaction with the file system, LLM, and tools happens through OpenCode's standard interfaces. Configuration is done via `opencode.json` / `opencode.jsonc`.

### 7.2 Persistent Memory & Knowledge Retrieval

KodeHold uses **opencode-mem** as its persistent memory backend (per [ADR-0051](../adr/ADR-0051-opencode-mem-persistent-memory.md)), providing cross-session memory with semantic search, auto-capture, and automatic compaction. **Graphify** provides structural code and documentation retrieval. The two systems serve different purposes — Graphify for exact structural code queries, opencode-mem for runtime learnings and session context.

**opencode-mem** — an MCP server providing persistent memory with:
- Semantic search via vector embeddings (USearch backend, nomic-embed-text-v1 model)
- Auto-capture of conversation context without explicit agent action
- Project-scoped memory with configurable default scope
- Compaction to manage memory limits automatically
- Local-first storage at `~/.opencode-mem/data`

MCP tools available to all agents:

| Tool | Purpose |
|------|---------|
| `search_memories(query, scope?)` | Semantic search across stored memories |
| `add_memory(content, scope?, tags?)` | Store a new memory |
| `get_memory(id)` | Retrieve a specific memory by ID |
| `list_memories(scope?, tags?)` | List memories with optional filters |
| `update_memory(id, content)` | Update an existing memory |
| `delete_memory(id)` | Remove a memory |

> **Project scoping is MANDATORY.** Every `search_memories` and `add_memory` call MUST include `scope: "project"` to prevent cross-project memory bleed. KodeHold shares an opencode-mem instance with other agents. See [ADR-0051 §3b](../adr/ADR-0051-opencode-mem-persistent-memory.md) for details.

**Graphify** (per [ADR-0054](../adr/ADR-0054-replace-opencode-rag-with-graphify.md)) is KodeHold's mechanism for structural code retrieval. It uses tree-sitter AST parsing to build a deterministic, navigable graph of files, functions, classes, imports, and their relationships — providing exact structural queries. Installed via `uv tool install graphifyy` and registered via `graphify install`.

| Layer | What it retrieves | Strength |
|-------|------------------|----------|
| **Graphify (PRIMARY)** | Structural code relationships (callers, callees, imports, class hierarchy) | Exact, deterministic, navigable |
| **opencode-mem** | Runtime learnings, session context | Cross-session persistence |

The code retrieval flow is: **Graphify → opencode-mem**. Agents use Graphify first for structural queries, then `search_memories` for runtime context.

> **Note on built-in OpenCode tools:** OpenCode provides platform-level primitives (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) that exist at the tool level. These are NOT part of KodeHold's documented retrieval workflow. All code retrieval goes through Graphify per ADR-0054. See [ADR-0054](../adr/ADR-0054-replace-opencode-rag-with-graphify.md).

> **Previous systems:** The agentmemory daemon (`iii`, port 3111) was removed per ADR-0050. opencode-mem is now the sole memory system. OpenCode RAG's standalone `opencode-rag mcp` server was removed per ADR-0054 — Graphify is now the sole code retrieval method. See [ADR-0050](../adr/ADR-0050-agentmemory-to-opencode-rag-migration.md), [ADR-0051](../adr/ADR-0051-opencode-mem-persistent-memory.md), [ADR-0054](../adr/ADR-0054-replace-opencode-rag-with-graphify.md), and [ADR-0057](../adr/ADR-0057-migrate-file-memory-to-opencode-mem.md).

### 7.3 CLI Operations

OpenCode's native tools (`glob`, `grep`, `read`, `bash`) are used for all file and code operations. No external CLI proxy is required.

### 7.4 Skills System

KodeHold uses OpenCode skills (`.opencode/skills/<name>/SKILL.md`) for reusable,
on-demand instruction sets shared across multiple agents. Skills are loaded
via the `skill` tool with zero token cost until invoked.

| Skill | Purpose | Used by |
|-------|---------|---------|
| `state-awareness` | Lifecycle state check preamble and mismatch reporting | All 6 team subagents |
| `investigate` | 4-phase systematic debugging (Iron Law, pattern analysis, 3-strike rule) | FLS, Engineers, Reviewers, Director |

See `docs/adr/ADR-0013-investigate-skill.md` for the full ADR on the investigate skill.

### 7.5 Session Context Compression

On small-context models (Ollama 32K), chat history grows with every delegation round and eventually overflows. Session context compression is handled automatically by opencode-mem's auto-capture and compaction — no manual checkpoint protocol is needed.

**Compression mechanism:** opencode-mem captures conversation context automatically and compacts memories when limits are reached. This replaces the file-based checkpoint system removed per ADR-0057.

**Agent guidance:**
- Director session lifecycle uses `search_memories` to load recent context at session start
- No manual checkpoint files need to be created or managed
- opencode-mem handles compaction transparently

**Note:** ADR-0052 (Structured Durable Execution) was the formal YAML frontmatter checkpoint schema. The checkpoint system has been superseded by opencode-mem's auto-capture per ADR-0057. ADR-0019 (Session Context Compression) is now fully Superseded.

### 7.6 Adopted Project Symlinks (ADR-0012)

When KodeHold adopts an existing project, `workspace.py adopt` creates a **symlink** from `workspaces/<name>/` to the real project directory. This is a symlink, not a copy — the project stays at its original location.

**Path behavior:**
| Operation | Path | Resolves To |
|-----------|------|-------------|
| File access | `workspaces/<name>/src/file.py` | `<real_path>/src/file.py` |
| Real path | `realpath workspaces/<name>/` | `<real_path>/` |
| Git operations | `git -C workspaces/<name>/` | `<real_path>/` |
| Module imports | Through symlink | Transparent — imports work normally |

**Agent guidance:**
- **Engineers:** Use the symlink path for consistency. Module imports and build commands resolve through symlinks transparently.
- **Testers:** Symlinked paths can cause test collection failures (pytest, jest). Use `realpath` to resolve to the absolute path when test discovery fails. Set `PYTHONPATH`/`NODE_PATH` to the real path.
- **All teams:** Use `realpath workspaces/<name>` to verify the symlink target exists before relying on it.

**Known issues (ADR-0012):**
- Symlinks break if the target is moved without recreating the symlink
- Some test frameworks produce confusing errors on symlinked paths
- Error messages may show the symlink path instead of the real path

### 7.7 Prospective Memory (ADR-0021)

Prospective memory enables deferred actions, recurring tasks, and future intentions that survive session boundaries. Instead of losing "I should check X next time" when a session ends, tasks are stored in opencode-mem and checked at session start.

**Scope (v1):**
- Deferred tasks — execute after a timestamp
- Recurring tasks — re-create after execution (no scheduler — AI agents have no time sense)

**Out of scope (future):**
- Trigger-based execution — requires event monitoring that AI agents cannot do reliably

#### Storage Format

Tasks are stored via `add_memory` with `tags: ["prospective"]`. The content uses a structured format:

```
[PROSPECTIVE-TASK]
id: <short-uuid>
type: deferred|recurring
action: <what to do — plain language>
execute_after: <ISO 8601 timestamp>
recurring_interval: <duration, e.g. "2d", "1w">  (recurring only)
priority: critical|high|medium|low
context: <additional context needed to execute>
created_at: <ISO 8601 timestamp>
status: pending
```

#### Task Types

| Type | Fields | Behavior |
|------|--------|----------|
| **Deferred** | `execute_after` | Checked at session start. If `execute_after <= now()` → present to Director. One-shot. |
| **Recurring** | `execute_after` + `recurring_interval` | Same as deferred, but after execution, Scribes re-creates with `execute_after = now + interval`. |

#### Session-Start Integration

Add a new step in Director's session lifecycle (section "Session Lifecycle" in director.md), between step 1 (context loading) and step 2 (session summary):

```
1.5. Check prospective tasks:
     search_memories(query="prospective pending", scope="project")
     Parse results for execute_after <= now()
     If due tasks found → present to user as "Pending tasks:"
     User decides: execute now / skip / dismiss
```

#### Task Lifecycle

```
Created → Pending → [Due] → Executing → Completed
                                        ↓
                               Re-created (recurring) or forgotten (deferred)
```

- **Created:** Scribes stores via `add_memory(content="...", tags=["prospective"], scope="project")`
- **Due:** Session-start check finds `execute_after <= now()` — presented to Director
- **Executing:** Director delegates to appropriate team
- **Completed:** Scribes updates status in the file's frontmatter or removes the file
- **Recurring re-create:** After completion, Scribes stores new task with `execute_after = now + interval`

#### Token Budget

| Priority | Max Tasks | Rationale |
|----------|-----------|-----------|
| Critical | 5 | Must execute — blocking issues |
| High | 10 | Important but not blocking |
| Medium | 15 | Nice-to-have deferred actions |
| Low | 5 | Recurring maintenance |

Total: ~35 tasks max. Scribes enforces by expiring oldest low-priority tasks when limit is reached.

#### TODO.md Integration

Prospective tasks are **separate** from TODO.md. TODO.md tracks "what we're building now"; prospective memory tracks "what to do later." A summary line in TODO.md can reference active prospective task count:

```markdown
## Prospective Tasks
- 3 deferred tasks in opencode-mem (next due: 2026-06-01)
```

Scribes updates this line when creating/expiring tasks.

---

## 8. LLM Support

### 8.1 Bring Your Own Model

KodeHold does not mandate a specific LLM model. The user's global OpenCode model configuration is used as the default for all operations. No per-team model overrides are set in agent definitions — all teams inherit the same default model.

**Local inference:** Ollama is the primary local inference provider for LLM generation:

- **Ollama** — single-process server for LLM inference (qwen3.5:9b). Runs on GPU with full VRAM allocation. The provider configuration in `opencode.json` enables Ollama as the default option.

**Hybrid Embedding Strategy (ADR-0053):** Embeddings use sentence-transformers on CPU, keeping Ollama dedicated to LLM inference:

| Component | Location | Model | Purpose |
|-----------|----------|-------|---------|
| LLM (Ollama) | GPU, port 11434 | qwen3.5:9b | Chat completions, analysis |
| Embeddings (CPU) | Local Python | bge-m3 (567M params) | Vector embeddings for RAG |

**Why sentence-transformers on CPU:**
- bge-m3 uses XLMRobertaModel (encoder-only, Transformers-compatible)
- ~567M parameters, fast enough on CPU (<100ms for short texts)
- Zero VRAM consumption — leaves full GPU for LLM
- No Ollama blocking — embedding requests served independently
- Simple setup: `pip install sentence-transformers`

This architecture is documented in ADR-0053 (Hybrid Embedding Strategy — sentence-transformers + Ollama).

### 8.2 Second Opinion

For critical decisions, the Director can request a second opinion from a different AI model:
- Design decisions above a complexity threshold
- Security-critical code
- Architectural trade-offs
- Cross-model validation for bug-prone areas

Protocol:
1. Director identifies a decision requiring second opinion
2. Context is packaged (design doc excerpt + code + question)
3. Request is sent to secondary LLM (different model/provider)
4. Response is fed back to the primary workflow
5. Director reconciles differences or escalates

---

## 9. Token Optimization Strategy

> **Note:** The Token Optimization Strategy has been superseded by **ADR-0060** (Loop-Engineering Integration for Workspace Management). ADR-0060 defines loop-engineering integration, token cost estimation via `loop-cost`, and safety mechanisms via `loop-gate` and `loop-context`. See ADR-0060 for the current token budget protocol.

---

## 10. File Layout

```
kodehold/
├── .opencode/                     # OpenCode agent/subagent configs
│   ├── opencode.json              # Local overrides
│   ├── agents/
│   │   ├── architects.md          # Design authority (+ deprecated YAML frontmatter)
│   │   ├── engineers.md           # Implementation team (+ deprecated YAML frontmatter)
│   │   ├── fls.md                 # Front Line Support (+ deprecated YAML frontmatter)
│   │   ├── reviewers.md           # Code/design review (+ deprecated YAML frontmatter)
│   │   ├── testers.md             # Verification team (+ deprecated YAML frontmatter)
│   │   ├── scribes.md             # Memory and documentation (+ deprecated YAML frontmatter)
│   │   ├── director.md            # Orchestrator (+ deprecated YAML frontmatter)
│   │   └── second-opinion.md      # Cross-model review (+ deprecated YAML frontmatter)
│   ├── references/
│   │   └── kodehold-protocol.md   # Shared protocol reference
│   ├── plugins/
│   │   └── graphify.js          # Graphify knowledge graph plugin
│   └── skills/                    # Reusable skills
│       ├── README.md              # Skill index
│       ├── investigate/
│       │   └── SKILL.md           # Systematic debugging protocol (4 phases)
│       ├── ponytail-audit/
│       │   └── SKILL.md           # Whole-repo over-engineering audit
│       ├── ponytail-review/
│       │   └── SKILL.md           # Diff-level over-engineering review
│       └── state-awareness/
│           └── SKILL.md           # Lifecycle state checking + mismatch protocol
├── config/                        # YAML-based configuration (ADR-0037)
│   ├── agents.yaml                # Agent configuration — all 8 agents
│   ├── agents.schema.json         # JSON Schema for agents.yaml validation
│   └── tasks.yaml                 # Workflow and gate definitions
├── docs/
│   ├── design/
│   │   └── README.md              # This file — main design document
│   ├── adr/
│   │   ├── README.md              # ADR index
│   │   ├── ADR-0001-*.md
│   │   └── ...
│   └── decisions/                 # Working notes, options analysis
├── .github/workflows/
│   ├── kodehold-ci.yml            # CI pipeline (smoke, yaml, integration)
├── scripts/
│   ├── benchmark.sh               # Performance benchmarks
│   ├── detect-test-framework.sh   # Non-Python test framework detection (ADR-0047)
│   ├── gate.py                    # Lifecycle state transition gate
│   ├── lib/                       # Script helper library
│   ├── migrations/                # Data/config migration scripts
│   ├── ship.py                    # Shipping gate checklist automation
│   ├── sync-agent-config.sh       # Syncs frontmatter between .md and agents.yaml
│   ├── validate-config.sh         # Validates config/agents.yaml against schema
│   └── workspace.py               # Workspace management (init, adopt)
├── tests/
│   ├── run.sh                     # Test suite runner
│   ├── smoke/                     # Script functionality tests
│   ├── init/                      # YAML config tests (Python/pytest)
│   └── integration/               # Gate & script flow tests
├── opencode.json                  # OpenCode project configuration (Ollama provider, permissions)
├── AGENTS.md                      # Director — orchestrator, lifecycle, quality gates
├── README.md                      # Project overview
├── VERSION.md                     # Version history
├── TODO.md                        # Task list
├── CHANGES.md                     # Changelog
└── .gitignore                     # Git ignore rules
```

---

## 11. Changelog

- **v1.25.0 (2026-07-22):** ADR-0060 implemented — Loop-Engineering Integration for Workspace Management complete. Added `workspace.py loop` subcommand (list, enable, disable, run), `workspace.py cron` subcommand (install, remove, list), `workspace.py audit/cost/sync` commands. Created `workspace-loop-management` skill. Updated §3.0 (Loop Engineering Operational Framework) with implementation status. All three phases complete: Phase 1 (loop-init integration), Phase 2 (enable/disable/run + cron), Phase 3 (audit/cost/sync).
- **v1.24.0 (2026-07-22):** ADR-0060 accepted — Loop-Engineering Integration for Workspace Management. Updated §3.0 (Loop Engineering Operational Framework) to reference ADR-0060 instead of ADR-0058. Updated §5 (ADR Index) — replaced ADR-0058 with ADR-0060. Updated §9 (Token Optimization Strategy) to reference ADR-0060. ADR-0060 supersedes archived ADR-0058 and ADR-0059.
- **v1.23.0 (2026-07-22):** Loop Engineering Phase 2 completion. Created `scripts/loop_runner.py` — pure Python L1 loop engine replacing `opencode run`. Added Discord webhook notifications (`--webhook` flag). ADR-0059 accepted — workspace.py rewritten with 10 subcommands, YAML registry (config/workspaces.yaml), copy mode default. ADR-0007 and `.opencode/memory/` references cleaned from all active operational files.
- **v1.22.0 (2026-07-18):** ADR-0057 completion — removed deprecated file-based memory references from design doc. Updated §3.6 (Scribes), §6.1 (CLOSED state), §6.3 (Reopening), §6.4 (Commit Protection), §7.2 (Previous systems note corrected), §7.5 (Session Context Compression rewritten to reflect opencode-mem auto-capture), §7.7 (Prospective Memory updated to use `add_memory`/`search_memories`), §9 (Token Optimization table), §10 (File Layout — removed `commands/`, `graphify-knowledge-flow`, `resume` skill). Removed checkpoint/compression protocol references. See ADR-0057 for full migration details.
- **v1.22.0 (2026-07-21):** ADR-0058 accepted — Loop Engineering integration framework, Token Budget Protocol v2, three-phase roadmap (Foundation → Automation → Deep Integration). Added §3.0 (Loop Engineering Operational Framework) with Phase 1 implementation plan. Updated §5 (ADR Index) — added ADR-0055/0056/0057/0058 entries. Simplified §9 (Token Optimization Strategy) to reference ADR-0058. Updated ADR index in `docs/adr/README.md`.
- **v1.21.0 (2026-07-14):** ADR-0054 completion — replaced all remaining OpenCode RAG references with Graphify across all documentation. Created `graphify-knowledge-flow` skill to replace `opencode-rag-knowledge-flow`. Updated AGENTS.md, design doc §7.2/§7.4/§10, skills README, root README, ADR-0050, ADR-0051, and config/agents.yaml. Graphify is now the sole documented code retrieval method; platform-level OpenCode RAG primitives (search_semantic, find_usages, get_file_skeleton, describe_image) are explicitly noted as not part of KodeHold's workflow.
- **v1.20.0 (2026-07-14):** ADR-0054: OpenCode RAG → Graphify migration. Replaced standalone `opencode-rag mcp` server with Graphify knowledge graph as the sole code retrieval mechanism. Graphify handles all code retrieval; built-in OpenCode RAG tools are platform-level primitives, not part of KodeHold's documented workflow. Updated Section 5 (ADR Index) and Section 7.2 (Persistent Memory & Knowledge Retrieval) with Graphify as sole retrieval layer. References ADR-0054.
- **v1.19.0 (2026-07-09):** Updated Section 8.1 (Bring Your Own Model) to document hybrid embedding strategy: sentence-transformers on CPU for embeddings (bge-m3), Ollama for LLM inference (qwen3.5:9b). Replaces previous vLLM dual-instance plan. References ADR-0053 (Hybrid Embedding Strategy — sentence-transformers + Ollama).
- **v1.18.0 (2026-07-02):** ADR-0052 promoted from Proposed → Accepted. Updated ADR-0019 superseded-by reference from "agentmemory" to "ADR-0052". Updated ADR README index with ADR-0052 Accepted entry.
- **v1.17.0 (2026-07-02):** Registered ADR-0052 (Structured Durable Execution, Proposed) — formal YAML frontmatter checkpoint schema and auto-checkpoint on every delegation. Combines issues #59 (checkpoint format) and #35 (durable execution). Updated Section 7.5 to note ADR-0052 supersedes ADR-0019's compression protocol. Added ADR-0052 to ADR index.
- **v1.16.0 (2026-07-01):** ADR-0051 integration — replaced Section 7.2 (Persistent Memory & Knowledge Retrieval) with opencode-mem as persistent memory backend and OpenCode RAG for code/doc search. Added ADR-0051 to ADR index. ADR-0050 §5 (File-Based Persistent Storage) superseded. See ADR-0051 for full rationale and configuration.
- **v1.15.2 (2026-06-28):** ADR-0050 implementation — replaced Section 7.2 (Agentmemory → Persistent Memory & Knowledge Retrieval), removed Project Slug Migration subsection, updated all remaining agentmemory references throughout design doc to reflect file-based memory storage and OpenCode RAG tools. See ADR-0050 for full migration mapping.
- **v1.15.1 (2026-06-27):** ADR-0050 promoted from Proposed → Accepted after Reviewers approval. Second opinion skipped per user request. ADR status updated, `.design_reviewed` marker created.
- **v1.15.0 (2026-06-27):** Registered ADR-0050 (Agentmemory → OpenCode RAG Migration, Proposed) — replaces agentmemory dependency with OpenCode's built-in RAG tools (search_semantic, find_usages, get_file_skeleton, describe_image). Updates Section 5 (ADR Index) and Section 7.4 (Skills table). See ADR-0050 for full migration plan.
- **v1.14.0 (2026-06-19):** Added ADR-0049 (Lazy Senior Dev Philosophy, Proposed) — adopts Ponytail's "The Ladder" as KodeHold's coding philosophy. Principle #9 added to design doc. Integration points: engineers.md (workflow step 2c), reviewers.md (checklist items), director.md (delegation reference).
- **v1.14.1 (2026-06-19):** ADR-0049 promoted from Proposed → Accepted after Reviewers approval and second opinion pass. All indexes updated, `.second_opinion_done` marker cleaned.
- **v1.13.3 (2026-06-14):** Implemented ADR-0046 — auto git-init for workspace init and adopt. `ws_init()` now creates git repo, `ws_adopt()` creates git repo if missing, new `ensure-git` subcommand for backfill. Both deepresearch and pai-model-router workspaces backfilled.
- **v1.13.2 (2026-06-14):** Implemented ADR-0048 — Mandatory Tool Documentation Review Before Implementation. Updated all 5 team agents (architects, engineers, testers, reviewers, fls) with documentation-reading workflow steps. ADR-0048 promoted from Proposed to Accepted.
- **v1.13.1 (2026-06-13):** Added ADR-0047 (Universal Test Execution Standard, Accepted). Defines three test modes (quick/full/smoke), venv discovery chain, symlink handling, non-Python framework detection script, and agent file integrations. All 4 agent files updated to reference the standard.
- **v1.13.0 (2026-06-13):** Added ADR-0046 (Automatic Git Repository Initialization for Workspace Management, Proposed) to ADR Index table. The ADR specifies that `workspace.py init` and `workspace.py adopt` should automatically initialize git repositories when none exist.
- **v1.12.9 (2026-06-06):** ADR-0045 promoted from Proposed → Accepted after Reviewers approval. Patch mem::remember to create KV.relations entry on supersede.
- **v1.12.8 (2026-06-06):** Viewer: Added 'Memory Types' tab showing all semantic facts and procedural memories with search/filter. Dashboard cards now link to full view when >5 items exist.
- **v1.12.7 (2026-06-06):** Archive detection tested — 25 unit tests added in `test/archive-detection.test.ts` covering happy path (archive detection fires `/session/end`), negative case (no `time.archived` → no action), and edge cases (missing session ID, null info, malformed time object). All tests pass. ADR-0044 Compliance table updated with unit test verification.
- **v1.12.6 (2026-06-06):** ADR-0044 implementation — three in-plugin session lifecycle mechanisms deployed in agentmemory-capture.ts: archive detection (session.updated with time.archived), per-process 24-hour idle timer (resets on activity), and process exit handlers (SIGTERM/SIGINT). Removed cron-based cleanup scripts (`scripts/agentmemory-session-cleanup.sh`, `scripts/agentmemory-session-cleanup.py`). Updated ADR-0044 status remains Accepted.
- **v1.12.5 (2026-06-06):** ADR-0044 promoted from Proposed → Accepted after Reviewers approval and second opinion. Updated ADR index table. ADR file updated with: clarified cron diagram, moved SIGKILL orphan to Deferred/Future Work, strengthened configurable timeout rationale, added v1.1 changelog entry.
- **v1.12.4 (2026-06-06):** Added ADR-0044 (Automatic Session Lifecycle Management, Proposed) to ADR Index table. The ADR replaces cron-based session cleanup with three in-plugin mechanisms in the agentmemory-capture plugin: archive detection via session.updated, per-process 24-hour idle timer, and process exit handlers (SIGTERM/SIGINT).
- **v1.12.2 (2026-06-03):** Knowledge Recall Protocol implementation — fixed broken lesson recall by adding project scoping (`project="kodehold"`), increasing limit to 10, using team-prefixed query format, adding fallback step. Batch-tagged 122 existing lessons with companion lessons. Created ADR-0038 and design doc `docs/design/knowledge-recall.md`.
- **v1.12.1 (2026-06-03):** Agentmemory v0.9.25 upgrade. 5 obsolete patches removed, archived to patches-v0.9.24/. Viewer bind via AGENTMEMORY_VIEWER_HOST env var. All upstream bug fixes from our GitHub reports now included.
- **v1.12.0 (2026-06-02):** Implemented Issue #34 — YAML-based agent & task configuration. Phase 1-4 complete: `config/agents.yaml`, `config/agents.schema.json`, `config/tasks.yaml`, `validate-config.sh`, `sync-agent-config.sh`, schema validation tests (46 tests). ADR-0037 promoted from Proposed → Accepted.
- **v1.11.0 (2026-06-02):** Added ADR-0037 (YAML-Based Agent and Task Configuration, Proposed) — YAML-based config schema with `config/agents.yaml`, `config/tasks.yaml`, and `config/agents.schema.json`. Updated §10 (File Layout) to include `config/` directory, §5 ADR index table with ADR-0037. The ADR defines the YAML schema, JSON Schema validation, trigger extraction, migration strategy from `.md` frontmatter, and backwards-compatible overlay pattern.
- **v1.10.2 (2026-06-02):** Added ADR-0036 (Project Slug Convention — Stable Canonical Identifiers, Proposed) to ADR Index table. The ADR defines a formal lowercase kebab-case `[a-z][a-z0-9-]*` slug format for agentmemory `project` identifiers, superseding ADR-0028 Section 6's full-filesystem-path approach.
- **v1.10.1 (2026-06-02):** ADR status audit and sync — corrected 7 ADR statuses across design doc table and ADR README index (ADR-0004, ADR-0009 → Deprecated; ADR-0014, ADR-0019, ADR-0021, ADR-0026 → Superseded; ADR-0027 → Deprecated; ADR-0028, ADR-0029 → Accepted). Updated ADR-0026 file status to Superseded with note on cross-provider second opinion subagent. Updated ADR-0029 file status to Accepted noting all 5 migration phases complete. Updated ADR-0033 inter-agent-signals-sentinels file status to Superseded noting ADR-0033 (Crystals + Signals) as replacement.
- **v1.10.0 (2026-06-02):** Added ADR-0035 (Custom KodeHold Viewer) to ADR index — standalone interactive HTML viewer with Frontier, Routines, and Signals tabs plus project filter for Actions. Also added ADR-0034 (Workflow Monitor Interface, Accepted) to ADR index.
- **v1.9.0 (2026-06-02):** Phase 2 ICM→Agentmemory migration — replaced all ~28 ICM references throughout the design document with agentmemory equivalents. Updated Section 2 (Principle 4), Section 3 (Scribes box + 3.6), Section 6 (lifecycle states, reopening, commit protection), Section 7.2 (full rewrite describing agentmemory as the active memory system), Section 7.4 (skill reference), Section 7.5 (session context compression), Section 7.7 (prospective memory), Section 8.2 (light mode), Section 9 (token optimization), and Section 10 (file layout). Updated ADR-0030 status to Accepted in file header and ADR index. Synced ADR-0031 and ADR-0032 statuses in ADR index.
- **v1.5.0 (2026-05-31):** ADR-0028: Agentmemory Project Detection Strategy — three-stage `resolveProject()` in OpenCode plugin. Replaces single-line project path assignment with env var / git toplevel / fallback resolution.
- **v1.4.23 (2026-05-31):** Documented agentmemory binding configuration — `iii` daemon listens on `0.0.0.0` (all interfaces) via `~/.agentmemory/iii-config.yaml`. Added note in Section 7.2 (ICM) explaining the custom config approach that survives npm updates.
- **v1.4.22 (2026-05-30):** ICM memoir restructure — 7 team memoirs consolidated into `kodehold-teams` (27 concepts, 16 links). Learnings consolidated into `kodehold-learnings` (63 concepts, 68 links). All ADR references in ADR-0027, ADR-0023, ADR-0009 updated. CHANGES.md, VERSION.md bumped to 0.17.0.
- **v1.4.21 (2026-05-29):** Implemented ADR-0021 — Prospective Memory (Task Queue & Scheduler). Added section 7.7 to design doc. Updated director.md session lifecycle with prospective task check (step 1.5). Updated scribes.md with Prospective Memory CRUD operations. ADR-0021 status promoted to Accepted. Scope: deferred + recurring tasks only; trigger engine deferred to future.
- **v1.4.20 (2026-05-29):** ADR-0027 implementation review fixes — SKILL.md renumbered to match ADR-0027 step numbering (steps 1-2 Pre-task, steps 4-8 Post-task, step 3 removed from knowledge flow). docs/icm-knowledge-flow.md updated with ADR-0027 reference, 3 invocation modes, Scribes Post-task-only documentation, consolidation threshold fixed (>5→>7). Fixed `memoit=` typo to `memoir=`. Added Full mode to Mode Selection table. Updated second-opinion.md with Post-task mode declaration.
- **v1.4.19 (2026-05-29):** Updated all 6 agent files with ADR-0027 invocation modes (Pre-task/Post-task). Scribes uses Post-task only; other teams use Pre-task default. Fixes token waste and semantic confusion.
- **v1.4.18 (2026-05-29):** Registered ADR-0027 — ICM Knowledge Flow Invocation Modes (Proposed). Added to ADR index in both `docs/adr/README.md` and design doc ADR table.
- **v1.4.17 (2026-05-29):** Fixed missing YAML frontmatter in ICM Knowledge Flow skill file (`.opencode/skills/icm-knowledge-flow/SKILL.md`). Skill now registers correctly with OpenCode's skill discovery.
- **v1.4.16 (2026-05-29):** Registered ADR-0026 — Second Opinion Same-Model Bias Enforcement (Proposed). Added to ADR index in both `docs/adr/README.md` and design doc ADR table.
- **v1.4.15 (2026-05-29):** Shipping gate alignment — AGENTS.md and ship.sh now agree on step count (8 total: 1 manual Team Meeting + 7 automated). CHANGES.md check upgraded from warn to fail.
- **v1.4.14 (2026-05-29):** Fixed FLS→Scribes protocol inconsistency (bug #15) — removed direct ICM storage from FLS workflow, now delegates through Director→Scribes per ADR-0010.
- **v1.4.13 (2026-05-29):** Added Dependabot configuration (`.github/dependabot.yml`) for weekly automated dependency updates — GitHub Actions and npm packages in `.opencode/`.
- **v1.4.12 (2026-05-29):** GitHub MCP server upgrade — replaced deprecated `@modelcontextprotocol/server-github` npm package with official `github/github-mcp-server` Go binary v1.1.2. Eliminates intermittent authentication failures. Closed issues #22 (stale test count) and #25 (GitHub MCP auth).
- **v1.4.11 (2026-05-29):** Fixed ADR-0019 status to "Accepted" across all ADR files (ADR-0019 file, ADR README index). Design doc already had correct status.
- **v1.4.10 (2026-05-29):** Documentation audit and fixes — corrected test count (10 → 12), marked light mode as complete, fixed version header consistency, updated README.md ADR range and test count, closed stale GitHub issues (#4, #5).
- **v1.4.9 (2026-05-29):** Added ADR-0020 through ADR-0025 — AI Agent Memory Stack features: Hierarchical Memory (Hot/Warm/Cold), Prospective Memory (Task Queue & Scheduler), Automated Episodic Extraction, Semantic Memory Automation, Shared Memory (Multi-Agent Alignment), and A2A Protocol (Agent-to-Agent Coordination). All status: Proposed.
- **v1.4.8 (2026-05-29):** Implemented ADR-0019 — Session Context Compression. Updated director.md with compression triggers every 4 rounds, summary template, and consolidation policy. Updated scribes.md with summary template, escalation path for large topics. ADR-0019 status promoted to Accepted. This completes the last remaining KODEHOLD_LIGHT=1 component.
- **v1.4.7 (2026-05-29):** Added Principle #8 — Safe Operations (no automatic `git clean -fd`). Added corresponding constraints in director.md and AGENTS.md. Added Commit Protection Protocol — prevents data loss from uncommitted ADR, design, and agent files at session end. Updated Director agent with 5-step protocol, Scribes agent with file persistence verification step in pre-transition workflow, and design doc section 6.4.
- **v1.4 (2026-05-28):** Added ADR-0017 — Reviewers as gatekeeper for lifecycle transitions and mandatory second opinion on ADRs/design documents. Updated Review Cadence (4.2) with gatekeeper authority and `.second_opinion_done` marker. Updated Quality Gates (6.2) with second opinion markers for INIT→ACTIVE and REOPEN→ACTIVE.
- **v1.4.1 (2026-05-28):** ADR-0017 status updated to Accepted. Documented `--yes` flag ordering constraint in gate.sh (must be first flag).
- **v1.4.2 (2026-05-28):** gate.sh `--reviewer-mode` output now includes `CHECKS` and `MARKERS_REQUIRED` fields per ADR-0017. `CHECKS` lists individual check results (e.g., `design_reviewed:PASS,second_opinion_done:FAIL`). `MARKERS_REQUIRED` lists markers required for the transition.
- **v1.4.3 (2026-05-28):** Added ADR-0019 — Session context compression via periodic ICM summaries. Added section 7.5 (Session Context Compression) to Integration. Updated token optimization table with compression savings estimate.
- **v1.4.4 (2026-05-29):** Clarified Section 7.2 (ICM) — workspace/adopted projects do not get their own `.icm/`; all project memory uses the central `.icm/` with topic prefix scoping (`kodehold-<project>-*`).
- **v1.4.5 (2026-05-29):** Fixed ADR status inconsistencies — ADR-0015, ADR-0016 promoted to Accepted; ADR-0019 status updated to "Designed — not yet implemented"
- **v1.3 (2026-05-28):** Added early review gates (ADR-0016) — three-checkpoint review system in ACTIVE phase. Updated Review Cadence (4.2) and Quality Gates (6.2) with Gate 1 (design review, `.design_review_v2`) and Gate 2 (code review, `.code_reviewed`).
- **v1.2 (2026-05-27):** Clarified quality-gate marker semantics so cleanup
  happens only on successful pass paths; documented INIT→ACTIVE non-interactive
  confirmation bypass behavior (`--yes`, `OPENCODE_NONINTERACTIVE=true`).
