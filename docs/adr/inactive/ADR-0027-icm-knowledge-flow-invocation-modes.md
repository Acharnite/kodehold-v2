---
status: Superseded
superseded-by: ADR-0030
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0027: ICM Knowledge Flow Invocation Modes

## Status

Deprecated

Superseded by ADR-0030 (Agentmemory Knowledge Flow).

**Replaced by:** [ADR-0030](ADR-0030-agentmemory-knowledge-flow.md)

## Context

The ICM Knowledge Flow skill (defined in `.opencode/skills/icm-knowledge-flow/SKILL.md`) prescribes an 8-step protocol that every team agent executes on every delegation. The 8 steps are:

| Step | Action | Timing |
|------|--------|--------|
| 1 | Search shared learnings (`kodehold-learnings` memoir) | Pre-task |
| 2 | Search team learnings (`kodehold-learnings` memoir) | Pre-task |
| 3 | Execute task (team's standard workflow) | — |
| 4 | Reflect (identify what was learned) | Post-task |
| 5 | Consolidate check (if topic >7 entries, consolidate) | Post-task |
| 6 | Store shared learnings (`kodehold-learnings` memoir) | Post-task |
| 7 | Store team learnings (`kodehold-learnings` memoir) | Post-task |
| 8 | Distill/refine concepts in memoirs | Post-task |

**Problem:** All 6 agent files instruct teams to "execute each step" without distinguishing which steps apply at which point in the delegation lifecycle. This creates three concrete issues:

1. **Wasted tokens.** Scribes is always invoked post-task (per ADR-0018). It runs steps 1-2 (search) immediately before step 6-7 (store) — within the same delegation call — making the search redundant since no task execution separates them from the store.

2. **Semantic confusion.** The canonical SKILL.md correctly notes "Steps 5-8 run AFTER the task (step 3)" but this is a comment, not a structural enforcement. Agent files repeat "execute each step" without lifecycle context.

3. **No mode differentiation.** A team that only needs to retrieve context (pre-task) and a team that only needs to store results (post-task) are both told to run all 8 steps. The only exception is Scribes, which the Director sometimes invokes with an explicit "skip search" instruction — an ad-hoc workaround, not a design.

Key forces:
- Steps 1-2 (search) are valuable before task execution — they load relevant past learnings into context
- Steps 4-8 (reflect/store/refine) are valuable after task execution — they capture new insights
- Running search immediately before store (when no task separates them) is wasteful
- Scribes only stores — it never executes a task between search and store
- A "Full" mode is needed for rare cases where a single delegation both retrieves and stores in one call

## Decision

### Three Invocation Modes

Define three invocation modes for the ICM Knowledge Flow skill. Each mode is a named subset of the 8 steps:

| Mode | Steps | When to Use |
|------|-------|-------------|
| **Pre-task** | 1, 2 | Before the team executes its core task. Loads relevant past learnings into context. |
| **Post-task** | 4, 5, 6, 7, 8 | After the team completes its core task. Reflects on what was learned and stores it. |
| **Full** | 1, 2, 3, 4, 5, 6, 7, 8 | Rare — when a single delegation both searches context and stores results with no separate call. |

Step 3 (Execute task) is **not** a knowledge flow step — it is the team's own workflow, invoked between Pre-task and Post-task by the Director's delegation sequence.

### Team → Mode Mapping

| Team | Pre-task Mode | Post-task Mode | Notes |
|------|:---:|:---:|-------|
| **Architects** | Steps 1-2 | Steps 4-8 | Design work benefits from searching past patterns |
| **Engineers** | Steps 1-2 | Steps 4-8 | Code implementation benefits from past learnings |
| **Testers** | Steps 1-2 | Steps 4-8 | Test strategy informed by prior findings |
| **Reviewers** | Steps 1-2 | Steps 4-8 | Review context loaded from past decisions |
| **FLS** | Steps 1-2 | Steps 4-8 | Support benefits from searching resolved issues |
| **Scribes** | — | Steps 4-8 only | Scribes is always invoked post-task; no search needed |

**Exception — Full mode:** When the Director invokes a team with a combined "search and store" intent (rare), the agent runs steps 1-2 before its task and steps 4-8 after, in a single delegation call. This is explicitly signaled in the Task tool prompt, e.g.:

```
Execute ICM Knowledge Flow in FULL mode: search learnings before task, store learnings after.
```

If no mode is specified, agents default to **Pre-task** (steps 1-2 only). The Director is responsible for invoking Post-task as a separate delegation or including it in the delegation prompt.

### SKILL.md Restructuring

The SKILL.md must be restructured to present three named modes instead of one linear sequence:

```
# ICM Knowledge Flow

## Modes

### Pre-task (steps 1-2)
Run BEFORE the team executes its core task.
Loads relevant past learnings into context.

1. Search shared learnings — search `kodehold-learnings` memoir for relevant patterns
2. Search team learnings — search `kodehold-learnings` memoir for team-specific patterns

### Post-task (steps 4-8)
Run AFTER the team completes its core task.
Reflects on what was learned and stores it.

4. Reflect — after execution, identify what was learned
5. Consolidate check — if target topic has >7 entries, consolidate first (ICM warns at >7)
6. Store shared learnings — save findings in `kodehold-learnings`
7. Store team learnings — save team-specific findings in `kodehold-learnings`
8. Distill/refine concepts — add/refine concepts in relevant memoirs

### Full (steps 1-2, then 4-8)
Rare — run Pre-task before the task, Post-task after.
Used when a single delegation both searches and stores.

## Usage

Each team agent references this skill with a mode parameter.
The Director specifies the mode in the Task tool prompt.
Default mode (if unspecified): Pre-task.
```

Step 3 (Execute task) is deliberately removed from the skill — it is the team's own workflow, not a knowledge flow concern.

### Agent File Changes

Each of the 6 agent files (`.opencode/agents/<team>.md`) must be updated:

1. **Replace** the generic "execute each step" instruction with mode-aware instructions
2. **Default to Pre-task** in the ICM Knowledge Flow section
3. **Document Post-task** as a separate invocation the Director triggers after task completion
4. **Remove step 3** from the knowledge flow — it is the agent's core workflow, not part of ICM

Example updated ICM section for Engineers:

```markdown
## ICM Knowledge Flow

Load the skill at `.opencode/skills/icm-knowledge-flow/SKILL.md` and execute with these parameters:

- **Mode**: Pre-task (default) — run steps 1-2 only
- Team: `engineers`
- Shared learnings query: `"code pattern OR implementation OR refactor"`
- Team memoir: `kodehold-teams`, query: `"engineering OR implementation OR code"`
```

Scribes' agent file must explicitly note it uses **Post-task only** mode and skips search steps.

### Director's Responsibility

The Director is responsible for:

1. **Specifying mode** in each Task tool delegation prompt when the default (Pre-task) is insufficient
2. **Invoking Post-task** as a follow-up delegation when results need to be stored
3. **Using Full mode** only when explicitly combining search and store in one delegation

The Director's delegation table (in `AGENTS.md`) should document the default mode per team.

### Inherited from ADR-0009

This ADR extends ADR-0009 (ICM MCP Integration) by:
- Defining *when* MCP tools are called in the delegation lifecycle, not just *which* tools
- Making the Post-task storage pattern (steps 4-8) a first-class mode instead of an implicit "run everything"
- Aligning with ADR-0018 (Scribes centralization) — Scribes is always Post-task because it never executes a task between search and store

### Team-Specific Parameters

Each team loads the same skill but parameterizes it differently:

| Team | Shared Learnings Query | Team Memoir | Team Learnings Topic | Concept Memoirs |
|------|----------------------|-------------|---------------------|-----------------|
| **Architects** | `"design pattern OR architecture OR tech evaluation"` | `kodehold-teams` | `kodehold-learnings` | `kodehold-teams`, `kodehold-learnings` |
| **Engineers** | `"implementation OR pattern OR library OR performance"` | `kodehold-teams` | `kodehold-learnings` | `kodehold-teams`, `kodehold-learnings` |
| **Testers** | `"test OR edge case OR regression OR coverage"` | `kodehold-teams` | `kodehold-learnings` | `kodehold-teams`, `kodehold-learnings` |
| **Reviewers** | `"review OR security OR quality OR bug pattern"` | `kodehold-teams` | `kodehold-learnings` | `kodehold-teams`, `kodehold-learnings` |
| **Scribes** | `"documentation OR knowledge OR memory"` | `kodehold-teams` | `kodehold-learnings` | `kodehold-teams`, `kodehold-learnings` |
| **FLS** | `"hotfix OR bug OR escalation OR pattern"` | `kodehold-teams` | `kodehold-learnings` | `kodehold-teams`, `kodehold-learnings` |

### How Parameters Map to Steps

| Parameter | Used in Step | Purpose |
|-----------|-------------|---------|
| Shared learnings query | Step 1 | What to search in `kodehold-learnings` before starting work |
| Team memoir | Step 2, Step 8 | Where to search for team-specific patterns (Step 2) and where to store distilled concepts (Step 8) |
| Team learnings topic | Step 7 | Where to store team-specific findings after task completion |
| Concept memoirs | Step 8 | Which memoirs to add/refine concepts in |

### Integration with KodeHold Lifecycle

The knowledge flow runs on every task delegation, adapting to the current lifecycle phase:

| Phase | Step 3 (Task) | Steps 6-7 (Store) |
|-------|--------------|-------------------|
| **INIT** | Architects create design doc + ADRs | Store design decisions, technology choices |
| **ACTIVE** | Engineers implement, Testers test, Reviewers review | Store implementation patterns, test strategies, review findings |
| **REVIEW** | Final verification and team meeting | Store review results, test outcomes |
| **CLOSED** | Scribes final documentation | Store project summary, extract reusable concepts |
| **REOPEN** | Impact analysis, design updates | Store impact assessment, updated decisions |

### Relationship to ICM Operations

The knowledge flow uses ICM MCP tools for all storage and retrieval:

- **Steps 1-2 (Search):** `icm_memory_recall`, `icm_memoir_search`
- **Step 5 (Consolidate):** `icm_memory_consolidate`
- **Steps 6-7 (Store):** `icm_memory_store`
- **Step 8 (Distill):** `icm_memoir_add_concept`, `icm_memoir_refine`, `icm_memory_extract_patterns`

### Consolidation Policy

ICM warns when a topic accumulates >7 entries. The knowledge flow enforces proactive consolidation:

- **Step 5** checks entry count before storing
- At >7 entries: consolidate before adding new memories
- Use `icm_memory_consolidate` to merge related entries
- Use `icm_memory_extract_patterns` to detect recurring patterns and create memoir concepts

### Auto-Dedup

ICM's MCP server auto-deduplicates: if a new memory has >85% hybrid similarity to an existing one in the same topic, it updates instead of creating a duplicate. This means agents should be descriptive enough in their memory content that semantically different facts don't collide.

## Consequences

### Positive

1. **Token savings.** Scribes no longer runs search steps (1-2) it never uses. Other teams skip Post-task when only retrieving context.
2. **Clarity.** Agents know exactly which steps apply to their invocation context. No more "execute each step" ambiguity.
3. **Structured lifecycle.** Pre-task → Task → Post-task becomes an explicit delegation pattern the Director orchestrates, not an implicit assumption.
4. **Extensibility.** New modes can be added (e.g., "Search-only" for read-only queries) without restructuring the skill.
5. **Consistency.** All teams follow the same mode definitions. Scribes' special case (Post-task only) is documented as a mode, not an ad-hoc exception.

### Negative

1. **Increased complexity.** Three modes instead of one linear sequence. Agents and Director must be aware of mode selection.
2. **Mode drift risk.** If the Director forgets to invoke Post-task after a team completes its work, learnings are lost. Mitigation: the Director's delegation table documents Post-task as a standard follow-up.
3. **Scribes adaptation.** Scribes' workflow must be updated to handle Post-task-only execution. Low risk since it already runs this way in practice.

### Follow-up

- [ ] Update `.opencode/skills/icm-knowledge-flow/SKILL.md` with three-mode structure
- [ ] Update all 6 agent files (`.opencode/agents/<team>.md`) with mode-aware ICM sections
- [ ] Update `AGENTS.md` delegation table with default mode per team
- [ ] Test that Scribes correctly skips search steps in Post-task mode
- [ ] Verify no agent runs step 3 as part of knowledge flow

## ADR References

- **ADR-0009** (ICM MCP Integration) — predecessor; defines which tools and memoirs are used
- **ADR-0018** (Scribes Centralization) — Scribes is always invoked post-task; this ADR formalizes that pattern
- **ADR-0006** (Second Opinion) — not directly related, but follows same delegation lifecycle awareness
