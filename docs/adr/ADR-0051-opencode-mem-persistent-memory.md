# ADR-0051: opencode-mem as KodeHold Persistent Memory Backend

## Status

Accepted

**Date:** 2026-07-01

**Note:** ADR-0051 numbering conflict — the ADR index already lists ADR-0051 as "Frontend Reactivity Strategy for DeepResearch" (2026-06-24). If that ADR file is created later, renumber this ADR to ADR-0052.

> **Update (ADR-0054):** The OpenCode RAG tools referenced in this ADR as complementary to opencode-mem have been superseded by Graphify knowledge graph per ADR-0054. The opencode-mem backend is unchanged, but the code retrieval partner is now Graphify, not OpenCode RAG. See ADR-0054 for details.

## Context

### The Problem

ADR-0050 removed the agentmemory dependency and proposed file-based `.opencode/memory/` storage as the replacement. That storage was **never implemented** — the directories exist but are empty:

```
.opencode/memory/
├── checkpoints/    # empty
├── decisions/      # empty
├── lessons/        # empty
├── metrics/        # empty
├── patterns/       # empty
└── prospective/    # empty
```

KodeHold currently has **no persistent memory across sessions**. Every session starts from zero context. Agents cannot recall prior decisions, learnings, or session state without re-reading all documentation. This is a significant productivity loss — agents re-discover patterns, re-read ADRs, and re-derive context that a memory system would provide instantly.

### Why File-Based Storage Failed

ADR-0050 proposed structured markdown files under `.opencode/memory/` as the persistent store. This approach has inherent limitations:

1. **No semantic search.** Files are retrieved by path, not by meaning. A file named `checkpoints/session-abc.md` cannot be found by querying "what did we decide about the memory backend?"
2. **No auto-capture.** File-based storage requires explicit writes. Agents must remember to save state — which defeats the purpose of persistent memory.
3. **No embedding or retrieval.** Without vector embeddings, there is no way to find relevant memories by semantic similarity. The `search_semantic` tool searches the indexed codebase (source files, docs), not runtime memory.
4. **No compaction or lifecycle management.** Memory files accumulate without consolidation, deduplication, or archival.
5. **High friction.** Every memory operation requires file creation, YAML frontmatter formatting, and manual organization.

### Why opencode-mem

[opencode-mem](https://github.com/nicholaswatertank/opencode-mem) is an MCP server that provides persistent memory for OpenCode with:

- **Semantic search** via vector embeddings (USearch backend, nomic-embed-text-v1 model)
- **Auto-capture** of conversation context without explicit agent action
- **Project-scoped memory** with configurable default scope
- **Compaction** to manage memory limits automatically
- **Local-first storage** at `~/.opencode-mem/data` — no external services required
- **REST API** on localhost for MCP integration

### Validation

Bob (the user's other agent) has been using opencode-mem successfully with the following validated configuration:

```json
{
  "webServerEnabled": true,
  "webServerPort": 4747,
  "webServerHost": "0.0.0.0",
  "storagePath": "~/.opencode-mem/data",
  "embeddingModel": "Xenova/nomic-embed-text-v1",
  "embeddingDimensions": 768,
  "memory": { "defaultScope": "project" },
  "autoCaptureEnabled": true,
  "autoCaptureLanguage": "en",
  "memoryProvider": "openai-chat",
  "memoryModel": "deepseek/deepseek-v4-flash",
  "memoryApiUrl": "https://openrouter.ai/api/v1",
  "memoryApiKey": "env://OPENROUTER_API_KEY",
  "showAutoCaptureToasts": true,
  "showUserProfileToasts": true,
  "showErrorToasts": true,
  "vectorBackend": "usearch-first",
  "compaction": { "enabled": true, "memoryLimit": 10 }
}
```

This configuration is proven to work and will be reused for KodeHold with minor adjustments.

### Key Forces

1. **Session amnesia.** Without persistent memory, every session starts cold. Agents re-read ADRs, re-discover patterns, and re-derive context that could be recalled in seconds.

2. **ADR-0050 left a gap.** The migration removed agentmemory but the file-based replacement was never implemented. KodeHold has been operating without any persistent memory since the migration.

3. **opencode-mem is validated infrastructure.** Bob's configuration proves it works with OpenCode's MCP system. No daemon management, no npm packages, no custom scripts — just an MCP server entry in `opencode.json`.

4. **Complementary to code retrieval tools.** Graphify (per ADR-0054) searches structural code relationships (callers, callees, imports, class hierarchy). opencode-mem searches runtime memory — session context, agent learnings, conversation history. They serve different purposes and do not overlap.

5. **Minimal integration surface.** opencode-mem integrates via MCP. Adding it requires one entry in `opencode.json` — no agent file rewrites, no skill changes, no script updates.

## Decision

### 1. Add opencode-mem MCP Server to opencode.json

Add the opencode-mem MCP server configuration to `opencode.json`:

```json
{
  "mcp": {
    "opencode-mem": {
      "type": "local",
      "command": ["npx", "-y", "opencode-mem"],
      "environment": {
        "OPENCODE_API_KEY": "{env:OPENCODE_API_KEY}"
      }
    }
  }
}
```

This exposes opencode-mem's memory tools (`search_memories`, `add_memory`, `get_memory`, `delete_memory`, `list_memories`, `update_memory`) to all agents via MCP.

### 2. Configure opencode-mem for KodeHold

Create `~/.config/opencode/opencode-mem.jsonc` (or reuse Bob's existing config if KodeHold shares the same OpenCode instance):

```jsonc
{
  "webServerEnabled": true,
  "webServerPort": 4747,
  "webServerHost": "0.0.0.0",
  "storagePath": "~/.opencode-mem/data",
  "embeddingModel": "Xenova/nomic-embed-text-v1",
  "embeddingDimensions": 768,
  "memory": { "defaultScope": "project" },
  "autoCaptureEnabled": true,
  "autoCaptureLanguage": "en",
  "memoryProvider": "openai-chat",
  "memoryModel": "deepseek/deepseek-v4-flash",
  "memoryApiUrl": "https://openrouter.ai/api/v1",
  "memoryApiKey": "env://OPENROUTER_API_KEY",
  "showAutoCaptureToasts": true,
  "showUserProfileToasts": true,
  "showErrorToasts": true,
  "vectorBackend": "usearch-first",
  "compaction": { "enabled": true, "memoryLimit": 10 }
}
```

**Key configuration decisions:**
- `storagePath: "~/.opencode-mem/data"` — shared storage with Bob (same OpenCode instance). Project scoping via `memory.defaultScope: "project"` keeps memories isolated per workspace.
- `embeddingModel: "Xenova/nomic-embed-text-v1"` — local embedding via ONNX. No API calls for embedding generation.
- `memoryModel: "deepseek/deepseek-v4-flash"` via OpenRouter — LLM for memory summarization and auto-capture processing.
- `compaction.enabled: true, memoryLimit: 10` — automatic memory consolidation when limits are reached.

### 3. Agent Memory Tool Integration

opencode-mem provides these MCP tools available to all agents:

| Tool | Purpose | Primary Users |
|------|---------|---------------|
| `search_memories(query, scope?)` | Semantic search across stored memories | All teams (pre-task knowledge retrieval) |
| `add_memory(content, scope?, tags?)` | Store a new memory | All teams (post-task learning capture) |
| `get_memory(id)` | Retrieve a specific memory by ID | All teams |
| `list_memories(scope?, tags?)` | List memories with optional filters | Director, Scribes |
| `update_memory(id, content)` | Update an existing memory | Scribes (memory maintenance) |
| `delete_memory(id)` | Remove a memory | Scribes (cleanup) |

**Team-specific integration:**

| Team | Memory Usage | Pattern |
|------|-------------|---------|
| **Director** | Recall prior delegation patterns, session context, project history | `search_memories(query="<project> delegation patterns", scope="project")` before delegating |
| **Architects** | Recall prior design decisions, ADR rationale, technology evaluations | `search_memories(query="<topic> design decisions", scope="project")` before writing ADRs |
| **Engineers** | Recall prior implementation patterns, bug fixes, refactorings | `search_memories(query="<domain> implementation patterns", scope="project")` before coding |
| **Reviewers** | Recall prior review findings, common issues, reviewer preferences | `search_memories(query="<area> review findings", scope="project")` before reviewing |
| **Testers** | Recall prior test patterns, edge cases, regression history | `search_memories(query="<feature> test patterns", scope="project")` before testing |
| **FLS** | Recall prior triage decisions, hotfix patterns, escalation history | `search_memories(query="<symptom> triage history", scope="project")` before triaging |
| **Scribes** | Maintain memory hygiene, consolidate, archive | `list_memories(scope="project")` + `update_memory()` for maintenance |

### 3b. Project Scoping Requirement

> **CRITICAL: All memory tool calls MUST include `scope: "project"`.** KodeHold shares an opencode-mem instance with other agents (Bob). Without explicit project scoping, `search_memories` and `add_memory` operations will return or write memories from ALL projects, causing cross-project memory bleed.

**Rule:** Every `search_memories`, `add_memory`, `list_memories`, and `delete_all_memories` call MUST include `scope: "project"`. There are NO exceptions.

**Why this matters:**
- `opencode-mem.jsonc` sets `memory.defaultScope: "project"` as the default
- But agents should not rely on the default — always pass `scope: "project"` explicitly
- This is defense-in-depth: even if the default changes, agents remain safe
- Bob's memories (from other projects) must never appear in KodeHold search results

**Implementation:** The `graphify-knowledge-flow` skill and all 7 agent files include this scoping requirement. The skill's "Persistent Memory Recall" section shows all memory tool calls with `scope: "project"`.

### 4. Relationship to Graphify and Code Retrieval

opencode-mem and Graphify serve **different purposes**:

| Aspect | Graphify | opencode-mem |
|--------|----------|--------------|
| **What it retrieves** | Structural code relationships (callers, callees, imports, class hierarchy) | Runtime memory (session context, agent learnings) |
| **When it's populated** | At graph build time (tree-sitter AST parsing) | At runtime (auto-capture + explicit `add_memory`) |
| **Search method** | Deterministic, navigable AST-based graph queries | Semantic vector search on memory entries |
| **Use case** | "What does this function call?" "Where is this symbol defined?" | "What did we learn last session?" "What triage pattern applied here?" |
| **Scope** | Codebase structure and dependencies | Conversation history and agent knowledge |
| **Persistence** | Tied to file system (regenerated on file changes) | Independent of files (memories persist across sessions) |

**Integration pattern:** Agents use Graphify for code retrieval and opencode-mem for runtime memory. The `graphify-knowledge-flow` skill handles Graphify retrieval; a parallel memory recall step handles opencode-mem retrieval. See [ADR-0054](ADR-0054-replace-opencode-rag-with-graphify.md) for the full Graphify integration.

> **Previous complementary system:** OpenCode RAG tools (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) were previously used as the code retrieval layer alongside opencode-mem (per v1.16.0). As of ADR-0054, these have been replaced by Graphify as the sole documented code retrieval method. The built-in tools remain available as platform-level primitives but are not part of KodeHold's workflow.

### 5. Deprecate File-Based Memory Directories

The `.opencode/memory/` directory structure proposed in ADR-0050 is **superseded** by opencode-mem:

| Directory | Status | Action |
|-----------|--------|--------|
| `.opencode/memory/checkpoints/` | Empty, never used | Deprecate — opencode-mem handles session context |
| `.opencode/memory/decisions/` | Empty, never used | Deprecate — decisions go in ADRs (file-based) |
| `.opencode/memory/lessons/` | Empty, never used | Deprecate — opencode-mem stores learnings |
| `.opencode/memory/metrics/` | Empty, never used | Deprecate — metrics go in token-report.py *(removed)* |
| `.opencode/memory/patterns/` | Empty, never used | Deprecate — opencode-mem stores patterns |
| `.opencode/memory/prospective/` | Empty, never used | Deprecate — use todowrite for task tracking |

**Note:** The `.opencode/memory/` directory itself should be kept (for `.gitignore` purposes and any future file-based storage needs), but the subdirectories are no longer part of the memory architecture.

### 6. Impact on ADRs

#### ADRs Superseded by This ADR

| ADR | Title | Current Status | Impact |
|-----|-------|---------------|--------|
| ADR-0050 | Agentmemory → OpenCode RAG Migration | Accepted | Section 5 (File-Based Persistent Storage) superseded. The file-based storage proposal was never implemented; opencode-mem replaces it. The rest of ADR-0050 (agentmemory removal, RAG tool adoption) remains valid. |

#### ADRs to Update

| ADR | Update Required |
|-----|----------------|
| ADR-0050 | Add note that §5 (File-Based Persistent Storage) is superseded by ADR-0051. Update Section 7.2 reference in design doc to mention opencode-mem. |
| ADR-0038 | Knowledge Recall Protocol — add opencode-mem as a recall source alongside Graphify |
| ADR-0039 | Pre-Flight Knowledge Check — add opencode-mem search as a pre-flight step |

## Consequences

### Positive

1. **Persistent memory across sessions.** Agents recall prior context, decisions, and learnings without re-reading all documentation.
2. **Semantic search on runtime memory.** Unlike file-based storage, memories are searchable by meaning, not just by path.
3. **Auto-capture.** Conversations are automatically captured — agents don't need to remember to save state.
4. **Local-first.** All data stays on the local machine at `~/.opencode-mem/data`. No external service dependencies for storage.
5. **Minimal integration surface.** One MCP server entry in `opencode.json`. No agent file rewrites, no skill changes, no script updates.
6. **Validated configuration.** Bob's proven config means no trial-and-error — it works out of the box.
7. **Compaction.** Automatic memory consolidation prevents unbounded growth.

### Negative

1. **New dependency.** opencode-mem adds an MCP server dependency. If it fails, memory tools are unavailable (but agents still function — they just can't recall memories).
   *Mitigation:* opencode-mem is a local process, not a remote service. Failure modes are limited to npm issues or embedding model failures. Graphify and file-based documentation remain available regardless.

2. **Shared storage with Bob.** If KodeHold and Bob share `~/.opencode-mem/data`, memories from both agents are in the same store. Project scoping (`defaultScope: "project"`) isolates them, but cross-project queries could surface unrelated memories.
   *Mitigation:* Project scoping is enforced by opencode-mem. If isolation is insufficient, configure a separate `storagePath` for KodeHold (e.g., `~/.opencode-mem/kodehold-data`).

3. **OpenRouter dependency for memory LLM.** The `memoryModel` uses OpenRouter (`deepseek/deepseek-v4-flash`) for memory processing. If OpenRouter is unreachable, auto-capture and memory summarization may degrade.
   *Mitigation:* `embeddingModel` is local (Xenova/nomic-embed-text-v1 via ONNX). Search works without OpenRouter. Only auto-capture processing requires the LLM.

4. **ADR-0051 numbering conflict.** The ADR index already lists ADR-0051 as "Frontend Reactivity Strategy for DeepResearch" (2026-06-24). This ADR uses the same number.
   *Mitigation:* If the Frontend Reactivity ADR file is created, renumber this ADR to ADR-0052.

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| opencode-mem MCP server fails to start | Low | Medium | Agents function without memory; OpenCode RAG still works |
| Memory search returns irrelevant results | Medium | Low | Project scoping + compaction limits noise |
| Shared storage causes cross-agent memory pollution | Low | Medium | Configure separate `storagePath` if needed |
| Embedding model download fails (first run) | Low | Low | nomic-embed-text-v1 is small (~130MB); falls back gracefully |
| OpenRouter outage stops auto-capture | Medium | Low | Manual `add_memory` still works; search unaffected |

## Documentation

| Field | Value |
|-------|-------|
| **Tool** | opencode-mem |
| **Source** | https://github.com/nicholaswatertank/opencode-mem |
| **Version documented** | Latest (2026-07-01) |
| **Key concepts** | MCP server for persistent memory. Semantic search via USearch + nomic-embed-text-v1. Auto-capture of conversations. Project-scoped memory. Local-first storage. |
| **Configuration** | `~/.config/opencode/opencode-mem.jsonc` — validated by Bob |
| **Known Gotchas** | • **First run downloads embedding model** — nomic-embed-text-v1 (~130MB ONNX) downloads on first use. Subsequent runs use cached model. |
| | • **OpenRouter API key required** — `memoryModel` processing uses OpenRouter. Set `OPENROUTER_API_KEY` environment variable. |
| | • **Port 4747 must be available** — opencode-mem web server binds to `0.0.0.0:4747`. Ensure no conflict with other services. |
| | • **Project scoping is by workspace path** — memories are scoped to the OpenCode workspace directory. Switching workspaces changes the memory scope. |

## ADR References

- **ADR-0050** (Agentmemory → OpenCode RAG Migration) — §5 (File-Based Persistent Storage) superseded by this ADR
- **ADR-0038** (Knowledge Recall Protocol) — to be updated with opencode-mem recall step
- **ADR-0039** (Pre-Flight Knowledge Check) — to be updated with opencode-mem pre-flight step

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.0 | 2026-07-01 | Initial ADR — opencode-mem as KodeHold Persistent Memory Backend |
