---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0007: Token Optimization Strategy

## Status

Superseded by [ADR-0058](../adr/ADR-0058-loop-engineering-integration.md)

**Superseded:** 2026-07-21

All concepts from this ADR have been absorbed and modernized by ADR-0058:
- **Per-phase token budgets** → modernized with per-automation-run caps and kill switch
- **English-only policy** → absorbed into standard KodeHold conventions
- **Tiered context loading** → superseded by Graphify + opencode-mem retrieval
- **Minimal prompt templates** → replaced by The Ladder (ADR-0049)
- **File chunking** → superseded by Graphify structural queries
- **Token tracking** → replaced by `scripts/token_usage.py` + loop-cost integration

## Context

Token consumption is the primary cost driver in LLM-based systems. KodeHold, as a multi-team orchestrator, is particularly vulnerable because each task involves multiple agents, each consuming tokens for context, prompts, and responses. Without an intentional optimization strategy, token usage scales linearly with team count and project complexity.

Key forces:
- Token cost is both monetary (API models) and contextual (context window limits)
- Aggressive optimization can degrade output quality if not done carefully
- Different operations have different token profiles (reading vs. writing vs. reviewing)
- Configs and documentation in non-English languages consume more tokens for the same information

## Decision

### Strategy Layers

#### 1. Language: English Only
All configuration files, prompts, agent messages, documentation, ADRs, and design documents are written in English. Compared to Danish (the user's native language), English uses 15-25% fewer tokens for equivalent technical content due to compound word differences and more concise technical vocabulary.

#### 2. CLI: RTK Mandatory
All shell commands use RTK as a proxy. RTK's compact format reduces token output by 40-60% compared to standard CLI tools. This is mandatory, not optional. See ADR-0004.

#### 3. Context Windows: Tiered Loading
| Tier | Description | Token Budget | When Used |
|------|-------------|-------------|-----------|
| Full | Complete files + ICM memories | Up to context limit | Initial load, critical reviews |
| Summary | ICM summaries only | 30% of full | Normal operations |
| Minimal | Titles + keywords only | 10% of full | Light mode, routine checks |

#### 4. Prompt Templates: Minimal
All agent prompts use the shortest effective template. Examples are stored separately and loaded only when needed. The template hierarchy:
- **Core**: System prompt + single instruction (no examples, no explanations)
- **Extended**: Core + up to 2 examples (used for complex or unfamiliar tasks)
- **Full**: Core + examples + reference material (used only for first-time tasks)

#### 5. Chunking: Large File Splitting
Files > 150 lines are chunked. Each chunk is processed independently. Chunks are summarized and only summaries are carried forward.

#### 6. Deduplication: Context Cache
Within a single session, context is deduplicated. If two teams receive the same design doc excerpt, it is loaded once and referenced, not duplicated.

#### 7. Token Budget: Per-Phase Allocation
| Phase | Max Tokens | Notes |
|-------|-----------|-------|
| Context load | 8k | Including design doc + ICM summary |
| Code generation | 12k | Including spec + constraints |
| Code review | 8k | Including diff + standards |
| Test generation | 8k | Including spec + code |
| Documentation | 4k | Including code + decisions |
| Second opinion | 6k | See ADR-0006 |

### Compliance

Token usage is tracked per session via:
1. ICM message table (token counts per message)
2. Director's token budget tracker — runs `scripts/token-usage.sh` before each delegation and warns when approaching per-phase budgets (80% warning, 100% alert)
3. Session compression logging — Scribes includes per-team token consumption in ICM summaries
4. Checkpoint token usage — session checkpoints include token usage per team

## Consequences

- Positive: Predictable token costs — budgets prevent runaway consumption
- Positive: English-only reduces token count without losing information
- Positive: Tiered loading means context fits within smaller windows
- Negative: Chunking can miss cross-chunk patterns (mitigated by chunk overlap)
- Negative: Minimal prompts may lead to lower quality on unfamiliar tasks
- Negative: Enforcing token budgets adds orchestration complexity
