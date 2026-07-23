---
status: Superseded
superseded-by: agentmemory (memory_graph_query / memory_reflect / memory_insight_list)
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0023: Semantic Memory Automation

## Status

Superseded

Replaced by agentmemory (memory_graph_query, memory_reflect, memory_insight_list) — Layer 3 (chat.system.transform) implements automated semantic recall. ICM memoir system (concepts, links, extract_patterns) provides the knowledge graph. GitHub issue #29 closed.

## Context

KodeHold has memoir/concept/link capabilities via ICM, but these require manual creation. ADRs contain architectural decisions that could be extracted as concepts, and design documents contain component relationships that could be mapped to the knowledge graph. Currently, no automatic concept extraction exists.

The current approach has these limitations:

- ADRs are written but concepts within them are not extracted to the knowledge graph
- Design doc components are not linked to related concepts across projects
- Cross-project knowledge reuse requires manual concept creation
- The ICM knowledge flow skill provides manual steps, but no automation
- Similar decisions across projects are not linked automatically
- The knowledge graph remains sparse despite rich ADR/design doc content

The key forces are:

- Semantic memory (what is true) is distinct from episodic memory (what happened)
- ADRs are the primary source of architectural knowledge — natural extraction target
- Design documents contain component relationships — natural linking target
- ICM memoir system already supports concepts and links — no new infrastructure needed
- Too much extraction creates noise — must focus on significant concepts only
- Cross-project knowledge reuse is a key value proposition of KodeHold

## Decision

Implement hooks that automatically extract concepts from new ADRs and design doc changes, updating the knowledge graph via the ICM memoir system.

### Extraction Sources

| Source | What to Extract | Target Memoir |
|--------|----------------|---------------|
| New ADR | Decision, context, consequences as concepts | `kodehold-teams` |
| Updated ADR | Refined concepts, new relationships | `kodehold-teams` |
| Design doc component | Component role, relationships, dependencies | `kodehold-teams` |
| Team structure | Team roles, responsibilities, interactions | `kodehold-teams` |
| Cross-project patterns | Reusable patterns, common decisions | `kodehold-learnings` |

### Concept Extraction Rules

| Rule | Description | Example |
|------|-------------|---------|
| Decision concept | Extract the core decision from ADR Context/Decision | "Hierarchical Memory" from ADR-0020 |
| Relationship concept | Extract links between concepts | "ADR-0020 depends_on ADR-0004 (ICM)" |
| Pattern concept | Extract reusable patterns across ADRs | "3-tier memory" pattern from ADR-0020 |
| Anti-pattern concept | Extract things to avoid | "Static importance" anti-pattern |

### Linking Rules

| Source Concept | Target Concept | Relation | When |
|----------------|---------------|----------|------|
| New ADR | Related ADRs | `related_to` | ADR references another ADR |
| New ADR | Components | `refines` | ADR modifies existing component |
| Design change | Existing concepts | `instance_of` | Component is instance of pattern |
| Cross-project | Similar decisions | `alternative_to` | Different approach to same problem |

### Extraction Triggers

| Trigger | When | Action |
|---------|------|--------|
| ADR created | After `write` to `docs/adr/ADR-*.md` | Extract concepts from new ADR |
| ADR updated | After status change or significant edit | Refine existing concepts |
| Design doc updated | After component section changes | Update component relationships |
| Team meeting | After REVIEW→CLOSED | Extract cross-cutting concepts |
| Project closed | After final state storage | Extract project-level patterns |

### Knowledge Graph Maintenance

| Operation | Frequency | Purpose |
|-----------|-----------|---------|
| Concept deduplication | On each extraction | Prevent duplicate concepts |
| Link validation | Weekly sweep | Remove orphaned links |
| Confidence boost | On concept refinement | Increase confidence for repeated patterns |
| Concept consolidation | Monthly | Merge similar concepts |

### Deduplication Strategy

Before creating a new concept:

1. Search existing memoir for concepts with similar names (fuzzy match)
2. If match found with confidence > 0.8 → refine existing concept instead
3. If match found with confidence 0.5-0.8 → create link as `related_to`
4. If no match → create new concept

### Memoir Structure

| Memoir | Purpose | Concept Types |
|--------|---------|---------------|
| `kodehold-teams` | Architecture decisions, team knowledge | Decision, Pattern, Anti-pattern, Component, Role, Responsibility, Workflow |
| `kodehold-learnings` | Cross-project lessons | Pattern, Anti-pattern, Gotcha, Best-practice |

### Implementation Plan

| File | Change |
|------|--------|
| scribes.md | Add concept extraction workflow, deduplication, linking rules |
| architect agent | Add extraction trigger after ADR creation |
| icm-knowledge-flow SKILL.md | Add concept extraction and linking steps |
| design doc | Add section 7.9 — Semantic Memory Automation |

## Consequences

- Positive: Cross-project knowledge reuse — concepts from one project inform others
- Positive: Knowledge graph becomes richer over time without manual effort
- Positive: Concept deduplication prevents knowledge graph pollution
- Positive: Links between ADRs create navigable decision graphs
- Negative: Deduplication is imperfect — may create near-duplicate concepts
- Negative: Extraction adds overhead to ADR creation (~100-200 tokens)
- Negative: Knowledge graph maintenance requires periodic sweeps
- Neutral: Deduplication thresholds (0.5, 0.8) may need tuning
- Note: agentmemory's knowledge graph (memory_graph_query + memory_reflect + memory_insight_list) provides this functionality.
