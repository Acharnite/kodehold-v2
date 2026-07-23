---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0015: Director Delegation Enforcement via Tool Permissions

## Status

Accepted

## Context

KodeHold's architecture depends on the Director orchestrating work by delegating to teams via the Task tool. However, the Director model (especially smaller models) frequently overrides this protocol and implements code directly instead of delegating. This undermines the entire team-based workflow.

Without enforcement:

- The Director implements features directly, bypassing Architects, Engineers, Testers, and Reviewers
- Quality gates are skipped — no testing, no code review, no design validation
- The team model becomes a suggestion rather than a constraint
- Delegation protocols documented in AGENTS.md and director.md are ignored at runtime

The key forces are:

- Enforcement must be runtime (detected by OpenCode's permission system) and structural (task-only workflow)
- The Director must retain enough bash access to run gate.sh, workspace.sh, ICM commands, and git status/log/diff
- All other bash commands must require user approval (the `*: ask` pattern) to prevent the Director from running arbitrary scripts
- The model must still be able to read files, search, and use skills — denying everything breaks the orchestration role
- The approach must work without framework-level enforcement (OpenCode does not structurally block tool misuse)

## Decision

We implement a three-layer enforcement strategy using OpenCode's existing permission system, combined with prompt-level protocols:

### Layer 1: opencode.json Tool Permissions

The Director agent's permissions in `opencode.json` are configured as follows:

| Tool | Permission | Rationale |
|------|-----------|-----------|
| `write` | `deny` | Director must not create or modify files directly |
| `edit` | `deny` | Director must not modify code or documents directly |
| `task` | `allow` | Primary delegation mechanism — the only way to make changes |
| `read` | `allow` | Must read design docs, ADRs, project files to orchestrate |
| `glob` | `allow` | Must find files to understand project structure |
| `grep` | `allow` | Must search code to understand context |
| `skill` | `allow` | Must load skills (state-awareness, investigate, etc.) |
| `bash` | pattern-based | Whitelist only safe commands, everything else requires approval |

### Bash Whitelist

| Pattern | Purpose | Why allowed without approval |
|---------|---------|------------------------------|
| `gate.sh --status` | Check current state | Read-only, safe |
| `gate.sh --transition` | Execute state transitions | Core orchestration function |
| `workspace.sh` | Workspace management | Core orchestration function |
| `icm` | ICM memory operations | Core orchestration function |
| `git status` | Check repository state | Read-only, safe |
| `git log` | View commit history | Read-only, safe |
| `git diff` | View changes | Read-only, safe |

All other bash commands use `*: ask` — the user must approve before execution. This prevents the Director from running arbitrary scripts, npm commands, or other tools that would constitute direct implementation.

### Layer 2: Triage-Check Protocol

The Director's system prompt includes a mandatory triage check:

> **Before ANY action, the Director must answer: "Is this a triage task?"**

| Signal | Delegate to |
|--------|------------|
| Bug / error / "fix this" | `fls` |
| Feature request | `architects` → `engineers` |
| Design question | `architects` |
| Test failure | `engineers` → `testers` |
| Read-only question | Answer directly |
| Gate / ICM / git read | Execute directly |
| Documentation | `scribes` |

This is a prompt-level constraint — the Director must consciously route work rather than doing it.

### Layer 3: Delegation Examples

The system prompt includes six concrete delegation examples mapping common triggers to responsible teams:

| Trigger | Task tool subagent_type | Sequence |
|---------|------------------------|----------|
| Design/ADR | `architects` → `scribes` | Post-task documentation |
| Implementation | `engineers` → `scribes` | Post-task documentation |
| Investigate/Debug | `engineers` or `fls` via investigate skill → `scribes` | Root cause first, fix second |
| Test | `testers` → `scribes` | Must finish before review |
| Review | `reviewers` → `scribes` | Must run after tests pass |
| Memory/Docs | `scribes` | — |

### Enforcement Characteristics

| Aspect | Mechanism | Level |
|--------|-----------|-------|
| File creation/modification | `write: deny`, `edit: deny` | Runtime (OpenCode) |
| Arbitrary bash | `*: ask` pattern | Runtime (OpenCode) |
| Delegation routing | Triage-check + examples | Structural (prompt) |
| Task-only workflow | Task tool is `allow`, everything else blocks | Runtime (OpenCode) |

### Validation

The test script `tests/init/04-director-permissions.sh` validates all 9 permission assertions:

1. `write` = `deny`
2. `edit` = `deny`
3. `task` = `allow`
4. `read` = `allow`
5. `glob` = `allow`
6. `grep` = `allow`
7. `skill` = `allow`
8. `bash` patterns match whitelist
9. Default bash = `*: ask`

### Limitation

OpenCode source investigation confirmed: delegation is not technically enforced at runtime — the model can still call tools that bypass permissions in certain edge cases. The enforcement relies on agent cooperation (the prompt-level layers) combined with runtime blocking (the permission system). This is sufficient because the permission layer catches the most common failure modes (direct file writes/edits) while the prompt layers handle routing decisions.

## Consequences

- Positive: Director can no longer directly create or modify files — must delegate via Task tool
- Positive: Bash whitelist prevents arbitrary command execution while preserving orchestration capabilities
- Positive: Triage-check protocol forces conscious routing decisions before action
- Positive: Delegation examples provide concrete patterns the model can follow
- Positive: Test script validates all 9 permission assertions on every init
- Negative: Enforcement is not 100% airtight — prompt-level layers can be bypassed by large context models
- Negative: Bash whitelist may need updating as new orchestration commands are added
- Neutral: `*: ask` on bash means the Director may prompt for approval on safe commands not in the whitelist — acceptable trade-off for safety
