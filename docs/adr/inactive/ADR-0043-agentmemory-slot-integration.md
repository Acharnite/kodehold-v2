---
status: Accepted
phase:
  current: 4
  total: 4
  names:
    1: "Fill empty slots"
    2: "Restructure populated slots"
    3: "Update agent files — remove migrated boilerplate"
    4: "Let agentmemory auto-populate 3 slots"
  status:
    1: done
    2: done
    3: done
    4: done
---

# ADR-0043: Agentmemory Slot Integration for KodeHold

## Status

Deprecated

**Version:** 3.0
**Last Updated:** 2026-06-06

### Changelog

| Version | Date | Changes |
|---------|------|---------|
| 3.0 | 2026-06-06 | Phase 4: agentmemory auto-populates session_patterns, pending_items, project_context via slot-reflect. We no longer write to these 3 slots. Documented how to USE the auto-populated data. Curated project reference content moved to AGENTS.md. |
| 2.1 | 2026-06-06 | Accepted. Final version. |
| 2.0 | 2026-06-06 | Removed dual-instance architecture. KodeHold is single agentmemory instance. Bob uses separate lightweight memory server. |
| 1.2 | 2026-06-06 | Add dual-instance architecture decision: run two separate agentmemory instances (KodeHold + Bob) instead of relying on global slots to prevent cross-project pollution |
| 1.1 | 2026-06-06 | Address review feedback: fix guidance timing, clarify global safety rules, reword net token metric, add global domain-agnostic principle, migration rationale, permissions note, slot corruption risk, size limit flexibility |
| 1.0 | 2026-06-06 | Initial proposal |

## Context

### Slot System Overview

Agentmemory provides 8 editable, size-limited memory units (slots) that are auto-injected into agent context. Slots survive restarts, persist across sessions, and require zero retrieval overhead — they are always available. There are two scopes:

| Scope | Injects into | Count |
|-------|-------------|-------|
| `global` | ALL projects | 3 slots |
| `project` | Current project only | 5 slots |

Pinned slots are always injected into context. Unpinned slots are available on demand.

**Size limit flexibility:** The default `sizeLimit` values (1000-3000 chars) shown in the Current Slot State table can be increased per slot up to the 20000 char hard cap if needed. This should be done sparingly and with awareness of the context injection cost — larger slots mean more tokens consumed per session.

### Current Slot State (6 June 2026)

| # | Slot | Size Limit | Scope | Pinned | Current Content | Assessment |
|---|------|-----------|-------|--------|----------------|------------|
| 1 | `persona` | 1000 | global | ✅ | **Empty** | Wasted slot — zero content |
| 2 | `tool_guidelines` | 1500 | global | ✅ | **Empty** | Wasted slot — zero content |
| 3 | `user_preferences` | 2000 | global | ✅ | **Empty** | Wasted slot — zero content |
| 4 | `guidance` | 1500 | project | ✅ | **Empty** | Wasted slot — zero content |
| 5 | `pending_items` | 2000 | project | ✅ | Auto-populated by agentmemory | Agentmemory appends TODO-detected items from observations. We no longer write manually. |
| 6 | `project_context` | 3000 | project | ✅ | Auto-populated by agentmemory | Agentmemory appends file paths touched in recent sessions. Curated reference moved to AGENTS.md. |
| 7 | `self_notes` | 1500 | project | ❌ | **Empty** | Unused — zero content, unpinned (opt-in read) |
| 8 | `session_patterns` | 1500 | project | ❌ | Auto-populated by agentmemory | Agentmemory overwrites with reflection data (last reflection, commands, errors). |

**Summary:** 5 of 8 slots curated by KodeHold (persona, tool_guidelines, user_preferences, guidance, self_notes). 3 slots auto-populated by agentmemory (session_patterns, pending_items, project_context).

### The Problem: Hardcoded Boilerplate

KodeHold hardcodes rules and behavioral guidelines directly in agent files (`director.md`, `scribes.md`, `AGENTS.md`, and team agent files under `.opencode/agents/`). This creates four structural problems:

#### 1. Duplication

The same rules appear in multiple files. For example:

- **Delegation tables** appear in both `AGENTS.md` (lines 17-28) and `director.md` (lines 410-418, 434-446)
- **Triage-check tables** appear in both `AGENTS.md` (lines 32-44) and `director.md` (lines 316-336)
- **State transition tables** appear in both files
- **Gate tables** appear in both files

#### 2. Token Waste

The director.md is 710 lines (~22,000 tokens). AGENTS.md is 97 lines (~3,000 tokens). Together they contain significant duplicate content that could be:

- Stored once in a slot (auto-injected)
- Removed from individual agent files
- Saving an estimated 1,600+ tokens of duplicate boilerplate

#### 3. Drift

When a rule changes (e.g., a new delegation pattern), it must be updated in every file that contains it. In practice, this leads to inconsistent rules across files — some updated, some stale.

#### 4. Cross-Session Amnesia

Agent files are static. They cannot reflect session-specific context, recent patterns, or evolving preferences. Slots persist and can be updated — making them the natural home for:

- Session directives (`guidance` slot — replaced every session)
- Evolving user preferences (`user_preferences` slot — updated when preferences change)
- Behavioral pattern accumulation (`session_patterns` slot — appended every session)

### Slot Management MCP Tools

Agentmemory provides 6 MCP tools for slot management, all available to every agent:

| Tool | Function | Used For |
|------|----------|----------|
| `memory_slot_create` | Create new slot | Setup (no new slots needed — all 8 exist) |
| `memory_slot_get` | Read single slot | Reading specific slot content |
| `memory_slot_list` | List all slots | Discovery |
| `memory_slot_replace` | Replace slot content | **Structured writes** (preferred for most slots) |
| `memory_slot_append` | Append text to slot | Append-log patterns (`pending_items`, `self_notes`) |
| `memory_slot_delete` | Delete slot | Cleanup |

### Key Forces

1. **Cross-session persistence** — Slots survive agent restarts and project reopens. Hardcoded rules do not.
2. **Global sharing** — 3 global slots inject into ALL projects. Ideal for persona, tool rules, and preferences that apply universally.
3. **Automatic context injection** — Pinned slots are always in context. No recall, no query, no retrieval step required.
4. **Token budget** — Slots have size limits (1000-3000 chars). Content must be concise and structured.
5. **Single source of truth** — Slots eliminate dual-source drift by removing duplicated rules from agent files.
6. **ADR-gated changes** — Global slots (`persona`, `tool_guidelines`, `user_preferences`) should change only via ADR process.

### The Global Slot Concern

Global slots (`persona`, `tool_guidelines`, `user_preferences`) are scoped to inject into ALL projects within the same agentmemory instance. During v1.2, this was identified as a potential problem: content designed for KodeHold's Director agent would be wrong for Bob's general-purpose coding assistant.

This concern is resolved by architecture separation. Bob now runs its own lightweight MCP memory server — a separate system from agentmemory entirely. KodeHold has sole ownership of this agentmemory instance. All projects within KodeHold share the same director/scribes/architecture patterns, so global slots with KodeHold-specific content are appropriate and safe. The isolation boundary is at the service level, not the slot scope level.

## Decision

### Single-Instance Architecture

**Problem:** During v1.2, dual agentmemory instances were proposed to isolate KodeHold-specific content from Bob's general-purpose projects. This was driven by concern that global slots (`persona`, `tool_guidelines`, `user_preferences`) would leak inappropriate content across projects.

**Resolution:** Bob now has its own lightweight MCP memory server — entirely separate from agentmemory. KodeHold operates a single agentmemory instance. All projects within KodeHold share the same agent architecture and workflow patterns, making global slots with KodeHold-specific content appropriate and safe.

**KodeHold instance configuration:**

| Aspect | Value |
|--------|-------|
| Systemd service | `agentmemory-kodehold.service` |
| Data directory | `~/.agentmemory/kodehold/data/` |
| REST port | 3111 |
| Stream port | 3112 |
| Viewer port | 3113 |
| Start command | `agentmemory --port 3111` |

**opencode.json MCP configuration:**

```json
{
  "mcp": {
    "agentmemory-kodehold": {
      "type": "local",
      "command": ["npx", "-y", "@agentmemory/mcp"],
      "env": { "AGENTMEMORY_URL": "http://localhost:3111" }
    }
  }
}
```

**How agentmemory finds its config:**
Resolution order (from cli.mjs):
1. `AGENTMEMORY_III_CONFIG` env var
2. `process.cwd()/iii-config.yaml`  ← WorkingDirectory
3. `~/.agentmemory/iii-config.yaml`
4. Next to binary

The `--port N` flag relocates the entire port quartet (REST=N, Streams=N+1, Viewer=N+2, Engine=N+46023).

---

### Slot Content Specifications

Each slot gets a defined purpose, content proposal, and strict ownership model.

> **Global slots are KodeHold-scoped:** KodeHold operates a single agentmemory instance. The three global slots (`persona`, `tool_guidelines`, `user_preferences`) inject into all KodeHold projects. Their content is designed for KodeHold's agent architecture and workflow patterns. Since all KodeHold projects share the same team structure, delegation model, and processes, this is appropriate and safe. Bob has its own separate lightweight MCP memory server and does not share KodeHold's agentmemory instance.

---

### Global Slot 1: `persona` (1000 chars, pinned)

**Purpose:** Defines agent identity — role, tone, behavioural guidelines. Injected into ALL projects, so it must be generic (no project-specific content).

**Proposed content:**

```
You are an agentic coding assistant. Your core traits:

ROLE: Engineer, architect, and documenter who ships working software. You own outcomes, not just tasks.

TONE: Direct, technical, and concise. Prefer precise language over fluff. Use "we" when reasoning together with the user. No emojis unless the user uses them first.

BEHAVIOUR:
- Read before you write. Understand existing code before modifying.
- Prefer small, focused changes over sweeping rewrites.
- When stuck, state what you know and what you need — then ask.
- Never hallucinate file paths, API endpoints, or package names.
- Test your assumptions. Run the code before declaring victory.
```

---

### Global Slot 2: `tool_guidelines` (1500 chars, pinned)

**Purpose:** Rules for tool selection and sequencing. Injected into ALL projects, covers universal tool-ordering principles.

**Proposed content:**

```
Tool ordering rules:

1. READ BEFORE WRITE — Always read files before editing them. Use Glob+Grep to find files, Read to understand them, then Edit/Write.

2. PREFER SPECIALIZED OVER GENERIC — Use the most specific tool for the job:
   - Find files: Glob (not bash find)
   - Search content: Grep (not bash grep)
   - Read: Read tool (not bash cat)
   - Edit sections: Edit (not sed)
   - Write new files: Write (not bash heredoc)

3. BATCH INDEPENDENT READS — Read multiple files in parallel. Never read one file at a time.

4. AVOID BASH FOR TEXT — Do NOT use bash for grep/sed/cat/head/tail/find operations. Use dedicated tools.

5. SIZE MATTERS — For files over 500 lines, read in sections (offset+limit). Read the first 50 lines first to understand structure.

6. VERIFY CHANGES — After an edit, read the modified region to confirm the change is correct.

7. COMMIT LAST — Only commit when explicitly asked. Before commit: check git status, diff, and recent log.

8. NEVER git clean -fd without explicit user confirmation.

9. NEVER force-push, use -i interactive, or amend without explicit request.

> **Note:** Rules 8-9 are universal safety rules, not just KodeHold policies. They protect against destructive operations (`git clean -fd`, force-push, interactive rebase) in ANY project. They belong in a global slot because their applicability is universal.

```

---

### Global Slot 3: `user_preferences` (2000 chars, pinned)

**Purpose:** Language, coding style, naming conventions, and project-level preferences. Generic enough to apply across projects, specific enough to be useful.

**Proposed content:**

```
LANGUAGE: English for all code, comments, commit messages, and agent communication.
(When KODEHOLD_LIGHT=1: English only — token optimization.)
(Danish may be spoken for local context — respond in English.)

CODING STYLE:
- Python: Follow PEP 8. Use type hints. Prefer explicit imports over wildcard.
- JavaScript/TypeScript: Use ES6+ syntax. Prefer const over let. Use async/await over callbacks.
- Shell: Use #!/usr/bin/env bash. Set -euo pipefail. Prefer [[ ]] over [ ].
- Rust: Follow rustfmt. Use clippy. Document public APIs.
- Go: Follow gofmt. Use golangci-lint.

NAMING:
- Files: kebab-case for config/scripts, snake_case for Python modules, PascalCase for React components.
- Git branches: kebab-case with type prefix: feat/, fix/, docs/, refactor/.
- Git commits: Conventional Commits: type(scope): description. E.g., feat(auth): add OAuth2 login.

PROJECT STRUCTURE:
- Source in src/ or project root.
- Tests in test/ or alongside source (__test__/ or .test.ts).
- Docs in docs/.
- Config in .config/ or root.

PREFERENCES:
- Prefer simple, readable code over clever/optimized.
- Add comments for WHY, not WHAT. The code says what it does.
- Error messages should tell you what happened AND what to do about it.
```

---

### Project Slot 4: `guidance` (1500 chars, pinned)

**Purpose:** Next-session directives / handoff guidance. Content populated at session end becomes the guidance for the NEXT session start. Provides focus, warns about known issues, sets priorities.

**Proposed initial content (template — populated at session end, consumed at next session start):**

```
GUIDANCE FOR NEXT SESSION — <relative date, e.g., "next session">

Focus area: <current goal or milestone>
State: <current project state e.g., ACTIVE phase>
Key constraint: <current limiting factor, e.g., "ctx limit approaching" or "awaiting review">

Watch out for:
- <known issue 1>
- <known issue 2>

Previous session ended with: <last checkpoint summary>

Priorities:
1. <P1>
2. <P2>
3. <P3>

Avoid: <distractions, forbidden actions>
```

---

### Project Slot 5: `pending_items` (2000 chars, pinned)

**Current state:** Auto-populated by agentmemory's `mem::slot-reflect` mechanism.

**Agentmemory behavior:**
- Triggered by `event::session::stopped` when `AGENTMEMORY_REFLECT=true`
- Scans recent observations for "todo" in title or narrative
- Appends new TODO entries to the slot (deduplicated)
- Truncates from the beginning if content exceeds `sizeLimit`

**Auto-populated format:**
```
- Observation title 1
- Observation title 2
```

**Management rules:**
- **We do NOT write to this slot.** agentmemory owns it.
- TODO entries are auto-detected from session observations.
- Director may read this slot at session start to discover pending work.
- Oldest entries are auto-trimmed when sizeLimit reached.

---

### Project Slot 6: `project_context` (3000 chars, pinned)

**Current state:** Auto-populated by agentmemory's `mem::slot-reflect` mechanism.

**Agentmemory behavior:**
- Triggered by `event::session::stopped` when `AGENTMEMORY_REFLECT=true`
- Collects file paths from observations in the session
- Appends new file paths to the slot (deduplicated)
- Adds header "Files touched in recent sessions:" if slot is empty
- Truncates from the beginning if content exceeds `sizeLimit`

**Auto-populated format:**
```
Files touched in recent sessions:
- src/foo.ts
- .opencode/agents/director.md
```

**What we moved:**
The curated reference content (State, Architecture, Commands, Conventions, Dependencies, Access) that was previously in this slot has been moved to:
- `AGENTS.md` — Quick Reference section (commands, conventions, architecture)
- `director.md` — Session Lifecycle (state info)
- `.kodehold-state` — lifecycle state (already there)

**Management rules:**
- **We do NOT write to this slot.** agentmemory owns it.
- File paths provide session awareness — what was touched recently.
- If the slot gets too noisy, Scribes may clean up via `memory_slot_replace`, but this is NOT routine maintenance.

---

### Project Slot 7: `self_notes` (1500 chars, unpinned)

**Purpose:** Free-form hypothesis tracking. Opt-in read — only loaded when explicitly needed. Append-only.

**Proposed content:** (initially empty — filled ad-hoc)

```
# Self Notes
<!-- Appended by agents during session. Cleared when consolidated. -->

- <note about hypothesis, dead end, or thing to revisit>
- <cross-reference to agentmemory memory_id if consolidated>
```

**Management rules:**
- Append only (use `memory_slot_append`).
- Any agent can write, any agent can read (by unpinning).
- Director may delegate Scribes to consolidate long notes into agentmemory and clear the slot.

---

### Project Slot 8: `session_patterns` (1500 chars, unpinned)

**Current state:** Auto-populated by agentmemory's `mem::slot-reflect` mechanism.

**Agentmemory behavior:**
- Triggered by `event::session::stopped` when `AGENTMEMORY_REFLECT=true`
- Scans recent observations for errors and commands
- **Overwrites** entire slot content each session (not append)

**Auto-populated format:**
```
last reflection: 2026-06-06T04:00:01.552Z
- commands: 1 in last 12 observations
- errors: 1 in last 12 observations
```

**Management rules:**
- **We do NOT write to this slot.** agentmemory owns it entirely.
- The content is replaced every session — no accumulation.
- Read this slot to check session health (error frequency, command activity).

---

### Summary of Changes

| Slot | Current | Management | Token Delta |
|------|---------|------------|-------------|
| `persona` | 450 chars identity | Curated by KodeHold | +450 |
| `tool_guidelines` | ~1060 chars rules | Curated by KodeHold | +1060 |
| `user_preferences` | ~1000 chars preferences | Curated by KodeHold | +1000 |
| `guidance` | ~400 chars template | Curated by KodeHold (Scribes writes) | +400 |
| `pending_items` | Auto-populated by agentmemory | Agentmemory owns (TODO append) | -2000 |
| `project_context` | Auto-populated by agentmemory | Agentmemory owns (file path append) | -1500 |
| `self_notes` | ~100 chars template | Curated by KodeHold (append-only) | +100 |
| `session_patterns` | Auto-populated by agentmemory | Agentmemory owns (overwrites) | -500 |

**Slot management breakdown:** 5 slots curated by KodeHold teams. 3 slots auto-populated by agentmemory.
**Total auto-populated content:** ~200-300 chars per session (varies based on session activity).

### What Slots Are NOT For

Slots are NOT a replacement for:

- **Agentmemory memories** — Structured knowledge (lessons, decisions, patterns) belongs in the memory database, not in slots. Slots are for context injection; memories are for retrieval.
- **Crystals** — Completed work compression belongs in crystals, not slots.
- **Design documents** — Design docs remain in `docs/design/`. Slots provide a summary reference.
- **ADRs** — ADRs remain in `docs/adr/`. Slots reference key ADR numbers but do not store ADR content.
- **File paths in curated slots** — File paths belong in agentmemory-auto-populated slots (project_context), not in curated slots. The `project_context` slot is intentionally used by agentmemory for file path tracking.

## Migration Strategy (4 Phases)

**Rationale for order:** Phase 1 (fill empty slots) first because it delivers immediate value with zero data loss risk. Phase 2 (restructure populated slots) next because the messy slots need attention while Phase 1 context is still fresh. Phase 3 (update agent files) followed by Phase 4 (accept auto-population) last because rules must exist in slots BEFORE they can be safely removed from agent files. This prevents a window where rules exist in neither place.

### Phase 1: Fill Empty Slots (zero risk)

**Scope:** Slots 1-4 (persona, tool_guidelines, user_preferences, guidance) and 7-8 (self_notes template, session_patterns template).

**Risk:** None — slots are empty, writing to them has no data loss.

**Steps:**

| # | Action | Who | Tool |
|---|--------|-----|------|
| 1 | Write `persona` slot content | Scribes | `memory_slot_replace(label="persona", content=...)` |
| 2 | Write `tool_guidelines` slot content | Scribes | `memory_slot_replace(label="tool_guidelines", content=...)` |
| 3 | Write `user_preferences` slot content | Scribes | `memory_slot_replace(label="user_preferences", content=...)` |
| 4 | Write `guidance` template | Scribes | `memory_slot_replace(label="guidance", content=...)` |
| 5 | Write `self_notes` template | Scribes | `memory_slot_replace(label="self_notes", content=...)` |
| 6 | Write `session_patterns` template | Scribes | `memory_slot_replace(label="session_patterns", content=...)` |
| 7 | Seed content in `docs/slots/` for source control | Scribes | Write `docs/slots/*.md` files |

**Verification:** Run `memory_slot_list` and confirm all 8 slots have content. Verify injected context in next session.

---

### Phase 2: Restructure Populated Slots (medium risk — data loss possible)

**Scope:** Slots 5-6 (pending_items, project_context).

**Risk:** Medium — current content is noisy but may contain user-generated items. Must capture before clear.

**Steps:**

| # | Action | Who | Tool |
|---|--------|-----|------|
| 1 | Export `pending_items` content to agentmemory | Scribes | `memory_slot_get(label="pending_items")` → `memory_save(content=..., type="fact")` |
| 2 | Clear and reseed `pending_items` with P1/P2/P3 structure | Scribes | `memory_slot_replace(label="pending_items", content=...)` |
| 3 | Extract any salvageable context from `project_context` | Scribes | `memory_slot_get(label="project_context")` → extract key file paths |
| 4 | Restructure `project_context` with sections | Scribes | `memory_slot_replace(label="project_context", content=...)` |
| 5 | Update seed files in `docs/slots/` | Scribes | Write updated content |

**Rollback:** Archived content is in agentmemory. Use `memory_recall(query="pending_items archive")` to retrieve.

---

### Phase 3: Update Agent Files — Remove Migrated Rules

**Scope:** `director.md`, `scribes.md`, `AGENTS.md`, all team agent files under `.opencode/agents/`.

**Risk:** Low — rules are preserved in slots. Only duplicates are removed.

**What to remove from agent files:**

| File | What to Remove | Replace With | Est. Tokens Saved |
|------|---------------|--------------|-------------------|
| `director.md` | Core Protocol (lines 29-35), duplicated team tables, triage tables, delegation tables, commit rules, constraints that are now in `tool_guidelines`/`persona`/`user_preferences` | "See slot: persona, tool_guidelines, user_preferences" | ~800 |
| `AGENTS.md` | Quick Reference delegation table, triage-check table, state/gate tables (lines 8-70) — ALL redundant with director.md | "See director.md (full agent definition)" | ~600 |
| `scribes.md` | Any universal tool-ordering rules, naming conventions, language preferences | "See slots: tool_guidelines, user_preferences" | ~100 |
| Other agent files | Duplicate persona instructions, tool rules, preference defaults | "See slots" reference | ~100 |

**Important:** Do NOT remove unique content — only content that exists verbatim (or near-verbatim) in the new slot content. Each agent file retains its team-specific instructions.

**Steps:**

| # | Action | Who | Tool |
|---|--------|-----|------|
| 1 | Audit director.md — tag lines that duplicate slot content | Architects | Read + annotate |
| 2 | Audit AGENTS.md — the Quick Reference and triage sections duplicate slot content | Architects | Read + annotate |
| 3 | Audit scribes.md — remove universal rules, keep Scribes-specific instructions | Architects | Read + annotate |
| 4 | Edit director.md — remove duplicates, add slot references | Scribes | Edit tool |
| 5 | Edit AGENTS.md — condense to essentials | Scribes | Edit tool |
| 6 | Edit scribes.md — remove universal rules | Scribes | Edit tool |
| 7 | Verify context injection — start test session, confirm slots load | Director | Read slot content |

---

### Phase 4: Let Agentmemory Auto-Populate 3 Slots

**Scope:** Slots 5, 6, 8 (pending_items, project_context, session_patterns).

**Discovery:** During post-implementation verification, we discovered that agentmemory's built-in `mem::slot-reflect` mechanism (triggered by `event::session::stopped` when `AGENTMEMORY_REFLECT=true`) automatically writes to these 3 slots every session end.

**Decision:** Accept agentmemory's auto-population. Stop writing to these 3 slots. Document how to use the auto-populated data.

**Source reference:** `src/functions/slots.ts` (lines 384-475) in the agentmemory repository.

**Changes made:**
- Cleared our curated content from `session_patterns`, `pending_items`, `project_context`
- Moved curated project reference content (State, Architecture, Commands, Conventions, Dependencies, Access) to `AGENTS.md` and `director.md`
- Updated seed files in `docs/slots/` to match agentmemory's output format
- Updated Automation Model to show agentmemory as writer

## Automation Model

Clear ownership per slot — every slot has a single writer to prevent conflicts.

| Slot | Read | Write | Cadence | Tool | Backup |
|------|------|-------|---------|------|--------|
| `persona` | All agents | Architects (via ADR) | Rarely (ADR-gated) | `memory_slot_replace` | `docs/slots/persona.md` |
| `tool_guidelines` | All agents | Architects (via ADR) | Rarely (ADR-gated) | `memory_slot_replace` | `docs/slots/tool_guidelines.md` |
| `user_preferences` | All agents | User (via Architects ADR) | When preferences change | `memory_slot_replace` | `docs/slots/user_preferences.md` |
| `guidance` | All agents | Scribes | Every session end | `memory_slot_replace` | (ephemeral — not backed up) |
| `pending_items` | All agents | **agentmemory** (auto) | Every session end | `mem::slot-reflect` (append) | N/A — auto-populated |
| `project_context` | All agents | **agentmemory** (auto) | Every session end | `mem::slot-reflect` (append) | N/A — auto-populated |
| `self_notes` | Any agent (opt-in) | Any agent (append) | Ad-hoc | `memory_slot_append` | Consolidated to agentmemory |
| `session_patterns` | All agents | **agentmemory** (auto) | Every session end | `mem::slot-reflect` (overwrite) | N/A — auto-populated |

### Write Model: Replace vs. Append

> **Note on permissions:** Agentmemory does not support per-agent read/write permissions on slots. The ownership model defined above (single writer per slot, enforced by team convention) IS the permission system. Scribes, as the designated writer for most slots, is responsible for respecting ownership boundaries.

| Operation | Used For | Rationale |
|-----------|----------|-----------|
| `memory_slot_replace` | persona, tool_guidelines, user_preferences, guidance | Structured slots — content is curated, not accumulated. Replace avoids the 413 overflow error. |
| `memory_slot_append` | self_notes | Accumulative slot — entries grow over time. Risk of 413; mitigated by periodic rotation/consolidation by Scribes. |
| `mem::slot-reflect` (auto) | pending_items, project_context, session_patterns | Auto-populated by agentmemory at session end. We do NOT write to these slots manually. |

### Source Control Backup

All slot content (except ephemeral `guidance`) is backed up in `docs/slots/<slot-name>.md`. This provides:

- Git history of content changes
- Recovery if agentmemory is reset
- PR review of slot modifications
- Onboarding reference for new team members

**File structure:**
```
docs/slots/
  persona.md              — matches persona slot content
  tool_guidelines.md      — matches tool_guidelines slot content
  user_preferences.md     — matches user_preferences slot content
  pending_items.md        — documents agentmemory's auto-populated format (we don't write to this slot)
  project_context.md      — documents agentmemory's auto-populated format (we don't write to this slot)
  self_notes.md           — placeholder template
  session_patterns.md     — documents agentmemory's auto-populated format (we don't write to this slot)
```

**Note:** `guidance` is NOT backed up — it is ephemeral per session.
The 3 agentmemory-auto-populated slots (pending_items, project_context, session_patterns) have seed files that document their expected format, but content is managed by agentmemory, not by us.

**Sync process:** Scribes writes slot content and the corresponding `docs/slots/<name>.md` file in the same task. This keeps them in sync.

## Risks and Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Slot overflow (413 error on append)** — `memory_slot_append` returns 413 when content exceeds `sizeLimit` | Medium | High | Use `memory_slot_replace` for structured slots. For append-only slots (self_notes, session_patterns), Scribes monitors size and consolidates to agentmemory at 80% capacity. |
| 2 | **Stale global content** — persona, tool_guidelines, user_preferences drift out of date because changes require ADR process | Low | Medium | ADR-gated changes are appropriate for persona/tool/user-prefs — they should change rarely. Annual review by Architects. |
| 3 | **Global slot pollution** — project-specific content leaks into global slots (persona, tool_guidelines, user_preferences) | Low | High | Strict ADR review of global slot changes. Global slots must only contain content applicable to ALL projects. Reviewers gate checks this. |
| 4 | **Context injection overhead** — if slots are verbose, they waste injected context tokens | Medium | Low | Size limits (1000-3000 chars) are hard caps. Target actual content at 50-70% of limit. The combined injection of 8 pinned slots at target size is ~5000 chars — acceptable for context. |
| 5 | **Content loss on agentmemory reset** — slots are database-backed; a reset or DB corruption loses all content | Low | High | `docs/slots/` directory in source control provides seed content. Recovery: Scribes runs `memory_slot_replace` from seed files. |
| 6 | **Multi-writer conflicts** — two agents write the same slot simultaneously | Low | Medium | Clear ownership per slot (Automation Model table). Only Scribes writes to most slots. Director writes to pending_items. No concurrent writes expected. |
| 7 | **Slots not available during agent init** — `memory_slot_get` fails if agentmemory is down | Low | Low | Slot content is auto-injected at context build time by the OpenCode platform, not fetched on demand. If agentmemory is down, the entire system is down — not just slots. |
| 8 | **Seed files in `docs/slots/` drift from actual slot content** — source control backup becomes stale | Medium | Low | Scribes writes both slot AND seed file in the same operation. CI check: verify `docs/slots/` matches slot content on each commit. |
| 9 | **Slot corruption / inconsistent content** — a failed write or multi-agent race corrupts slot content mid-session | Low | High | Scribes validates content structure after every write. Session checkpoint includes a slot integrity check (verify pinned slots have expected structure). Recovery: re-seed from `docs/slots/` backup. |
| 10 | **slot-reflect overwrites curated content** — If we accidentally write curated content to a slot that agentmemory auto-populates, it will be overwritten at next session end | Low | Medium | Clear documentation: we do NOT write to pending_items, project_context, or session_patterns. Automation Model table shows agentmemory as writer. |

### Risk Scoring

| Score Range | Count | Action |
|-------------|-------|--------|
| High x Medium | 1 (slot overflow) | Mitigation in place (replace model) |
| Medium x Low | 1 (seed file drift) | Monitoring + CI check |
| Low x High | 2 (content loss, slot corruption) | Mitigation in place (docs/slots/ + integrity checks) |
| Low x Medium | 2 (pollution, slot-reflect overwrite) | Documentation + Automation Model |
| Low x Low | 2 (overhead, conflicts) | Accept |

## Agentmemory Auto-Population (slot-reflect)

### What is slot-reflect?

`mem::slot-reflect` is a built-in agentmemory function (registered in `src/functions/slots.ts`) that runs automatically at the end of every session. It is enabled via `AGENTMEMORY_REFLECT=true` in the environment configuration.

**Trigger:** `event::session::stopped` → calls `mem::slot-reflect` (see `src/triggers/events.ts` lines 49-64).

### What it tracks

| Data Source | Slot Written | Operation |
|-------------|-------------|-----------|
| Observations with "todo" in title/narrative | `pending_items` | Append |
| Observations with type=error | `session_patterns` | Overwrite |
| Observations with type=command_run | `session_patterns` | Overwrite |
| Observation file paths | `project_context` | Append |

### Configuration

Set in `~/.agentmemory/.env`:
```
AGENTMEMORY_SLOTS=true
AGENTMEMORY_REFLECT=true
```

### Why we accept it

Initial ADR-0043 implementation tried to manually manage all 8 slots. During Phase 2 verification, we discovered that agentmemory's `slot-reflect` was overwriting 3 slots. Rather than disabling it or patching agentmemory source code, we accept the behavior because:

1. **Less maintenance** — Agentmemory handles reflection automatically. No Scribes work needed.
2. **Valuable data** — Error frequency, command counts, file path tracking, and TODO detection are genuinely useful.
3. **Source integrity** — Patching agentmemory would create a fork risk. Accepting upstream behavior is lower friction.

## How to Use Auto-Populated Slot Content

The 3 slots auto-populated by agentmemory provide useful operational data. Here is how each team can use them:

### session_patterns — Health Monitor

**Read by:** Director, Scribes

**Content:** `last reflection: <timestamp>` + `- commands: N` + `- errors: N`

**Use cases:**

| Pattern | What it means | Action |
|---------|---------------|--------|
| `errors > 3` in a session | High error rate — something is broken | Director triggers FLS triage |
| `commands: 0` across 2+ sessions | Agent idle — no work being done | Director investigates session health |
| Errors increasing over N sessions | Regression trend | Scribes stores metric, Architects investigates |
| Commands increasing + errors decreasing | Healthy development pattern | No action — positive signal |

**Example — Director session-start check:**
```
Read session_patterns at start of new session.
If errors > threshold → warn user: "Previous session had N errors. Investigate?"
```

### pending_items — TODO Discovery

**Read by:** Director (at session start)

**Content:** Flat list of TODO-related observation titles

**Use cases:**

| Pattern | What it means | Action |
|---------|---------------|--------|
| New TODO items appear | Agent mentioned work items in previous session | Director presents to user: "These TODOs were detected from last session" |
| Same TODO persists across sessions | Item not being actioned | Director flags as potential priority |

**Note:** The slot is a flat list with no P1/P2/P3 structure. For prioritization, use `guidance` slot (written by Scribes at session end) or `pending_items` seed file as reference.

### project_context — Session Awareness

**Read by:** All agents (auto-injected — pinned)

**Content:** `Files touched in recent sessions:` + file paths

**Use cases:**

| Pattern | What it means | Action |
|---------|---------------|--------|
| Files from current project | Recent work context | Agent has immediate awareness of what files were modified |
| Files from multiple areas | Broad session activity | No action — informational |
| Slot is empty or stale | No recent sessions with file activity | Normal — new project or long break |

### Synchronization with guidance slot

The `guidance` slot (curated by Scribes) complements the 3 auto-populated slots:

```
guidance slot:    "Focus on auth module. Known issue: login timeout."
                  (Explicit direction — curated by Scribes)

session_patterns: "commands: 5, errors: 0"
                  (Operational health — auto-populated)

project_context:  "src/auth/login.ts, src/auth/session.ts"
                  (File awareness — auto-populated)

pending_items:    "- Fix login timeout"
                  (TODO detection — auto-populated)
```

### Operational workflow

1. **At session start** — Director reads all 3 auto-populated slots + `guidance`
   - Check `session_patterns` for health signals
   - Check `pending_items` for discovered TODOs
   - Check `project_context` for recent file context
   - Present relevant info to user

2. **During session** — Agents benefit from auto-injected `project_context` (file awareness)

3. **At session end** — agentmemory's `slot-reflect` runs automatically
   - Updates `session_patterns` with reflection data
   - Appends TODO items to `pending_items`
   - Appends file paths to `project_context`
   - No manual action needed

## Consequences

### Positive

1. **Reduced agent file size.** An estimated 1,600+ tokens of duplicate boilerplate removed from `director.md`, `AGENTS.md`, `scribes.md`, and other agent files. Each file is leaner, more focused on its team-specific purpose.

2. **Cross-project sharing.** Global slots (`persona`, `tool_guidelines`, `user_preferences`) inject into ALL projects. A single update propagates everywhere instantly — no need to edit N agent files.

3. **Persistent rules.** Slot content survives agent restarts, session resets, and model swaps. Rules are not lost when context is pruned.

4. **Automatic context injection.** Pinned slots are always in context. No recall step, no query, no "load context" ritual. The rules are simply there.

5. **Single source of truth.** Each rule exists in exactly one slot. Changes propagate automatically. No dual-source drift.

6. **Git-tracked seed files.** Slot content is version-controlled in `docs/slots/`, enabling PR review of changes and recovery from agentmemory reset.

7. **Clear ownership.** 5 slots curated by KodeHold teams, 3 slots auto-populated by agentmemory. No ambiguity.

8. **Zero-maintenance reflection.** agentmemory automatically tracks errors, commands, file paths, and TODOs — no Scribes work needed.

### Negative

1. **Single instance means global blast radius within KodeHold.** A mistake in `persona` or `tool_guidelines` affects all KodeHold projects. Mitigation: ADR-gated changes with Architect and Reviewer oversight. Bob's separate memory server is unaffected by KodeHold slot changes.

2. **ADR process overhead for global slot changes.** Updating `persona`, `tool_guidelines`, or `user_preferences` requires an ADR, which may feel heavy for small tweaks. Mitigation: batch small changes into periodic ADRs.

3. **Session context injection cost.** 5 pinned slots at ~5000 chars total adds ~1250 tokens to every context. Acceptable — the content is high-value and replaces boilerplate that would otherwise be in agent files.

4. **Auto-populated slots are flat structures.** pending_items is a flat list without P1/P2/P3. session_patterns is minimal (commands+errors only). For structured data, use guidance slot.

### Neutral

1. **ADR process for persona/tool_guidelines/user_preferences.** Global slot changes follow the same ADR lifecycle as infrastructure changes. This is appropriate for foundational agent configuration.

2. **Seeded content in `docs/slots/` is a new maintenance artifact.** A small cost for the benefit of git-tracked recovery.

3. **Scribes workload decreases slightly** — agentmemory now handles 3 slots automatically.

## Alternatives Considered

### Option A: Status Quo (Rejected)

Continue with hardcoded rules in agent files, leaving slots empty or minimally used.

**Why rejected:** Token waste (1,600+ duplicates), drift risk, cross-session amnesia, and no cross-project sharing. The slot infrastructure exists and is free — leaving it unused is suboptimal.

### Option B: Hybrid (Selected)

Migrate universal rules to slots. Keep team-specific rules in agent files. Use ADR-gated process for global slots.

**Why selected:** Best risk/reward balance. Slot integration provides immediate token savings and cross-project sharing. Team-specific rules remain in agent files where they belong.

### Option C: Full Migration

Move ALL rules (including team-specific) into slots. Agent files become pure delegation references.

**Why rejected:** Slots are size-limited. Team-specific rules (e.g., Scribes' 14 responsibilities, Engineers' testing protocol) would exceed slot limits. Agent files are the correct home for team-specific detail.

### Option D: Use Memories Instead of Slots

Store rules as agentmemory entries and load them via `memory_recall` at session start.

**Why rejected:** Memory recall requires an explicit query step and may return stale results. Slots are auto-injected — zero retrieval overhead. The "always in context" property is essential for persona, tool rules, and preferences.

### Option E: Single Instance with Global Slots (Deferred → Selected)

Run a single agentmemory instance with global-scope slots (`persona`, `tool_guidelines`, `user_preferences`) injecting into all KodeHold projects.

**Initial assessment (v1.2):** Rejected — global scope would cause cross-project pollution between KodeHold and Bob.

**Current assessment (v2.0):** Selected. Bob now has its own lightweight MCP memory server, entirely separate from agentmemory. KodeHold's single-instance global slots are safe: all projects within KodeHold share the same agent architecture and workflows. The cross-project pollution concern is resolved at the service boundary, not the slot scope level.

## Implementation Plan

### Phase 1 — Fill Empty Slots

| # | Action | Who | Est. Effort |
|---|--------|-----|-------------|
| 1 | Write persona slot (650 chars) + seed file | Scribes | 10 min |
| 2 | Write tool_guidelines slot (1100 chars) + seed file | Scribes | 15 min |
| 3 | Write user_preferences slot (1400 chars) + seed file | Scribes | 15 min |
| 4 | Write guidance template + seed file | Scribes | 5 min |
| 5 | Write self_notes template + seed file | Scribes | 5 min |
| 6 | Write session_patterns template + seed file | Scribes | 10 min |
| 7 | Verify: `memory_slot_list`, test session | Director | 5 min |

### Phase 2 — Restructure Populated Slots

| # | Action | Who | Est. Effort |
|---|--------|-----|-------------|
| 1 | Archive pending_items to agentmemory | Scribes | 5 min |
| 2 | Reseed pending_items with P1/P2/P3 | Scribes | 10 min |
| 3 | Extract salvageable context from project_context | Scribes | 5 min |
| 4 | Restructure project_context | Scribes | 15 min |

### Phase 3 — Update Agent Files

| # | Action | Who | Est. Effort |
|---|--------|-----|-------------|
| 1 | Audit + tag duplications in director.md | Architects | 20 min |
| 2 | Audit + tag duplications in AGENTS.md | Architects | 10 min |
| 3 | Edit director.md — remove duplicates, add slot refs | Scribes | 30 min |
| 4 | Edit AGENTS.md — condense to essentials | Scribes | 15 min |
| 5 | Edit scribes.md — remove universal rules | Scribes | 10 min |
| 6 | Verify context injection | Director | 10 min |

### Phase 4 — Accept Agentmemory Auto-Population

| # | Action | Who | Est. Effort |
|---|--------|-----|-------------|
| 1 | Clear curated content from session_patterns, pending_items, project_context | Scribes | 5 min |
| 2 | Move curated project reference to AGENTS.md / director.md | Scribes | 15 min |
| 3 | Update seed files in docs/slots/ to document auto-populated format | Scribes | 10 min |
| 4 | Update ADR-0043 to document Phase 4 and new ownership model | Scribes | 15 min |

## ADR References

- **ADR-0029** (Agentmemory Migration Strategy) — established the migration pattern from ICM to agentmemory. This ADR extends migration to slot-based context.
- **ADR-0030** (Agentmemory Knowledge Flow) — knowledge retrieval protocol. Slots complement this by providing always-injected context.
- **ADR-0018** (Centralize Documentation Under Scribes) — Scribes owns documentation. This ADR gives Scribes slot management responsibilities.
- **ADR-0007** (Token Optimization Strategy) — per-phase token budgets. Slot integration reduces token waste from duplicate rules.
- **ADR-0036** (Project Slug Convention) — stable project identifiers for project-scoped slots.
- **ADR-0042** (ADR Implementation Phase Board) — established YAML frontmatter phase tracking used in this ADR.

## Open Questions

1. **Should `self_notes` be pinned?** No — it is append-only and intended for opt-in reading. Pinning would waste context on notes that may not be relevant.

2. **Should we add curated project reference info to AGENTS.md?** — Yes, done in Phase 4. The curated reference (State, Architecture, Commands, Conventions, Dependencies, Access) was moved from project_context slot to AGENTS.md and director.md.

3. **What happens when `guidance` is not updated for a session?** The previous session's guidance remains. This is acceptable for the first session of a day but degrades over time. Scribes must update guidance as part of the session-end protocol.

4. **Should we create a `docs/slots/README.md`?** No — the design doc at `docs/design/README.md` covers architecture. Slot seed files are self-documenting.
