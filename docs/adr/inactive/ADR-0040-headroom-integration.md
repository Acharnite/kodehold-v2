---
status: Accepted
phase:
  current: 1
  total: 4
  names:
    1: "headroom learn integration"
    2: "MCP tools registration"
    3: "Controlled evaluation on workspace project"
    4: "Full proxy integration"
  status:
    1: done
    2: not-started
    3: not-started
    4: not-started
---

# ADR-0040: Headroom Integration — Context Compression Layer

## Status

Deprecated

**Version:** 1.9
**Last Updated:** 2026-06-05

## Context

### Current Token Strategy

KodeHold's existing token optimization strategy (ADR-0007) establishes per-phase token budgets, tiered context loading, and English-only conventions. Session compression (ADR-0019) provides manual compression every 4 delegation rounds via Scribes-generated summaries. Pre-flight enforcement (ADR-0039) ensures knowledge recall runs before each delegation.

Despite these measures, KodeHold has three gaps:

1. **No real-time compression.** Tool output, chat history, and intermediate results accumulate without automatic compression. The 4-round manual cycle saves 60-80% but happens in batches — the space between rounds is unmanaged. Models at 32K context windows (Ollama) still approach overflow in high-volume sessions.

2. **No tool-output compression.** Large JSON blobs, file reads, and command output consume significant context. ADR-0007's tiered loading handles design docs, but arbitrary tool output is uncompressed.

3. **No automated failure mining.** When sessions fail or produce poor results, Scribes manually distills lessons (ADR-0033 crystals feed into this). There is no automated pipeline that ingests failed sessions and extracts actionable patterns into agent instructions.

### What Headroom Is

Headroom (https://github.com/chopratejas/headroom) by chopratejas is an Apache 2.0 licensed toolkit for agent context compression. It provides:

- **Three-stage pipeline architecture** (beyond just individual algorithms):
  - **CacheAligner** — Stabilizes prefixes for KV cache hits at Anthropic/OpenAI, improving inference speed on cached contexts
  - **ContentRouter** — Auto-detects content type (JSON, code, logs, text, diffs, HTML) and routes to the optimal compressor
  - **IntelligentContext** — Scores messages by recency, semantic similarity, TOIN-learned patterns, error indicators, forward references, and token density
- **Real-time compression:** 60-95% token reduction on agent workloads
- **Reversible compression (CCR):** Originals stored locally for decompression on demand
- **`headroom learn`:** Mines failed agent sessions, extracts root causes, and writes findings to agent instructions
- **MCP server:** Exposes `headroom_compress`, `headroom_retrieve`, `headroom_stats` as MCP tools
- **Multiple modes:** Library (inline), Proxy (sidecar), Agent wrap (wrapper), MCP server

**Compression ratios by content type:**

| Content Type | Compression Ratio | Algorithm | Latency |
|-------------|-------------------|-----------|---------|
| JSON arrays | 70-90% | SmartCrusher | ~1ms |
| Source code | 40-70% | CodeCompressor/AST | ~10ms |
| Search results | 80-95% | SearchCompressor | ~2ms |
| Build/test logs | 85-95% | LogCompressor | ~3ms |
| Plain text | 60-80% | TextCompressor / Kompress-base | ~5ms |
| Git diffs | 60-80% | DiffCompressor | ~5ms |
| HTML | 50-70% | HTMLCompressor | ~5ms |
| Images | 40-90% | ML router | variable |

**Repository activity:** 12.4k stars, 806 forks, 153 releases, Apache 2.0 license, very active development (latest release v0.23.0, Jun 4 2026).

### Key Forces

1. **Token cost is both monetary and contextual.** API models cost per token; small context models (32K) hit limits in active sessions. Real-time compression directly addresses both.

2. **Manual compression is insufficient.** Scribes' 4-round cycle is reactive and coarse-grained. Real-time compression would save context continuously, extending session length without manual intervention.

3. **Failure mining is ad-hoc.** Crystals capture completed work, but failed sessions and near-misses are lost unless a human or Scribes explicitly distills them. `headroom learn` automates this.

4. **External dependency risk.** Headroom is a third-party tool with its own release cadence, bugs, and compatibility constraints. Integration must be incremental and reversible.

5. **Complementary to agentmemory, not competing.** Agentmemory provides persistent long-term memory (working → episodic → semantic → procedural). Headroom provides ephemeral real-time compression. They serve different layers of the stack.

## Decision

Adopt Headroom in a phased approach starting with `headroom learn` (Architect's Option C — Hybrid), with controlled evaluation before deeper integration.

### Why Phased?

Each phase validates the next. If `headroom learn` provides value with minimal friction, we proceed to MCP tool registration. If MCP tools demonstrate reliable compression on workspace projects, we evaluate proxy integration. Any phase can be halted independently.

### Why Start with `headroom learn`?

`headroom learn` is the lowest-risk entry point:

- No changes to the delegation flow or tool stack
- Runs post-hoc on failed sessions — no runtime risk
- Outputs directly to AGENTS.md, which KodeHold already uses for agent instructions
- Provides immediate value (automated failure mining) with almost zero integration cost

## Implementation Phases

### Phase 1: `headroom learn` Integration (Immediate)

**Timeline:** Immediate
**Risk:** Very low
**Effort:** One-time setup + documentation changes

`headroom learn` performs **success correlation** — rather than just mining failures, it correlates what failed with what *then succeeded*. For example: a failed attempt to read path X followed by a successful read of path Y produces the learning "X is actually at Y." This makes the output specifically actionable.

**Output categories** (written between `<!-- headroom:learn:start -->` markers in agent instructions):

| Category | Description |
|----------|-------------|
| Environment Facts | OS, PATH, installed tool versions, runtime constraints |
| File Path Corrections | Incorrect paths discovered and corrected during the session |
| Search Scope | Which directories or glob patterns were effective or ineffective |
| Command Patterns | Successful vs. failing command invocations |
| Known Large Files | Files that should be excluded from reads due to size |

**Architecture:** Scanner → Analyzer → Writer, using an adapter pattern supporting multiple agent systems (OpenCode, Claude Code, etc.).

**Proven scale:** Tested on **67,583 tool calls across 23 projects** — demonstrating robust real-world performance.

**CLI reference:**
- `headroom learn` — dry-run against the current project
- `headroom learn --apply` — write findings to agent instructions
- `headroom learn --project <path>` — analyze a specific project
- `headroom learn --all --apply` — analyze all projects and apply findings
- `headroom learn --claude-dir <path>` — specify custom CLAUDE.md directory

1. Install Headroom: `pip install headroom-ai`
2. Add `headroom learn` to Scribes' failure distillation workflow:
   - After a failed session (or on explicit request), Scribes runs `headroom learn --apply`
   - Review output between `<!-- headroom:learn:start -->` markers, integrate actionable findings into agent instructions
3. Document the protocol in Scribes' workflow

**Success criteria:** Scribes can run `headroom learn` on a failed session and produce actionable, specific corrections (not generic advice). No regression in existing workflows.

### Phase 2: MCP Tools Registration (After Phase 1)

**Timeline:** After Phase 1 validated
**Risk:** Low
**Effort:** Register MCP tools, update Director/Scribes protocols

1. Install with MCP extras: `pip install "headroom-ai[mcp]"` (lightweight, no heavy dependencies)
2. Register Headroom as an MCP server: `headroom mcp install` (auto-registers with Claude Code, or manual config as fallback)
3. Alternative manual config in `.opencode/mcp.json`:
   ```json
   {
     "headroom": {
       "command": "headroom",
       "args": ["mcp"],
       "type": "stdio"
     }
   }
   ```
4. Three MCP tools exposed:
   - **`headroom_compress`** — Compress content and return a hash. Accepts `content` and optional `context` (for IntelligentContext scoring).
   - **`headroom_retrieve`** — Retrieve original content by hash. Supports optional `query` for semantic search within compressed content.
   - **`headroom_stats`** — Returns session-level compression statistics (total tokens in, total out, savings percentage, per-type breakdown).
5. Expose optional compression at key choke points:
   - After large tool outputs (before they enter chat history)
   - Before session checkpoint storage
   - On explicit Director request (compression opt-in)
6. Update Director's protocol to allow opt-in compression calls
7. **Streamable HTTP Transport** supported for remote/Docker agents — proxy auto-exposes MCP at `http://host:8787/mcp`
8. **Sub-agent stats aggregation** via shared file at `~/.headroom/session_stats.jsonl`

**Success criteria:** MCP tools (`headroom_compress`, `headroom_retrieve`, `headroom_stats`) are accessible and functional. Compression can be triggered on demand with sub-500ms latency.

### Phase 3: Controlled Evaluation on Workspace Project (1 Week)

**Timeline:** After Phase 2
**Risk:** Low
**Effort:** Test on a workspace project, measure savings

1. Select a workspace project (e.g., a medium-complexity adopted project)
2. Run a full delegation cycle with Headroom compression enabled at choke points
3. Measure:
   - Token reduction per compression event
   - Impact on output quality (correctness, completeness)
   - Latency overhead
   - Reliability (error rate, decompression success)
4. Compare against baseline (same project without Headroom)

**Success criteria:** Quantitative data on compression ratio, quality impact, and reliability. Decision on Phase 4 based on data.

### Phase 4: Full Proxy Integration (If Evaluation Passes)

**Timeline:** After Phase 3, conditional on positive evaluation
**Risk:** Medium
**Effort:** Infrastructure setup, proxy configuration, monitoring

1. **Docker deployment:** `ghcr.io/chopratejas/headroom:latest` — or run standalone: `headroom proxy --port 8787`
2. **Alternative wrapper:** `headroom wrap claude` provides transparent wrapping without manual proxy config
3. **Production deployment:** gunicorn with uvicorn workers for production-grade serving
4. **Monitoring:** Prometheus metrics available at `/metrics`
5. **Cloud backend support:** Compatible with Bedrock, Vertex AI, Azure OpenAI, and OpenRouter for provider flexibility
6. **CLI configuration flags:**
   - `--budget <tokens>` — set output token budget
   - `--llmlingua` — enable LLMLingua-2 compression
   - `--no-intelligent-context` — disable IntelligentContext scoring
   - `--log-file <path>` — write compression logs
7. Configure KodeHold's LLM endpoint to route through the proxy
8. Enable real-time transparent compression of all LLM-bound context
9. Monitor for quality degradation, latency, and reliability issues

**Success criteria:** Proxy operates transparently with measurable token savings and no quality regression. Rollback plan: disable proxy and route directly.

## OpenCode Integration

### OpenCode Platform Context

KodeHold runs on OpenCode, an open-source agentic coding platform configured in `opencode.json` at the project root:

- **Two providers:** DeepSeek v4 Flash (primary, model `opencode/deepseek-v4-flash` at `opencode.ai/zen/v1` — configured in desktop app) and OpenRouter (second-opinion, model `google/gemma-3-12b-it` at `openrouter.ai/api/v1` — configured in project-level `opencode.json`). Note: `opencode.json` also contains an unused Ollama (`qwen3:8b-opencode`) entry; it is not actively used.
- **Multi-level config:** Provider configuration spans multiple levels — project-level `opencode.json` defines OpenRouter; the primary DeepSeek provider is set directly in the OpenCode desktop app UI.
- **Native compaction:** `compaction: { auto: true, prune: true, reserved: 7000 }` — rolling window keeping the last 7000 tokens, pruning older history
- **MCP servers:** One existing GitHub server (`github-mcp-server`). No Headroom MCP yet.
- **Agent model override:** Agents specify provider/model via YAML frontmatter (e.g., `model: openrouter/google/gemma-3-12b-it`), enabling per-agent provider routing without global config changes

### Integration by Phase

| Phase | opencode.json Changes | Runtime Impact |
|-------|----------------------|----------------|
| **1** | None | Scribes runs `headroom learn --apply` post-hoc; output to AGENTS.md |
| **2** | Add Headroom MCP to `mcpServers` | Agents call `headroom_compress`/`retrieve`/`stats` as opt-in tools |
| **3** | Same MCP as Phase 2 | Add measurement/monitoring instrumentation |
| **4a** | Configure DeepSeek endpoint to proxy port; optionally disable compaction | All primary agent traffic routes through Headroom transparently |
| **4b** | Add second provider entry with proxy baseURL for OpenRouter | Second-opinion agent routes through Headroom |

**Phase 1 — No opencode.json changes.** Scribes runs `headroom learn --apply` after failed sessions. Findings are written to `AGENTS.md` between `<!-- headroom:learn:start -->` markers. This is purely post-hoc — no delegation flow or tool stack changes.

**Phase 2 — Add Headroom MCP server.** Register Headroom alongside the existing GitHub server:
```json
"mcpServers": {
  "github": { "type": "local", "command": ["node", "tools/github-mcp-server"] },
  "headroom": { "type": "local", "command": ["headroom", "mcp"], "description": "Context compression" }
}
```
Three tools become available: `headroom_compress`, `headroom_retrieve`, `headroom_stats`. Agents opt in to compression per call. No provider routing changes needed.

**Phase 3 — Same MCP config.** No additional opencode.json changes. Add token usage monitoring and quality measurement instrumentation outside of OpenCode config.

**Phase 4a — Proxy for primary provider (DeepSeek v4 Flash):**
1. Start proxy: `headroom proxy --port 8787 --openai-api-url https://opencode.ai/zen/v1`
2. Configure DeepSeek's endpoint in the desktop app to point at the proxy, e.g. set the API URL to `http://localhost:8787/v1`. Alternatively, set via environment variable `OPENAI_BASE_URL=http://localhost:8787/v1` or add DeepSeek as an explicit provider in `opencode.json` with the proxy baseURL.
3. Optionally disable native compaction: `"compaction": { "auto": false }`

Note: This proxy is a passive pass-through — Headroom compresses all context transparently using its existing proxy mode, which is mature and tested (153 releases, 67,583 tool calls in benchmarks). No custom Headroom integration or agentmemory hooks are needed. Headroom's IntelligentContext scoring handles message prioritization automatically.

**Phase 4b — Proxy for second-opinion (OpenRouter/Gemma 3):**
- **Option A (dedicated proxy):** Start `headroom proxy --port 8788 --openai-api-url https://openrouter.ai/api/v1`. Add a second OpenRouter provider entry with `baseURL: http://localhost:8788/v1`. Second-opinion agent uses `model: openrouter-proxy/google/gemma-3-12b-it`.
- **Option B (multi-backend):** If Headroom supports routing by model prefix, use a single proxy with rules to route `google/gemma-*-*` to OpenRouter and everything else to DeepSeek. Falls back to Option A if unsupported.

### Provider Routing

KodeHold's multi-provider architecture maps to Headroom as follows:

| Agent Group | Model | Phase 1-3 Route | Phase 4 Route |
|-------------|-------|-----------------|---------------|
| Primary agents (director, engineers, scribes, etc.) | `opencode/deepseek-v4-flash` via DeepSeek API | Direct `opencode.ai/zen/v1` | Proxy `:8787` → `opencode.ai/zen/v1` |
| Second-opinion subagent | `google/gemma-3-12b-it` via OpenRouter | Direct OpenRouter | Proxy `:8788` → OpenRouter |

In Phase 4, both providers benefit from Headroom's IntelligentContext scoring, SmartCrusher compression, and reversible CCR storage. Each proxy instance operates independently — failure of one does not affect the other.

### Native Compaction Interaction

OpenCode's built-in compaction is a simple rolling window:
```json
"compaction": { "auto": true, "prune": true, "reserved": 7000 }
```
It keeps the last 7000 tokens of conversation history regardless of importance. Headroom's **IntelligentContext** scoring uses multi-factor prioritization:

- **Recency scoring** — more recent messages get higher priority
- **Semantic similarity** — messages related to current task are preserved
- **TOIN-learned patterns** — task-specific importance learned over time
- **Error indicators** — error messages and correction patterns are retained
- **Forward references** — content referenced later is prioritized
- **Token density** — information-dense content preferred over verbose

**Recommendation:** Replace native compaction when Headroom proxy is active:
```json
"compaction": { "auto": false }
```
Headroom's context-aware compression is strictly more sophisticated than OpenCode's rolling window. If proxy is disabled, re-enable native compaction as a fallback.

### Architecture Diagrams

**Before Headroom (Current):**
```
OpenCode ──→ OpenCode Zen (opencode.ai/zen/v1) ──→ opencode/deepseek-v4-flash
         └──→ OpenRouter ──────→ google/gemma-3-12b-it
```

**Phase 2 (MCP Tools):**
```
OpenCode ──→ OpenCode Zen (opencode.ai/zen/v1)
         ├──→ Headroom MCP ──→ headroom_compress/retrieve/stats
         └──→ GitHub MCP
Agent invokes headroom_compress → compressed result enters chat history
```

**Phase 4 (Full Proxy):**
```
Primary agents:    OpenCode ──→ Headroom (:8787) ──→ opencode.ai/zen/v1
Second-opinion:    OpenCode ──→ Headroom (:8788) ──→ OpenRouter
Headroom Proxy provides: IntelligentContext scoring, SmartCrusher,
CodeCompressor, LogCompressor, reversible CCR storage
```

### Rollback Procedure

To disable Headroom and return to original configuration:

1. **Stop proxy processes:**
   ```bash
   kill $(lsof -t -i:8787)  # Stop primary proxy
   kill $(lsof -t -i:8788)  # Stop second-opinion proxy
   ```
2. **Restore opencode.json:**
   - Revert DeepSeek endpoint in desktop app to `https://opencode.ai/zen/v1`
   - Remove any added provider entries (e.g., `openrouter-proxy`)
   - Re-enable native compaction: `"compaction": { "auto": true, "prune": true, "reserved": 7000 }`
3. **Remove MCP server** (if rolling back past Phase 2): delete the `headroom` entry from `mcpServers`
4. **No code changes needed** — all changes are configuration-only in `opencode.json`

### Config File Changes Summary

| Phase | File | Change |
|-------|------|--------|
| 1 | `AGENTS.md` | Add `<!-- headroom:learn:start -->` markers |
| 2 | `opencode.json` | Add `mcpServers.headroom` block |
| 3 | (none) | Measurement instrumentation only |
| 4a | Desktop app / env var / `opencode.json` | Configure DeepSeek endpoint to `:8787` proxy; set `compaction.auto: false` |
| 4b | `opencode.json` | Add second provider entry or routing rule for OpenRouter |

## Implementation Plan

### Phase 1 — headroom learn (Immediate)

| Step | Action | Who |
|------|--------|-----|
| 1 | `pip install headroom-ai` | User |
| 2 | Add Director workflow: detect delegation failures, delegate `headroom learn --apply` to Scribes | Director |
| 3 | Add Scribes workflow: receive delegation from Director, run `headroom learn --apply`, review findings between `<!-- headroom:learn:start -->` markers, integrate into AGENTS.md | Scribes |
| 4 | Review and integrate findings into AGENTS.md (available to all agents) | Scribes |

**No opencode.json changes required.** Director workflow updates are documented in director.md (§Headroom Learn Protocol); Scribes workflow updates are documented in scribes.md.

### Phase 2 — MCP Tools (After Phase 1 validated)

| Step | Action | Who |
|------|--------|-----|
| 1 | `pip install "headroom-ai[mcp]"` | User |
| 2 | `headroom mcp install` — auto-register with OpenCode | User |
| 3 | Update director.md with tool selection rules: `headroom_compress` = runtime content, `memory_compress_file` = files on disk | Scribes |
| 4 | Update scribes.md with boundaries: `headroom learn` = failure post-mortem → AGENTS.md; agentmemory consolidation = ongoing patterns → memory database | Scribes |
| 5 | Document double-compression risk during Phase 2–3 (OpenCode native compaction + headroom_compress run concurrently) | Scribes |

### Phase 3 — Controlled Evaluation (1 week after Phase 2)

| Step | Action | Who |
|------|--------|-----|
| 1 | Select a workspace project for evaluation | Director |
| 2 | Activate Headroom MCP compression at choke points (large tool outputs, before checkpoints) | Director |
| 3 | Measure: token reduction, quality impact, latency, reliability | Director |

**Evaluation criteria:**
- ≥40% token reduction on compressed outputs without measurable quality degradation
- Zero critical failures (compression or decompression errors that block work)
- Latency overhead <500ms per compression event
- Scribes reports no workflow disruption
- Pattern detection quality: compare agentmemory pattern mining output with/without Headroom
- Cost impact: measure compute overhead vs token savings in dollar terms
- Comparison baseline: same project cycle without Headroom

### Phase 4 — Proxy (Only if Phase 3 passes)

| Step | Action | Who |
|------|--------|-----|
| 1 | Start proxy: `headroom proxy --port 8787 --openai-api-url https://opencode.ai/zen/v1` | User |
| 2 | Configure DeepSeek endpoint in desktop app to `http://localhost:8787/v1` | User |
| 3 | Disable native compaction: `"compaction": { "auto": false }` in opencode.json | Scribes |
| 4 | Monitor for 1 week: token savings, latency, quality | Director |

**Rollback:** Stop proxy → revert endpoint in desktop app → re-enable compaction. See §OpenCode Integration → Rollback Procedure for the complete 4-step rollback process.

## Accuracy Benchmarks

Headroom's compression has been benchmarked against standard evaluation datasets to verify no degradation in output quality:

| Benchmark | Baseline | With Headroom | Delta |
|-----------|----------|---------------|-------|
| **GSM8K** (math reasoning) | 0.870 | 0.870 | ±0.000 |
| **TruthfulQA** (factual accuracy) | 0.530 | 0.560 | **+0.030** |
| **SQuAD v2** (question answering) | — | 97% accuracy at 19% compression | — |
| **BFCL** (tool calling) | — | 97% accuracy at 32% compression | — |

These results demonstrate that Headroom's compression preserves — and in some cases improves — output quality. The SQuAD v2 and BFCL results show that even at aggressive compression ratios (19-32% of original size), task accuracy remains above 97%.

## Consequences

### Positive

1. **Automated failure mining.** `headroom learn` extracts actionable patterns from failed sessions without manual effort, closing a gap in KodeHold's knowledge loop.

2. **Real-time compression.** Phases 2-4 progressively add real-time compression, reducing context pressure between Scribes' manual compression cycles.

3. **KV cache optimization.** CacheAligner stabilizes KV cache prefixes, improving inference speed on cached contexts at Anthropic/OpenAI — not just token savings, but faster model responses.

4. **Reversible compression.** CCR ensures no information is permanently lost — originals can be retrieved if needed.

5. **Complementary to existing infrastructure.** Agentmemory handles persistent knowledge; Headroom handles ephemeral compression. They address different layers without overlap.

6. **Incremental adoption.** Each phase can be halted independently. If Headroom proves unreliable, only the completed phases are affected.

7. **Proven accuracy.** Benchmarks show zero degradation on math reasoning (GSM8K) and factual improvement on TruthfulQA, with 97%+ task accuracy at high compression ratios.

### Negative

1. **External dependency.** Headroom adds a third-party dependency with its own maintenance burden, release cadence, and potential breaking changes.

2. **New infrastructure.** Even at Phase 2 (MCP), Headroom requires installation, configuration, and monitoring. Phase 4 adds a proxy process.

3. **Quality risk.** Compression is inherently lossy. Aggressive settings may discard nuance needed for complex tasks — though benchmarks suggest the risk is low at conservative settings.

4. **Latency overhead.** Compression algorithms add processing time. Real-time compression (Phase 4) could increase perceived latency.

5. **Compute overhead.** Compression algorithms consume CPU/GPU resources. Phase 2 (MCP) adds ~1-10ms per compression event. Phase 4 (proxy) adds a persistent process with ~100-500MB memory overhead. These costs are modest relative to LLM API costs but should be measured in Phase 3 evaluation.

6. **Debugging complexity.** Compressed context makes it harder to debug LLM behavior — the raw context seen by the model differs from what developers observe in logs. Mitigation: Headroom's CCR (reversible compression) preserves originals for debugging via `headroom_retrieve`. Log both compressed and original token counts.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Headroom project abandonment or breaking changes** | Low | High | Pin a specific version. Maintain rollback capability. Open source — can fork if needed. |
| 2 | **Compression degrades output quality** | Medium | Medium | Phase 3 evaluation measures quality impact. Conservative compression settings. Reversible (CCR). Benchmarks show no degradation. |
| 3 | **Proxy adds reliability failure point** | Low | Medium | Phase 4 proxy includes health checks and automatic bypass on failure. |
| 4 | **`headroom learn` produces low-quality output** | Medium | Low | Output is reviewed by Scribes before integration. No automated ingestion of learn output. |
| 5 | **Unquantified cost/benefit ratio** | Medium | Low | Phase 3 evaluation measures token savings vs compute cost. Headroom's benchmarks show 60-95% token reduction at ~1-10ms latency. At DeepSeek V4 Flash pricing ($0.14/M input, $0.28/M output), even 40% reduction on large contexts quickly justifies the overhead. |
| 6 | **Incompatibility with specific LLM providers** | Low | Medium | Proxy mode is configurable. Can fall back to MCP-only or library mode per provider. |

## Alternatives Considered

### Option A: Full Proxy Integration (Rejected for Now)

Run `headroom proxy --port 8787` as a sidecar between KodeHold and the LLM, enabling transparent real-time compression.

**Positive:**
- Maximum token savings (60-95% on agent workloads)
- Transparent to all agents — no protocol changes
- Works with any LLM provider

**Negative:**
- Highest infrastructure complexity
- Proxy is a new failure point in the critical path
- Quality impact is harder to measure and control
- If proxy fails, KodeHold is blocked
- Cannot evaluate incrementally

**Why rejected for now:** Too much risk without validation. Proxy integration should only proceed after Phases 1-3 demonstrate value and reliability.

### Option B: `headroom learn` Only (Rejected)

Adopt only `headroom learn` for automated failure mining, without MCP tools or proxy compression.

**Positive:**
- Lowest risk and effort
- Addresses the failure-mining gap
- No runtime changes

**Negative:**
- Does not address the real-time compression gap
- Misses 60-95% potential token savings from compression algorithms
- Phase 2 and 3 provide evaluation data that informs future decisions

**Why rejected:** Addresses only one of three gaps. The real-time compression gap (tool output, chat history) is equally important. Phases 2-3 are low-risk and provide essential data.

### Option C: Hybrid — `headroom learn` + MCP Tools (Selected)

Start with `headroom learn` (immediate value), then register MCP tools for opt-in compression (controlled evaluation), with proxy as a potential future step.

**Positive:**
- Lowest-risk path to maximum value
- Each phase is independently valuable and reversible
- Evaluation data from Phase 3 informs the proxy decision
- No single point of failure (MCP tools are opt-in)
- Addresses all three gaps (failure mining, real-time compression, tool-output compression)
- Proven benchmarks (97%+ accuracy at high compression) strengthen the case for deeper integration
- CacheAligner's KV cache optimization provides inference speed benefits, not just token savings

**Negative:**
- Slower time-to-value for proxy-level compression
- Requires multi-phase planning and execution
- MCP tool registration requires OpenCode configuration changes

**Why selected:** Provides the best risk/reward profile. Immediate value from `headroom learn`, controlled evaluation before deeper integration, and a clear path to proxy if warranted. Newly available benchmarks and granular compression data validate the architecture's effectiveness.

### Do Nothing (Rejected)

Continue with the current strategy: manual session compression every 4 rounds, no automated failure mining, no real-time compression.

**Why rejected:** The gaps are real and growing. As KodeHold scales to more projects and larger workloads, manual compression and ad-hoc failure mining become bottlenecks. Headroom addresses these gaps with a proven, available tool.

## Integration with Agentmemory

Agentmemory and Headroom serve different primary functions in KodeHold's memory architecture. This section analyzes their interaction points, identifies real functional overlaps, and defines KodeHold-only mitigations.

### 1.1 Comparison Table

| Aspect | Agentmemory | Headroom |
|--------|-------------|----------|
| **Purpose** | Persistent long-term memory | Ephemeral real-time compression |
| **Data** | Knowledge, lessons, decisions | Tool output, chat history, intermediate results |
| **Persistence** | Permanent (tiered consolidation) | Temporary (reversible via CCR) |
| **Granularity** | Structured memories + lessons | Raw text + structured data |
| **Access** | Recall, query, graph traversal | Compress, retrieve, stats |
| **Lifecycle** | Working → Episodic → Semantic → Procedural | In-session only (decompressed on demand) |

### 1.2 Data Flow Reality

The agentmemory capture plugin (`~/.config/opencode/plugins/agentmemory-capture.ts`) hooks into OpenCode's **INTERNAL** event system at the application layer. It captures tool output via OpenCode events (`tool_output: safeSlice(st.output, 8000)`), which fires **BEFORE** the data reaches the HTTP/network layer.

In Phase 4 proxy mode, the Headroom proxy compresses at the **HTTP level** (between OpenCode and the LLM provider). The capture plugin has already captured the raw output at the application level before it reaches the proxy.

```
Tool executes → OpenCode events fire → Capture plugin stores RAW output
                                                 ↓
                                       Output enters chat history
                                                 ↓
                                 Headroom proxy compresses (HTTP level)
                                                 ↓
                                       LLM receives compressed context
```

**Conclusion:** Agentmemory always receives raw, uncompressed tool output. The Headroom proxy operates downstream at a different layer of the stack. The capture plugin's observations are never affected by compression — agentmemory's pattern mining, reflection, and consolidation always operate on the original data.

### 1.3 Debugging and Observability

Headroom compression introduces a layer between tool output and the LLM that can complicate debugging:

**Challenge:** The LLM sees compressed context, while developers and logs see raw tool output. Discrepancies can make it hard to understand why the LLM behaves a certain way.

**Mitigations:**
- CCR (reversible compression): Originals are always retrievable via `headroom_retrieve` by hash
- Dual logging: Log both compressed and original token counts per request
- Phase 2 (MCP) is transparent — compression happens explicitly, so agents know when content is compressed
- Phase 4 (proxy): Monitor via Headroom's Prometheus metrics at `/metrics` and `headroom_stats` MCP tool
- Headroom's accuracy benchmarks (GSM8K ±0.000, TruthfulQA +0.030) show no quality degradation

### 1.4 Functional Overlap Analysis

With the data flow clarified, the overlap surface is significantly smaller than initially estimated. The earlier concerns about data fidelity (Overlap A) and pre-flight recall neutralization (Overlap B) are resolved by the layer separation — no action needed. Only three functional overlaps remain, all requiring KodeHold-only mitigations:

| Overlap | Severity | What happens | Mitigation (KodeHold-only) |
|---------|----------|-------------|---------------------------|
| **MCP tool namespace** | MEDIUM | 57 memory+compression tools may confuse agents — an agent looking for "compress" might call `memory_compress_file` when it should call `headroom_compress`, or vice versa | Update agent instructions (director.md, scribes.md) with clear tool selection rules and decision trees |
| **headroom learn vs consolidation** | MEDIUM | Both extract lessons from session data: agentmemory consolidates via 4-tier pipeline and reflection; `headroom learn` does failure-focused success correlation writing to AGENTS.md. Potential for duplicate or conflicting lessons. | Define clear boundaries in Scribes' workflow: `headroom learn` = failure post-mortem → AGENTS.md (agent instructions); agentmemory consolidation = ongoing pattern extraction → memory database. They target different outputs and serve different purposes. |
| **OpenCode compaction overlap** | LOW | OpenCode's native compaction (`auto: true, prune: true, reserved: 7000`) runs concurrently with Headroom IntelligentContext during Phase 2–3 (MCP active, no proxy). Risk of double-compression where content may be lost from both mechanisms. | Disable native compaction in Phase 4 (`"compaction": { "auto": false }` in opencode.json). For Phase 2-3: accept the double-compression risk — Native compaction is a simple rolling window (last 7K tokens); Headroom compression is opt-in and content-aware. The risk is content being pruned by OpenCode before agents can compress it, which is managed by running `headroom_compress` soon after large outputs. |

### 1.5 Conclusion

The integration analysis confirms that:

- **No changes required to agentmemory or Headroom.** The capture plugin and proxy operate at different layers — there is no data fidelity conflict. Agentmemory always receives raw observations regardless of compression mode.

- **All three mitigations are KodeHold-only:** agent instructions (tool selection rules), workflow definitions (Scribes' boundary between `headroom learn` and consolidation), and configuration (native compaction disable in Phase 4).

- **This keeps agentmemory general-purpose.** No special hooks, exemptions, or dual-write logic needed. Agentmemory's architecture remains clean.

- **This keeps Headroom unmodified.** No patches, forks, or custom builds required. Headroom can be installed and used as-is from PyPI.

- **Integration risk is LOW.** The remaining overlaps are well-understood, limited in scope, and trivially mitigated within KodeHold's existing configuration and agent instruction layer.

## ADR References

- **ADR-0007** (Token Optimization Strategy) — establishes per-phase token budgets, tiered loading, English-only convention
- **ADR-0019** (Session Context Compression via Periodic ICM Summaries) — manual compression every 4 delegation rounds (Superseded — replaced by agentmemory)
- **ADR-0039** (Pre-Flight Knowledge Check Enforcement) — ensures knowledge recall before delegation
- **ADR-0030** (Agentmemory Knowledge Flow) — defines the knowledge flow protocol
- **ADR-0033** (Crystals + Signals) — crystals produce lessons via consolidation pipeline

## Open Questions

1. **Should we pin a specific version of Headroom?** Yes — pin to the latest stable release at time of Phase 1 installation. Version pinning prevents unexpected breaking changes. Revisit pin periodically.

2. **What is the evaluation criteria for Phase 3?** See Evaluation criteria in the Phase 3 Implementation Plan section.

3. **Which compression algorithms should be enabled by default?** Start with SmartCrusher (JSON) and CodeCompressor (AST) for structured data. Evaluate Kompress-base (ModernBERT, requires PyTorch, ~2GB) for general text compression in Phase 3.

4. **Should `headroom learn` run automatically or on-demand?** Start on-demand (Scribes triggers it after failed sessions). If proven valuable, add a sentinel trigger for automatic execution.

5. **What is the rollback plan for Phase 4 proxy?** Disable proxy via configuration flag and route directly to LLM. No code changes required if proxy is a configurable endpoint.

6. **Which installation extras are needed per phase?**
   - Phase 1: `headroom-ai` (base)
   - Phase 2: `headroom-ai[mcp]` (adds MCP tools)
   - Phase 3: `headroom-ai[mcp,code]` (adds tree-sitter AST parsing)
   - Phase 4: `headroom-ai[proxy]` (adds proxy server, HTTP API)
    - Optional for advanced: `[ml]` (Kompress, ~2GB), `[all]` (everything)

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.9 | 2026-06-05 | Second-opinion review (Gemma 3 12B): added Cost Considerations and Debugging subsections, moved evaluation criteria to Phase 3 plan, strengthened rollback and proxy clarity, added cost/benefit risk, clarified compaction mitigation. Bumped version from 1.8. |
| 1.8 | 2026-06-05 | Phase 1 Implementation Plan: added Director role (delegation workflow) alongside Scribes, expanded table from 3 to 4 steps, updated note referencing director.md (§Headroom Learn Protocol) and scribes.md. Bumped version from 1.7. |
| 1.7 | 2026-06-05 | Fixed all DeepSeek API endpoint references from api.deepseek.com to opencode.ai/zen/v1 (OpenCode Zen). Updated model ID from opencode-go/deepseek-v4-flash to opencode/deepseek-v4-flash. Added concrete Implementation Plan section with step-by-step phase assignments. |
| 1.6 | 2026-06-05 | Revised Integration with Agentmemory section — simplified overlap analysis based on code investigation finding that capture plugin operates at application layer ahead of HTTP/proxy layer. Removed Risk 5 and Open Questions Q7–Q8. All mitigations are KodeHold-only (no agentmemory or Headroom changes). |
| 1.5 | 2026-06-05 | Expanded Integration with Agentmemory section — added Functional Overlap Analysis (10 overlap areas identified, 5 active with mitigations), Mitigations Summary, and Open Questions Q7–Q8. Added Risk 5 (compression reduces pattern mining quality). |
| 1.4 | 2026-06-05 | Removed all Ollama references — Ollama is unused/legacy; reduced to two providers (DeepSeek primary, OpenRouter second-opinion); updated OpenCode Platform Context, Provider Routing table, Phase 4b Option B, and architecture diagrams. |
| 1.3 | 2026-06-05 | Corrected provider information — DeepSeek v4 Flash is primary model (not Ollama); updated Phase 4a proxy, provider routing table, architecture diagrams, rollback procedure, and config summary. |
| 1.2 | 2026-06-05 | Added OpenCode Integration section — platform context, phase-by-phase config mapping, provider routing, native compaction interaction, architecture diagrams, rollback procedure, and config changes summary table. |
| 1.1 | 2026-06-05 | Expanded architecture details (3-stage pipeline), granular compression ratios table, accuracy benchmarks, repository stats, `headroom learn` success correlation, detailed MCP install/extras, Docker/proxy/cloud deployment options, CacheAligner KV benefits. |
| 1.0 | 2026-06-04 | Initial ADR — Headroom integration evaluation |
