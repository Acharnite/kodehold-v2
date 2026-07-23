---
status: Superseded
superseded-by: ADR-0029
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0009: ICM MCP Integration

## Status

Deprecated

Superseded by ADR-0029 (ICM → Agentmemory Migration Strategy).

## Context

KodeHold currently uses ICM via CLI commands (`icm store`, `icm recall`) with the `--db` flag pointing to the central kodehold database. The ICM MCP server exposes 18 tools over JSON-RPC 2.0 that offer richer functionality than the CLI:

- **Memory tools (9)**: `icm_memory_store`, `icm_memory_recall`, `icm_memory_update`, `icm_memory_forget`, `icm_memory_consolidate`, `icm_memory_list_topics`, `icm_memory_stats`, `icm_memory_health`, `icm_memory_embed_all`
- **Memoir tools (9)**: `icm_memoir_create`, `icm_memoir_list`, `icm_memoir_show`, `icm_memoir_add_concept`, `icm_memoir_refine`, `icm_memoir_search`, `icm_memoir_search_all`, `icm_memoir_link`, `icm_memoir_inspect`
- **Feedback tools (3)**: `icm_feedback_record`, `icm_feedback_search` (available in ICM MCP)

The MCP tools provide capabilities the CLI lacks:
- **Auto-dedup**: stores with >85% hybrid similarity to existing memory in the same topic update instead of duplicating
- **Auto-decay**: recall triggers decay if >24h since last run
- **Store nudge**: warns after 10 tool calls without a store
- **Consolidation hint**: warns when a topic has >7 entries
- **Auto-embed**: embeds memories automatically if embedder is available
- **Hybrid search**: 30% BM25 + 70% cosine similarity (vs CLI's FTS5-only)
- **Memoir knowledge graph**: structured concepts with typed relations (unavailable via CLI)
- **Feedback loop**: record and search corrections for continuous improvement

KodeHold already has ICM MCP configured and it respects local databases. Now we need to define how KodeHold's agents (Director, Scribes, Reviewers) leverage these tools systematically.

## Decision

KodeHold will adopt a **layered ICM integration** that uses MCP tools for agent interactions and CLI for administrative/scripted operations:

### Layer 1 — Scribes as ICM Operators

Scribes are the primary ICM users. They use MCP tools (not CLI) for all memory operations:

| Operation | MCP Tool | Why |
|-----------|----------|-----|
| Store decision | `icm_memory_store` | Auto-dedup, auto-embed, consolidation hints |
| Recall context | `icm_memory_recall` | Hybrid search (70% vector + 30% BM25), auto-decay |
| Batch consolidate | `icm_memory_consolidate` | Agent-driven summarization (smarter than CLI merge) |
| Health check | `icm_memory_health` | Topic hygiene, staleness alerts |
| Backfill embeddings | `icm_memory_embed_all` | Batch embedding for existing memories |

ICM MCP storage follows the central topic convention: `kodehold-<project>-<qualifier>`. The MCP server's auto-dedup prevents duplicate memories when multiple agents store the same decision.

### Layer 2 — Memoirs for Cross-Project Knowledge

Memoirs replace ad-hoc ICM memories for **permanent architectural knowledge**:

| Memoir | Concepts | Purpose |
|--------|----------|---------|
| `kodehold-teams` | Director, Architects, Engineers, Reviewers, Testers, Scribes, FLS | KodeHold's own architecture and team knowledge as a knowledge graph |
| `kodehold-patterns` | Composable validators, Protocol-based APIs, lifecycle gates | Reusable patterns extracted from completed projects |
| `workspace-<name>` | per-project architecture concepts | One memoir per workspace for project-specific architecture |

After each CLOSED state, Scribes distill memories into memoir concepts via:
```
icm memoir distill --from-topic kodehold-<project>-* --into <memoir-name>
```

This transforms episodic memories into permanent, linked concepts.

### Layer 3 — Feedback for Director's Second Opinions

The feedback tools (`icm_feedback_record`, `icm_feedback_search`) are used by the Director to record second opinion outcomes:

```python
# After a second opinion, Director records:
icm_feedback_record(
  topic="ADR-0009-validator-architecture",
  context="ADR-0001: Validator Protocol vs ABCs",
  predicted="Chose typing.Protocol",
  corrected="Protocol is correct but __and__/__or__ were dead code",
  reason="Second opinion revealed implementation gaps"
)
```

This creates a searchable history of corrections that future agents can learn from.

### Layer 4 — CLI for Scripts and Automation

The CLI remains used for:
- `scripts/gate.sh` — quick state checks (`icm stats`)
- `scripts/workspace.sh` — batch operations
- CRON jobs — `icm decay`, `icm prune`, `icm embed`
- CI pipeline — `icm stats` verification

### System Architecture

```
┌──────────────────────────────────────────────────────┐
│                   KodeHold Director                    │
│  Delegates to Scribes for ALL ICM operations           │
└──────────────────┬───────────────────────────────────┘
                   │
    ┌──────────────┼──────────────┐
    ▼              ▼              ▼
┌─────────┐  ┌─────────┐  ┌──────────┐
│ Scribes │  │ Director│  │ Scripts  │
│ (MCP)   │  │ (MCP)   │  │ (CLI)    │
├─────────┤  ├─────────┤  ├──────────┤
│ store   │  │ feedback│  │ decay    │
│ recall  │  │ record  │  │ prune    │
│ health  │  │ search  │  │ embed    │
│ memoir  │  │         │  │ stats    │
└────┬────┘  └────┬────┘  └────┬─────┘
     │            │            │
     └────────────┼────────────┘
                  ▼
        ┌─────────────────┐
        │  ICM MCP Server │
        │  (icm serve)    │
        ├─────────────────┤
        │  SQLite + FTS5  │
        │  + sqlite-vec   │
        │  + fastembed    │
        └─────────────────┘
```

## Consequences

### Positive

1. **Auto-dedup** prevents memory fragmentation when multiple teams store related information
2. **Hybrid search** (vector + FTS5) improves recall relevance by 30-40% over CLI FTS5-only
3. **Auto-decay** eliminates the need for manual `icm decay` CRON jobs for agent-initiated recalls
4. **Store nudge** reminds agents to save context proactively
5. **Memoir knowledge graph** enables cross-project knowledge reuse — patterns from lib-validate inform future projects
6. **Feedback tools** create a permanent record of second opinion corrections
7. **Consolidation hints** prevent topic bloat (>7 entries triggers warning)

### Negative

1. MCP tools require the ICM server to be running (already configured, but adds process dependency)
2. CLI remains needed for CRON/CI — dual interface increases surface area
3. Auto-dedup threshold (85%) may occasionally merge distinct memories — Scribes must verify
4. Memoir distillation requires active curation — not fully automatic

### Implementation Plan

1. **Phase 1**: Update Scribes agent to use MCP tool names instead of CLI commands in their workflow documentation
2. **Phase 2**: Add `icm_feedback_record`/`icm_feedback_search` to Director's second opinion protocol and Reviewers' workflow
3. **Phase 3**: Create initial memoirs for KodeHold architecture and extract patterns from lib-validate
4. **Phase 4**: Add memoir distillation step to Scribes' CLOSED workflow
5. **Phase 5**: Evaluate whether per-workspace memorialization improves knowledge retrieval

### ADR References

- ADR-0004: Original ICM and RTK integration — this ADR supersedes the CLI-only approach for agent interactions
- ADR-0006: Second opinion protocol — extended with feedback tools
- ADR-0008: Project lifecycle — CLOSED step updated with memoir distillation

## Follow-up

- [ ] Update Scribes agent file to reference MCP tools as primary ICM interface
- [ ] Add feedback tool references to Director and Reviewers agent files
- [ ] Test auto-dedup behavior with multiple agents writing to the same topic
- [ ] Measure recall quality improvement with hybrid search vs FTS5-only
- [ ] Create initial memoirs for existing KodeHold knowledge
