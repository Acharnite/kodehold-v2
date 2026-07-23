---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0038: Knowledge Recall Protocol

## Status

Accepted

**Note (ADR-0050):** The knowledge recall protocol using `agentmemory_memory_lesson_recall` has been replaced by OpenCode's `search_semantic` tool. See ADR-0050 for details.

**Version:** 1.0
**Last Updated:** 2026-06-03

## Context

### The Problem

Agentmemory's lesson system stores learnings via `memory_lesson_save` and auto-strengthens duplicates. However, the **recall path** was broken — teams searching for relevant lessons before starting work consistently returned zero results. The root causes:

1. **No project scoping on recall queries.** The `project` parameter was omitted from `memory_lesson_recall` calls, so searches returned results from across all projects (or none at all).

2. **Limit too low.** The default `limit=5` was insufficient for meaningful search results. Teams searching for patterns would get at most 5 hits — often fewer after relevance filtering.

3. **No team tags on lessons.** Lessons were stored without team-specific tags (e.g., `engineers`, `testers`, `reviewers`). Generic queries like "lessons patterns" returned zero results because the semantic search had no team context to narrow against.

4. **Generic queries returned zero results.** Without team-prefixed query formats, the semantic search engine had no anchor point. A query like "lessons patterns delegation" works because it combines the lesson taxonomy term with a domain term. A query like "delegation" alone is too broad and under-specific for agentmemory's hybrid search.

### Existing Infrastructure

The Agentmemory Knowledge Flow skill (`.opencode/skills/agentmemory-knowledge-flow/SKILL.md`) already defined a pre-task protocol for searching learnings, but it was never updated to use project-scoped queries with adequate limits and team-prefixed formats.

- ADR-0030 defined the initial knowledge flow protocol with ICM
- After the ICM→agentmemory migration (ADR-0029), the skill was updated but the recall parameters were not tuned
- 122 lessons existed in agentmemory but were effectively invisible to teams

### Key Insight

The lesson system's auto-strengthening (via `memory_lesson_save`) is append-only for tags — tags are immutable after creation. Once a lesson is stored without team tags, those tags cannot be added retroactively. The only way to make existing lessons discoverable by team is to create companion lessons with the correct tags.

## Decision

### Three-Part Implementation

We implement three improvements to fix the knowledge recall path:

#### 1. Batch-Tag Existing Lessons with Companion Lessons

All 122 existing lessons receive companion lessons with team tags and the `kodehold-learnings` topic tag. Each companion lesson uses a unique content marker (UUID suffix) to avoid collision with the original lesson — this is necessary because `memory_lesson_save` auto-strengthens on content match.

**Companion lesson structure:**
```
content: "[kodehold-learnings companion] <original-summary> [<uuid>]"
tags: "<team>, kodehold-learnings, <domain-tags>"
confidence: 0.8
project: "kodehold"
```

**Tag taxonomy:**
| Tag | Purpose | Applied To |
|-----|---------|------------|
| `kodehold-learnings` | Topic tag — all learnings | Every companion lesson |
| `engineers` | Engineering patterns | Lessons about implementation, code, refactoring |
| `testers` | Testing patterns | Lessons about tests, verification, edge cases |
| `reviewers` | Review patterns | Lessons about code review, design review |
| `architects` | Architecture patterns | Lessons about design, ADRs, technology choices |
| `fls` | Support patterns | Lessons about debugging, hotfixes, triage |
| `scribes` | Documentation patterns | Lessons about memory, documentation, processes |

**Why companion lessons instead of schema changes:**
- Schema change (add a `team` field to lessons) was rejected — would need a migration of the agentmemory database, which is complex and risky
- A hypothetical `memory_lesson_update` MCP tool was considered but rejected — too complex to build for this use case
- Companion lessons are simple, reversible, and work with existing infrastructure

#### 2. Update SKILL.md with Recall Parameters

The knowledge flow skill file is updated with three improvements:

- **Project scoping:** All `memory_lesson_recall` calls include `project="kodehold"`
- **Increased limit:** Default `limit=5` → `limit=10` for primary search, `limit=5` for fallback
- **Team-prefixed query format:** Queries use `"<team-name> lessons patterns <keywords>"` format instead of generic terms
- **Fallback step:** If the primary query returns fewer than 3 results, a broader search is attempted with `limit=5`

#### 3. Update Agent File Pre-Task Sections

All 5 agent files (engineers.md, reviewers.md, testers.md, architects.md, fls.md) have their pre-task knowledge flow sections updated to reference the skill correctly. The broken `kodehold-teams` references are removed.

**Files NOT changed (by design):**
- `scribes.md` — Scribes has no pre-task knowledge flow (documentation work does not benefit from lesson recall in the same way)
- `director.md` — No Lesson Tagging Rule added because auto-consolidation in agentmemory handles storage automatically; the Director does not need explicit lesson-save protocols

## Consequences

### Positive

1. **Knowledge recall works.** Teams searching for relevant patterns before starting work now get meaningful results — typically 5-10 lessons per query instead of zero.

2. **Backwards compatible.** No changes to the storage path, no schema migrations, no infrastructure changes. The companion approach works entirely through the existing `memory_lesson_save` API.

3. **Team-tagged discoverability.** Lessons are now discoverable by team context. An engineer searching for "engineers lessons patterns async" finds relevant engineering lessons, not architecture lessons.

4. **No protocol changes needed.** The existing ADR-0030 knowledge flow protocol was sound — it just needed correct parameters. No new ADRs or design documents are needed for the recall path.

### Negative

1. **Companion lessons create copies.** Each existing lesson now has a companion. This doubles the lesson count (~122 → ~244) and increases storage slightly. However, agentmemory's consolidation pipeline handles this gracefully — companion lessons are lightweight text records.

2. **No retroactive tag updates.** Companion lessons are copies, not updates. The original lessons remain untagged. If agentmemory ever gains a tag-update API, the originals should be updated and companions removed.

3. **Project slug normalization.** Lessons originally stored with full filesystem paths as project identifiers (e.g., `/home/kiffer/project/kodehold`) needed normalization to the `"kodehold"` slug format. This was done as part of the batch update.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Companion lessons get out of sync** with originals | Low | Low | Companions are search-only — they don't need to stay in sync. Originals are authoritative for content. |
| 2 | **Tag collision** between team tags | Low | Low | Each lesson typically belongs to one primary team. Multi-team lessons get multiple tags. |
| 3 | **Project slug changes** in the future | Low | Medium | The `"kodehold"` slug is now stable per ADR-0036. If it changes, all lessons need re-tagging. |

## Alternatives Considered

### Schema Change (Add `team` Field to Lessons)

Add a `team` dimension to agentmemory's lesson schema, then migrate all existing lessons.

**Rejected.** Agentmemory does not support schema migrations. Adding a field would require either:
- Dropping and recreating the lesson table (destructive)
- Building a custom migration tool (complex, high effort)
- Both approaches are high-risk for a recall-path fix

### `memory_lesson_update` MCP Tool

Build an MCP tool that allows updating lesson tags after creation.

**Rejected.** While this would be the cleanest solution, building a new MCP tool for a one-time tagging operation is disproportionate effort. The companion approach achieves the same result with existing infrastructure.

### Do Nothing

Leave the recall path broken and rely on teams to find lessons through other means (e.g., reading agentmemory directly).

**Rejected.** The entire purpose of the knowledge flow is to make lessons discoverable at the point of work. If recall is broken, the lesson system provides zero value.

## ADR References

- **ADR-0030** (Agentmemory Knowledge Flow) — defines the knowledge flow protocol that this ADR fixes the recall path for
- **ADR-0029** (ICM → Agentmemory Migration Strategy) — migration that created the lesson system now being tagged
- **ADR-0036** (Project Slug Convention) — defines the `"kodehold"` slug format used for project scoping
- **ADR-0033** (Crystals + Signals) — crystals produce lessons that are now discoverable via this recall fix

### Source Files Modified

- `.opencode/skills/agentmemory-knowledge-flow/SKILL.md` — updated recall parameters (project scoping, limit, query format, fallback)
- `.opencode/agents/engineers.md` — removed broken `kodehold-teams` reference, updated pre-task section
- `.opencode/agents/reviewers.md` — same
- `.opencode/agents/testers.md` — same
- `.opencode/agents/architects.md` — same
- `.opencode/agents/fls.md` — same
- `scripts/tag-lessons.py` — batch companion lesson creation (created and run once)

### Files NOT Modified (by design)

- `.opencode/agents/scribes.md` — no pre-task knowledge flow needed
- `.opencode/agents/director.md` — no Lesson Tagging Rule needed (auto-consolidation handles storage)
