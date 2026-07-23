---
status: Deprecated
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0004: ICM and RTK Integration Strategy

## Status

Deprecated

ICM replaced by agentmemory per ADR-0029 (ICM → Agentmemory Migration Strategy). RTK token optimization remains valid independently.

## Context

KodeHold must maintain persistent context across sessions and minimize token consumption. Two tools are already installed and available:
- **ICM** (Infinite Context Memory) v0.10.50 — persistent memory with vector embeddings, FTS5 search, concept extraction, and session tracking
- **RTK** (Runtime Toolkit) v0.40.0 — CLI proxy that produces compact, token-optimized output for all standard tools

Both must be deeply integrated into the orchestrator's workflow. Without intentional integration, they become optional extras that teams skip, defeating their purpose.

## Decision

### ICM Integration

**Mandatory usage. No team bypasses ICM.**

1. **Session Tracking** — Every KodeHold session is logged as an ICM session. Scribes are responsible for session initialization.
2. **Memory Store** — All project decisions, design rationale, and important context are stored as ICM memories with keywords, summaries, and importance scores.
3. **Concept Extraction** — After each project phase (design, implementation, review, close), Scribes run concept extraction to build cross-project knowledge.
4. **Vector Retrieval** — When reopening a project, Scribes query ICM by semantic similarity to reconstruct full context.
5. **Token-Aware Storage** — Memories are stored with both full text and summaries. Summaries are used for context loading in light mode (see ADR-0005).

Configuration in `.icm/config.toml` (central database in kodehold root):
```toml
[store]
path = ".icm/memories.db"

[embedding]
dimensions = 768
```

### RTK Integration

**Mandatory CLI proxy. All file and git operations go through RTK.**

1. **File operations**: `rtk ls`, `rtk read`, `rtk tree`, `rtk find`
2. **Content search**: `rtk grep` (replaces raw grep)
3. **Git operations**: `rtk git status`, `rtk git diff`, `rtk git log`
4. **Format**: All output uses `--format compact` to minimize tokens

### Interaction Pattern

When loading project context:
1. Director requests context from Scribes
2. Scribes query ICM with `icm query` for relevant memories
3. Scribes use RTK to read current project files
4. Context is assembled and summarized before being passed to teams
5. Teams receive only the relevant subset, not the full project

## Consequences

- Positive: ICM ensures zero context loss across sessions and projects
- Positive: RTK reduces token consumption by 40-60% on all file operations
- Positive: Scribes team has a clear, bounded responsibility
- Negative: Dependency on two external tools — ICM and RTK must remain installed
- Negative: RTK output, while compact, may omit detail that a full read would show
- Neutral: Both tools are already installed and versioned; no additional setup cost
