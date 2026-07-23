# ADR-0054: Replace opencode-rag with Graphify Knowledge Graph for Code Retrieval

## Status

**Accepted** — 2026-07-14

## Context

### The Problem

KodeHold currently relies on **opencode-rag** (the separate command `opencode-rag mcp` exposed as the `krypto-agent` MCP server) for semantic code retrieval. The three-tier retrieval architecture consists of:

1. **Built-in OpenCode RAG tools** (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) — these are part of OpenCode core, always available, and cannot be removed.

2. **`krypto-agent` MCP server** — a separate process running `opencode-rag mcp` from `/home/kiffer/project/krypto-agent`, registered as an MCP server in `opencode.json`. This duplicates functionality already provided by the built-in tools.

3. **Plugin references** — `.opencode/plugins/rag-plugin.js` and `.opencode/plugins/rag-tui.js` are listed in `opencode.json` but do not exist on disk. These are stale references from a previous configuration.

The `opencode-rag mcp` server uses **semantic vector search** (embeddings-based similarity) to find relevant code. In practice, this approach has demonstrated a recurring problem: **it finds wrong files.** Vector search operates on semantic similarity of text embeddings, which is fundamentally imprecise for code understanding:

- Two functions that talk about the same concept but do different things get conflated
- Structural relationships (caller/callee, import/export, class hierarchy) are invisible to vector search
- File paths and symbol names are treated as text, not as a navigable graph
- The relevance scores provide no structural context — a score of 0.85 could mean "similar comment text" rather than "actual dependency"

### Prior Art: ADR-0050 Migration

ADR-0050 (Agentmemory → OpenCode RAG Migration) established that OpenCode's built-in RAG tools (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) are the primary code retrieval mechanism. The `opencode-rag-knowledge-flow` skill was created to standardize their use.

However, the built-in `search_semantic` suffers from the same vector-search imprecision, and the separate `opencode-rag mcp` server (the `krypto-agent` entry in `opencode.json`) was never fully removed — it remains as a vestigial MCP server alongside the built-in tools.

### What is Graphify

[Graphify](https://github.com/Graphify-Labs/graphify) is an open-source AI coding assistant skill that builds a **deterministic knowledge graph** from code using tree-sitter AST parsing. Key characteristics:

- **Deterministic:** No LLM involved in code analysis — pure tree-sitter AST parsing
- **Structural:** Builds a graph of files, functions, classes, imports, and their relationships
- **Queryable:** Agents query the graph instead of doing vector similarity search
- **Local:** Runs entirely on the developer's machine, no external service
- **Lightweight:** Install via `uv tool install graphifyy`, register via `graphify install`
- **Outputs:** `graph.html` (interactive visualization), `GRAPH_REPORT.md` (summary), `graph.json` (machine-readable data)
- **Stars:** 85k+ on GitHub, YC S26

Graphify addresses the "wrong files" problem by providing **explicit structural relationships** rather than fuzzy semantic similarity. When an agent asks "what does this function call?" or "where is this symbol defined?", Graphify returns exact graph paths with file:line citations.

### Current State of Configuration

The `opencode.json` file currently has:

```json
"krypto-agent": {
  "type": "local",
  "command": ["opencode-rag", "mcp"],
  "cwd": "/home/kiffer/project/krypto-agent"
}
```

And plugin entries:

```json
"plugin": [
  ".opencode/plugins/rag-plugin.js",
  ".opencode/plugins/rag-tui.js"
]
```

The plugin files do **not exist** in `.opencode/plugins/`. They are dead configuration entries.

### Key Forces

1. **Vector search imprecision.** Semantic similarity conflates conceptually related but structurally unrelated code. A search for "error handling" might find error message strings instead of try/catch blocks or error handler functions.

2. **Redundant MCP server.** The `krypto-agent` MCP server running `opencode-rag mcp` duplicates functionality that OpenCode's built-in `search_semantic` already provides. Maintaining a separate process has operational cost (startup, memory, port allocation) with no incremental benefit.

3. **Dead plugin references.** `rag-plugin.js` and `rag-tui.js` are listed in config but don't exist on disk. These accumulate config debt and may cause warnings or errors during OpenCode startup.

4. **Structural understanding matters for code.** Developers (and AI agents) need to know not just what code is about, but how it connects. Call graphs, import chains, class hierarchies, and symbol definitions are structural questions that vector search answers poorly.

5. **Graphify is deterministic and local.** No LLM calls, no embedding API, no external service. Tree-sitter parsing is fast, deterministic, and language-aware. The knowledge graph is reproducible across runs.

6. **Built-in tools are platform-level primitives.** OpenCode's `search_semantic`, `find_usages`, `get_file_skeleton`, and `describe_image` are part of the OpenCode core and cannot be removed. They continue to exist at the platform level but are not part of KodeHold's documented retrieval workflow — agents use Graphify exclusively for code retrieval.

7. **Learning curve.** Teams must learn a new query paradigm (graph queries vs. semantic search) and integrate it into their workflows.

## Decision

Replace the separate `opencode-rag mcp` server with **Graphify knowledge graph** as the sole code retrieval mechanism. The built-in OpenCode RAG tools (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) are platform-level primitives that cannot be removed from OpenCode, but they are NOT part of KodeHold's documented retrieval workflow — agents use Graphify for all code retrieval.

### What Changes

| Component | Current | New |
|-----------|---------|-----|
| `krypto-agent` MCP server (opencode-rag mcp) | Runs as MCP server | **Removed** from `opencode.json` |
| `rag-plugin.js` reference | Listed in `opencode.json` | **Removed** (file doesn't exist) |
| `rag-tui.js` reference | Listed in `opencode.json` | **Removed** (file doesn't exist) |
| Graphify installation | Not present | `uv tool install graphifyy` + `graphify install` |
| Code retrieval | `search_semantic` (vector search) | Graphify knowledge graph query (sole method) |
| Agent instructions | Use `search_semantic` for code queries | Use Graphify for all code retrieval |
| `opencode-rag-knowledge-flow` skill | `search_semantic` primary | **Replaced** by `graphify-knowledge-flow` skill (Graphify sole method) |

### Architecture

```
Agent asks code question
  │
  ├── Graphify knowledge graph (CODE RETRIEVAL)
  │    └── graphify query "function X dependencies"
  │        └── returns graph path with file:line citations
  │
  └── opencode-mem (RUNTIME MEMORY)
       └── search_memories(query="what we learned", scope="project")
```

**Relationships:**

| Layer | What it retrieves | Strength | Weakness |
|-------|------------------|----------|----------|
| Graphify | Structural code relationships (callers, callees, imports, class hierarchy) | Exact, deterministic, navigable | Requires graph regeneration on file changes |
| opencode-mem | Runtime learnings, session context | Cross-session persistence | Only what was explicitly stored |

### What We Keep

The following **remain unchanged**:

- `opencode-mem` — separate MCP server for persistent memory (per ADR-0051)

**Note on built-in OpenCode RAG tools:** The built-in tools (`search_semantic`, `find_usages`, `get_file_skeleton`, `describe_image`) are platform-level primitives that are part of the OpenCode runtime and cannot be removed. They continue to exist at the platform level but are **not** part of KodeHold's documented retrieval workflow. All KodeHold agents use Graphify for code retrieval.

### What We Remove

- `krypto-agent` MCP server entry from `opencode.json` (the `opencode-rag mcp` command)
- `.opencode/plugins/rag-plugin.js` reference from `opencode.json` (file doesn't exist)
- `.opencode/plugins/rag-tui.js` reference from `opencode.json` (file doesn't exist)

## Migration Plan: 7 Phases

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4 ──→ Phase 5 ──→ Phase 6 ──→ Phase 7
(ADR)      (Config     (Install    (AGENTS.md) (Skill      (Agent      (Verification)
            cleanup)     Graphify)               update)     files)
```

### Phase 1: Write ADR and Update Design Doc (Current Phase)

**Deliverable:** This ADR file written to `docs/adr/ADR-0054-replace-opencode-rag-with-graphify.md`.

**Gate:** ADR written, ADR index updated, design doc §5 (ADR Index) updated.

### Phase 2: Clean up opencode.json

**Actions:**
1. Remove the `krypto-agent` MCP server entry from `opencode.json`
2. Remove the `rag-plugin.js` and `rag-tui.js` entries from the `plugin` array in `opencode.json`
3. If the `plugin` array becomes empty, optionally remove it entirely or leave as empty array

**Verification:**
- `grep -c "krypto-agent" opencode.json` returns 0
- `grep -c "rag-plugin\|rag-tui" opencode.json` returns 0
- `opencode` starts without errors related to missing plugins or MCP servers

### Phase 3: Install Graphify and Register with OpenCode

**Actions:**
1. Install Graphify: `uv tool install graphifyy` (documented here; actual execution delegated to Engineers)
2. Register with OpenCode: `graphify install` (registers the skill and adds AGENTS.md instructions)
3. Build initial knowledge graph: `graphify build` (parses entire workspace)
4. Verify installation: `graphify status` returns healthy

**Prerequisites:**
- `uv` must be installed (Python package manager)
- Python 3.10+ must be available

**Verification:**
- `which graphify` returns a valid path
- `graphify status` shows project indexed
- `graph.html`, `GRAPH_REPORT.md`, and `graph.json` exist in project root

### Phase 4: Update AGENTS.md Instructions

**Actions:**
1. Replace the OpenCode RAG Knowledge Flow section with a Graphify Knowledge Flow section
2. Document Graphify query syntax as the sole code retrieval method

**Before (current):**
```
### OpenCode RAG Knowledge Flow (Pre-task Mode)
Before starting work, search the indexed codebase:
- `search_semantic(query="<topic>", topK=5)` for relevant code chunks
- `find_usages(symbolName)` for symbol references
- `get_file_skeleton(filePath)` for file structure
```

**After (updated):**
```
### Graphify Knowledge Flow (Pre-task Mode)
Before starting work, retrieve code context via Graphify:

- `graphify query "function X dependencies and usages"` for structural relationships
- `graphify query "class hierarchy of Y"` for type relationships
- `graphify query "files that import Z"` for dependency analysis
- `graphify query "definition of symbol X"` for symbol definitions
```

### Phase 5: Replace opencode-rag-knowledge-flow with graphify-knowledge-flow Skill

**Actions:**
1. Create new `.opencode/skills/graphify-knowledge-flow/SKILL.md` using Graphify as the sole code retrieval method
2. Archive or remove the old `opencode-rag-knowledge-flow` skill
3. Update all cross-references from `opencode-rag-knowledge-flow` to `graphify-knowledge-flow`

**Decision tree for code retrieval (new skill):**
```
| Question type | Use | Example |
|--------------|-----|---------|
| "What does this function call?" | Graphify query | `graphify query "callers of validate_user"` |
| "Where is this symbol defined?" | Graphify query | `graphify query "definition of parse_config"` |
| "What imports this module?" | Graphify query | `graphify query "files importing auth"` |
| "How does error handling work?" | Graphify query | `graphify query "error handling patterns"` |
| "Find something similar to X" | Graphify query | `graphify query "code like validate_user but for tokens"` |
```

### Phase 6: Update Agent Files

**Actions:**
1. Update `director.md` pre-flight knowledge section to include Graphify queries
2. Update individual agent files (architects.md, engineers.md, reviewers.md, testers.md, fls.md, scribes.md) knowledge flow sections
3. Ensure all agents understand the priority: Graphify → memory (both for code retrieval; memory is for runtime context)

**director.md Pre-flight Knowledge Search (updated):**
```
1. Graphify query — for structural code understanding:
   graphify query "<topic> dependencies and references"
2. search_memories — for runtime context:
   search_memories(query="<delegation-topic> <team> lessons bugs", scope="project")
```

### Phase 7: Final Verification

**Actions:**
1. Verify no `opencode-rag` MCP server runs at startup
2. Verify Graphify graph is queryable
3. Run a test query through Graphify
4. Verify no stale plugin references exist
5. Verify all agents reference Graphify (not opencode-rag) for code retrieval

**Gate:**
- `grep -r "opencode-rag" opencode.json` returns 0
- `grep -r "rag-plugin\|rag-tui" opencode.json` returns 0
- `graphify status` returns healthy with indexed files
- All agents load without errors
- No remaining references to `opencode-rag-knowledge-flow` skill in agent definitions

## Consequences

### Positive

- ✅ **Exact structural queries** — Graphify returns precise file:line references for symbols, functions, classes, and imports. No more "found wrong file."
- ✅ **Deterministic results** — Same codebase always produces same graph. No embedding model drift or vector search randomness.
- ✅ **No separate MCP server** — Removing the `krypto-agent` entry eliminates a redundant process. One less daemon to manage.
- ✅ **Dead config cleanup** — Stale plugin references removed from `opencode.json`.
- ✅ **Language-aware parsing** — Tree-sitter understands syntax, not just text. Graphify distinguishes between a function definition, a function call, and a class method.
- ✅ **Interactive visualization** — `graph.html` provides a browsable graph of the codebase, useful for both agents and human developers.
- ✅ **Zero LLM cost for retrieval** — Graphify uses no LLM calls for its analysis. Only the final query interpretation by the agent uses tokens.
- ✅ **Simplified workflow** — Single retrieval method (Graphify) instead of layered fallback. Less decision overhead for agents.

### Negative

- ❌ **Graph regeneration on code changes** — The knowledge graph must be rebuilt when files change. This adds a step to the development workflow (or requires a file watcher).
- ❌ **Extra dependency** — `graphifyy` via `uv` is an additional Python tool dependency. Adds to setup time for new environments.
- ❌ **Learning curve** — Agents must learn to formulate graph queries vs. natural language search. The query syntax is different from `search_semantic`.
- ❌ **Build time for large codebases** — Tree-sitter parsing of very large repositories may take seconds to minutes. The graph must be current for accurate results.
- ❌ **No cross-repository search** — Graphify builds a graph per workspace. Cross-project queries require separate graph builds.

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Graphify becomes unmaintained | Low | Medium | Graphify is well-established (85k+ stars, YC S26). If truly abandoned, migration path is re-evaluated. |
| Graph regeneration is forgotten | Medium | Low | Add `graphify build` to pre-flight checklist or CI pipeline. The agent detects stale graphs and rebuilds. |
| Query syntax differs from expected | Medium | Low | Agents adapt through instruction updates. Graphify queries are natural language and flexible. |
| Tree-sitter parser gaps for niche languages | Low | Medium | Graphify only supports languages with tree-sitter grammars. Limited support possible via `find_usages` at platform level. |
| Plugin cleanup breaks OpenCode | Low | High | The plugin files don't exist — removing their references cannot break functionality. Verify after cleanup. |
| Graphify pre-v1.0 API instability | Low | Low | Graphify queries are simple CLI commands; API changes would affect `graphify install` registration, not query syntax. |

## Alternatives Considered

### 1. Keep opencode-rag but improve embeddings

Replace the embedding model (e.g., switch from a general model to a code-optimized model like `codebert` or `starcoder-embedding`) to improve retrieval accuracy.

**Rejected because:** Better embeddings still don't provide structural understanding. The fundamental problem is vector search imprecision for structural queries, not embedding model quality. Even perfect semantic embeddings can't answer "what calls this function?" — only structural analysis can.

### 2. Use Sourcegraph

Sourcegraph provides code search with structural awareness and cross-repository search.

**Rejected because:** Sourcegraph requires either a cloud account or a self-hosted instance. It adds significant infrastructure complexity (PostgreSQL, frontend, indexing service) that contradicts KodeHold's principle of minimizing infrastructure (per ADR-0050).

### 3. Use ripgrep + grep exclusively

Replace all code retrieval with regex-based search tools (`rg`, `grep`, `ag`).

**Rejected because:** Regex can't answer semantic questions ("how does error handling work?") or structural questions ("what are the transitive dependencies of this function?"). It's useful for exact pattern matching but insufficient for code understanding.

### 4. Use OpenCode RAG exclusively (status quo)

Keep the current architecture with `search_semantic` as primary and `find_usages` for symbol lookup.

**Rejected because:** The "finds wrong files" problem persists. Agents waste time navigating to semantically similar but structurally unrelated code. `find_usages` provides symbol references but doesn't build a navigable graph of relationships.

### 5. ✅ Graphify knowledge graph (CHOSEN)

Local, deterministic, tree-sitter-based knowledge graph with zero LLM cost for analysis. Structural understanding without infrastructure overhead.

**Selected because:** It directly addresses the "wrong files" problem with structural queries, requires minimal setup (`uv tool install graphifyy`), and provides a deterministic, standalone structural code retrieval solution.

## Review Notes

- **2026-07-14:** Initial version. ADR follows the Nygard format consistent with ADR-0050 and ADR-0053.

## References

- Graphify Repository: https://github.com/Graphify-Labs/graphify
- Graphify Documentation: https://github.com/Graphify-Labs/graphify#readme
- ADR-0050: Agentmemory → OpenCode RAG Migration (`docs/adr/ADR-0050-agentmemory-to-opencode-rag-migration.md`)
- ADR-0051: opencode-mem as KodeHold Persistent Memory Backend (`docs/adr/ADR-0051-opencode-mem-persistent-memory.md`)
- Design Doc §7.2: Persistent Memory & Knowledge Retrieval (`docs/design/README.md`)
- opencode-rag-knowledge-flow Skill (`.opencode/skills/opencode-rag-knowledge-flow/SKILL.md`)
- opencode.json (root configuration)

## Documentation

Per ADR-0048 §3 — mandatory tool documentation for ADRs that adopt a new tool.

### Tool Overview

| Field | Value |
|-------|-------|
| **Tool** | Graphify (Graphify-Labs/graphify) |
| **Official docs** | https://github.com/Graphify-Labs/graphify#readme |
| **Version documented** | v0.9.9 (latest release as of 2026-07-14) |
| **Key sections read** | README.md (installation, quickstart, CLI commands, query syntax, OpenCode integration), ARCHITECTURE.md (tree-sitter AST parsing, Leiden clustering, graph structure) |
| **Key API concepts** | `/graphify .` — build graph in current directory; `graphify query "<question>"` — query the knowledge graph; `graphify path <nodeA> <nodeB>` — trace connections between two nodes; `graphify explain <node>` — explain a node with its full context; `graphify install --platform opencode` — register with OpenCode via AGENTS.md; `graphify build` — rebuild graph from scratch; `graphify status` — check graph health; `graphify hook install` — install git hook for auto-rebuild |
| **Configuration prerequisites** | Python 3.10+; `uv` tool manager (recommended) or `pipx`; `uv tool install graphifyy` installs the CLI; `graphify install` registers skill with OpenCode |
| **Gotchas** | Tool is pre-v1.0 (v0.9.9) — API may evolve; graph must be rebuilt (`graphify build`) on code changes; only languages with tree-sitter grammars are supported; `graphify-out/` directory (~10-50MB for medium projects) should be added to `.gitignore`; `graphify install` modifies AGENTS.md — verify changes after installation |