---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0030: Agentmemory Knowledge Flow

## Status

Deprecated

**Phase:** Phase 2 (Infrastructure Migration) — replaces ADR-0027 and the `icm-knowledge-flow` skill.

## Context

### The Problem

The current `icm-knowledge-flow` skill (`.opencode/skills/icm-knowledge-flow/SKILL.md`) defines an 8-step protocol that all 6 team agents execute on every delegation. The skill is referenced by 8 agent definitions and the Director itself. As established in ADR-0027, the protocol has 3 invocation modes (Pre-task, Post-task, Full) but uses **ICM MCP tools** for all storage and retrieval.

Per ADR-0029 (Phase 2 — Infrastructure), all ICM tool calls must be replaced with agentmemory equivalents. The 3-mode knowledge flow is the most widely-referenced skill in the KodeHold codebase — referenced by every agent definition, the Director, and the second-opinion agent. Migrating it to agentmemory is the single largest file-impact item in Phase 2.

### Key Forces

1. **Same modes, new tools.** The 3-mode structure (Pre-task/Post-task/Full) is well-tested and must be preserved. Only the underlying tool calls change.
2. **All 8 agent definitions must update simultaneously.** A partial update creates a split state where some agents use agentmemory and others use ICM.
3. **Skill directory must be renamed.** `.opencode/skills/icm-knowledge-flow/` → `.opencode/skills/agentmemory-knowledge-flow/`
4. **ADR-0027 must be deprecated.** The protocol structure lives on, but the ICM tools are gone.
5. **Step numbering preserved.** The existing protocol uses steps 1-2 for pre-task (search), step 3 for execution (not part of the skill), and steps 4-8 for post-task (reflect, consolidate, store, refine).

## Decision

### Replace ICM Knowledge Flow with Agentmemory Knowledge Flow

1. **Rename skill directory:** `.opencode/skills/icm-knowledge-flow/` → `.opencode/skills/agentmemory-knowledge-flow/`
2. **Replace all ICM tool calls** with agentmemory equivalents in the 3-mode protocol
3. **Preserve the 3-mode structure and step numbering** (Pre-task steps 1-2, Post-task steps 4-8, Full = both)
4. **Update all 8 agent definitions** to reference "Agentmemory Knowledge Flow"
5. **Deprecate ADR-0027**

### Tool Mapping (from ADR-0029)

| ICM Tool | Agentmemory Equivalent |
|----------|----------------------|
| `icm_memoir_search(memoir="kodehold-learnings")` | `agentmemory_memory_lesson_recall(query=...)` |
| `icm_memoir_search(memoir="kodehold-teams")` | `agentmemory_memory_lesson_recall(query=...)` |
| `icm_memory_health(topic=...)` | `agentmemory_memory_diagnose()` |
| `icm_memory_consolidate(topic=..., summary=...)` | `agentmemory_memory_consolidate(tier="episodic")` |
| `icm_memory_store(topic=..., content=..., importance=...)` | `agentmemory_memory_save(content=..., type=..., project=...)` |
| `icm_memoir_refine(memoir=..., name=..., definition=...)` | `agentmemory_memory_lesson_save(content=..., tags=...)` |

### Agentmemory Knowledge Flow Protocol

#### Pre-task Mode (Steps 1-2)

| Step | Action | Tool Call |
|------|--------|-----------|
| 1 | Search shared learnings | `agentmemory_memory_lesson_recall(query="[keywords]", limit=5)` |
| 2 | Search team learnings | `agentmemory_memory_lesson_recall(query="[keywords]", limit=5)` |

#### Post-task Mode (Steps 4-8)

| Step | Action | Tool Call / Procedure |
|------|--------|----------------------|
| 4 | Reflect | Mental reflection — no tool call |
| 5 | Consolidate check | `agentmemory_memory_diagnose()` → if needed: `agentmemory_memory_consolidate(tier="episodic")` |
| 6 | Store shared learnings | `agentmemory_memory_save(content="...", type="pattern", project="kodehold", concepts="learnings")` |
| 7 | Store team learnings | `agentmemory_memory_save(content="...", type="pattern", project="<project>", concepts="<team>-learnings")` |
| 8 | Refine concepts | `agentmemory_memory_lesson_save(content="...", tags=["recurring-pattern"])` |

#### Full Mode (Steps 1-2, 4-8)
Both pre-task and post-task in a single delegation. Rare.

### Mode Selection

| Team | Default Mode |
|------|-------------|
| Engineers | Pre-task → (post-task via Director follow-up) |
| Testers | Pre-task → (post-task via Director follow-up) |
| Reviewers | Pre-task → (post-task via Director follow-up) |
| FLS | Pre-task → (post-task via Director follow-up) |
| Architects | Pre-task → (post-task via Director follow-up) |
| Scribes | Post-task only |
| (Rare/Explicit) | Full |

### Protocol Notes
- Steps 1-2 are SEARCH ONLY — no writes
- Step 4 is mental reflection — no tool calls
- Steps 5-8 are WRITE operations
- Step 3 (Execute task) is NOT part of knowledge flow
- Scribes never runs steps 1-2 (pre-task search)

## Consequences

### Positive
1. **No ICM dependency.** All knowledge flow operations use native agentmemory MCP tools.
2. **Better lesson management.** `memory_lesson_recall/save` provides structured lessons with confidence scoring.
3. **Same modes, same workflow.** Teams don't need to learn a new process.
4. **Consolidation automated.** `memory_consolidate(tier="episodic")` replaces manual consolidation.

### Negative
1. **All 8 agent files must change simultaneously.** Partial update creates split state.
2. **Teams must learn new tool names.** Mitigation: SKILL.md provides exact copy-paste-able tool calls.
3. **No topic-scoped health check.** `memory_diagnose()` is system-wide. Mitigation: auto-consolidation handles this.

## ADR References
- **ADR-0027** (ICM Knowledge Flow Invocation Modes) — **Deprecated** by this ADR
- **ADR-0028** (Agentmemory Project Detection) — established project naming
- **ADR-0029** (ICM → Agentmemory Migration Strategy) — Phase 2 implementation
- **ADR-0018** (Scribes Centralization) — Scribes Post-task-only preserved

### Source Files Referenced
- `.opencode/skills/icm-knowledge-flow/SKILL.md` — renamed to agentmemory-knowledge-flow
- `.opencode/agents/*.md` — all 8 agent definitions
- `docs/adr/ADR-0027-icm-knowledge-flow-invocation-modes.md` — to be deprecated
