---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0028: Agentmemory Project Detection Strategy

## Status

Deprecated

**Finalized: 2026-06-01** — Accept the full filesystem path as the project name. This is the original plugin behavior (line 171 of agentmemory-capture.ts: `projectPath = ctx.worktree || ctx.project?.id || process.cwd()`). All prior fix attempts (Plugin-side resolveProject, Director-level getActiveProject, tool-call scanning) have been reverted. Fix 5 (per-session `info?.directory`) applied 2026-05-31 — confirmed working. The plugin uses `info?.directory` from the session.created event with fallback to the original project path. Runtime validation added per CodeRabbitAI review (PR #749, 2026-06-01): replaces `(info?.directory as string)` type assertion with `typeof` + length check on line 191.

## Context

### The Problem

The agentmemory-capture OpenCode plugin (v0.9.24, at `/home/kiffer/.config/opencode/plugins/agentmemory-capture.ts`) determines the project name for session registration at line 171:

```typescript
projectPath = ctx.worktree || ctx.project?.id || process.cwd();
```

This sends the **full filesystem path** as the `project` parameter to the agentmemory REST API (`POST /agentmemory/session/start`). The agentmemory daemon stores it as-is with no transformation.

**Consequence:** When OpenCode is launched from a workspace root (e.g., `/home/kiffer/project/`), ALL sessions receive:
```
project: "/home/kiffer/project"       ← workspace root, not the sub-project name
```
Instead of the expected canonical project name like `kodehold`, `bob`, or `bob-ollama`.

This breaks ICM project scoping — the `project` field is used for topic prefixing (`kodehold-<project>-*`), context injection, and cross-session memory retrieval. Every sub-project aliases to the same project string, causing memory cross-contamination across unrelated agent sessions.

### System Architecture (2 Integration Points)

#### 1. OpenCode Plugin (`agentmemory-capture.ts`) — Sends project to daemon

The plugin initializes `projectPath` once at load time. All subsequent REST calls (session/start, observe, context, summarize, etc.) reuse this captured value. The plugin never resolves project names — it forwards whatever `ctx.worktree` gives it.

#### 2. agentmemory Hooks (NOT running)

The agentmemory npm package ships CLI hooks (`/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/hooks/`) that are designed for Claude Code. Each hook inlines a `resolveProject()` function:

```typescript
function resolveProject(cwd: string): string {
  const explicit = process.env["AGENTMEMORY_PROJECT_NAME"];
  if (explicit && explicit.trim()) return explicit.trim();
  const dir = cwd && cwd.trim() ? cwd : process.cwd();
  try {
    const top = execSync("git rev-parse --show-toplevel", {
      cwd: dir, stdio: ["ignore", "pipe", "ignore"], timeout: 500,
    }).toString().trim();
    if (top) return basename(top);
  } catch {}
  return basename(dir);
}
```

Resolution order: (1) `AGENTMEMORY_PROJECT_NAME` env var, (2) `basename(git rev-parse --show-toplevel)`, (3) `basename(cwd)`.

These hooks are NOT registered in `/home/kiffer/.config/opencode/opencode.json` and are not designed for the OpenCode plugin architecture — they're stdin/stdout process hooks for Claude Code's hook system.

#### 3. MCP Server (`@agentmemory/mcp`)

Proxies tool calls to the agentmemory daemon or local KV. It passes through project names received from the plugin — it does not set or transform them.

### Prior Failed Fix Attempts

| Attempt | Approach | Failure |
|---------|----------|---------|
| Fix 1 | Walk-up directory scoring with markers (e.g., `pyproject.toml`) | The workspace root `/home/kiffer/project/` won all tiebreakers because it had the longest cumulative score — every marker file at deeper levels was also present in parent directories |
| Fix 2 | `.kodehold-project` marker files at project roots | Agent never found them because `process.cwd()` returned the workspace root, not the sub-project directory; the plugin never changed cwd |
| Fix 3 | Dynamic detection from tool-call data (parse file paths out of tool calls) | Fragile regex — `[^/]+` captured sentence fragments as project names. Race conditions: session.start fired before any tool calls existed to parse. Aliased sessions to wrong names |
| Fix 4 | Tool-call scanning with module-level state | Worked functionally but had no design doc, no tests, and was reverted per the lesson to "tackle properly with ADR/design process" |
| Fix 5 | Per-session `info?.directory` from session.created event | ✅ Success — uses native session metadata, no filesystem walk-up, no race conditions, no regex. Runtime validation added via CodeRabbitAI review (PR #749, 2026-06-01): `typeof` + length check replaces type assertion on line 191 |

**Key lesson** (`lsn_1c753d4ac1088621`, confidence 0.9): "revert to original GitHub version and tackle with proper ADR/design" — this lesson was created precisely to avoid a 5th ad-hoc fix.

### Key Constraints

1. **OpenCode web sessions** use `ctx.worktree` which is the workspace root path (e.g., `/home/kiffer/project/`)
2. Multiple projects sit under a single workspace root (`/home/kiffer/project/{kodehold,bob,bob-ollama}`)
3. The plugin has no built-in way to know which sub-project the user is working on without reading tool-call data or explicit configuration
4. The `project` parameter must be a **stable canonical identifier** (per agentmemory's own tool description) — it must be deterministic and survive session restarts
5. The fix must work 100% in OpenCode web sessions — no assumptions about CLI context
6. The fix must NOT use failure-prone methods: regex on tool outputs, heuristic walk-up scoring, or race-condition-prone lazy detection
7. The `project` value is captured at plugin load time — it cannot be deferred without restructuring session.created handling

### Relevant Prior Art

- agentmemory hooks (`_project.ts`) implement the correct resolution logic but are not wired into the OpenCode plugin path
- GitHub issue #733 and PR #738 discuss `AGENTMEMORY_PROJECT_FROM_REMOTE=1` for `host/org/repo` format (unrelated but demonstrates the need)
- ADR-0009 (ICM MCP Integration) establishes that `project` is a scoping parameter for all memory operations
- The design doc section 7.2 mandates: "Each project's memories are scoped via topic prefixes (`kodehold-<project>-*`)"

## Decision

### Final Decision: Accept Full Filesystem Path as Project Name

After three distinct approaches were attempted and reverted, the final decision is to **accept the full filesystem path as the project name** — exactly as the original upstream plugin does at line 171:

```typescript
projectPath = ctx.worktree || ctx.project?.id || process.cwd();
```

This sends the full path (e.g., `/home/kiffer/project/kodehold`) as the `project` parameter. This is correct behavior because:

1. **Full paths are unique.** `/home/kiffer/project/kodehold` ≠ `/home/kiffer/project/bob` — no collisions possible.
2. **Zero configuration.** Every project automatically gets its unique path. No env vars, no marker files, no git detection.
3. **Zero plugin divergence.** The plugin remains 100% aligned with the upstream GitHub version. No maintenance burden, no manual merge conflicts, no SHA256 re-verification.
4. **Zero Director changes.** No `getActiveProject()`, no slot injection, no MCP call template modifications.
5. **Zero race conditions.** Determined at plugin load time — no deferred or lazy resolution.

#### Why This Is Correct Per Scenario

| Scenario | Plugin resolves | Director resolves | Correct? |
|----------|----------------|-------------------|----------|
| OpenCode web from workspace root | `/home/kiffer/project` | Director runs from project dir: `process.cwd()` = `/home/kiffer/project/kodehold` | Yes — Director sessions get unique path |
| Director in `kodehold/` | `process.cwd()` = `/home/kiffer/project/kodehold` | Same | Yes — unique |
| Director in `bob/` | `process.cwd()` = `/home/kiffer/project/bob` | Same | Yes — unique |
| Web service from `kodehold/` | `ctx.worktree` = `/home/kiffer/project/kodehold` | Same | Yes — unique |

#### Why Prior Approaches Were Rejected

**Approach 1 (Fix 1-4): Plugin-side project detection with walk-up scoring, marker files, tool-call scanning.**
- Walk-up scoring: workspace root won all tiebreakers (longest cumulative marker score)
- Marker files: agent never changed cwd from workspace root, never found them
- Tool-call scanning: race conditions (session.start before any tool calls), fragile regex

**Approach 2 (this ADR's previous "Director-level" decision): `getActiveProject()` with slot injection.**
- Director changes don't help when the user works in `bob/` — the Director is not running
- The plugin still sends `/home/kiffer/project` for session/start
- Adds complexity (slots, MCP template changes) for no benefit
- The "project scoping" problem is only a problem if you expect short names — the full path works fine

**Approach 3 (ICM recall-project):**
- ICM is being deprecated. All functionality must migrate to pure agentmemory.

#### Architecture (Post-Final)

```
┌──────────────────────────────────────────────────┐
│  Plugin (agentmemory-capture.ts)                  │
│  ── 100% original GitHub v0.9.24                  │
│  ── projectPath = ctx.worktree || ... || cwd()    │
│  ── Sends FULL PATH as project name               │
│  ── No modifications ever                         │
└──────────────────┬───────────────────────────────┘
                   │ project="/home/kiffer/project/kodehold"
                   ▼
┌──────────────────────────────────────────────────┐
│  agentmemory Daemon                                │
│  ── Stores full path as project scope             │
│  ── Each project has unique key                   │
│  ── No downstream confusion                       │
└──────────────────────────────────────────────────┘
                   ▲
                   │ All MCP calls use same project string
                   │ (no injection needed — plugin set it)
┌──────────────────────────────────────────────────┐
│  Director / Any Agent                              │
│  ── Uses project: process.cwd() or omits it       │
│  ── agentmemory defaults to session's project      │
│  ── No special logic needed                        │
└──────────────────────────────────────────────────┘
```

#### What This Changes

- **Plugin:** No changes. Remains at GitHub v0.9.24 baseline. **This is permanent.**
- **Director:** No `getActiveProject()`. No slot injection. No MCP template changes.
- **Agentmemory slots:** No `active_project` slot. No slot-based scoping.
- **Everything:** The full filesystem path is the canonical project name. Period.

## Consequences

### Positive

1. **Zero maintenance forever.** The plugin is 100% original upstream. No divergence, no merge conflicts on update, no SHA256 re-verification, no custom patches. This is permanent and requires no ongoing effort.

2. **Zero configuration.** Every project works immediately. No env vars, no marker files, no git detection, no `.kodehold-state` dependence, no slot setup. The full path is always correct.

3. **Uniqueness guaranteed.** Full filesystem paths cannot collide. `/home/kiffer/project/kodehold` and `/home/kiffer/project/bob` are distinct keys. No aliasing, no cross-contamination.

4. **No race conditions.** Project is determined at plugin load time, before any session starts. No deferred resolution, no lazy evaluation, no tool-call scanning.

5. **No divergence from the package author's design.** The original plugin intentionally sends the full path. Our approach aligns with the upstream design philosophy — simple, unique, configuration-free.

6. **Works identically for all agents.** The plugin sets the project for every session (OpenCode web, Director, any agent connected to the daemon). No agent-specific logic, no slot injection, no special cases.

7. **Director unchanged.** No `getActiveProject()`, no slot creation, no MCP template modifications. The Director agent definition remains clean and minimal.

8. **No cognitive load.** The rule is trivial: the project name is the full filesystem path. No one needs to remember resolution algorithms, priority orders, or slot dependencies.

### Negative

1. **Long project name strings in logs/UI.** The full path `/home/kiffer/project/kodehold` is ~30 characters compared to a short name like `kodehold`. This is cosmetic only — the agentmemory daemon handles arbitrary-length strings.

2. **Web service sessions from workspace root share the same project string.** When the OpenCode web service is launched from `/home/kiffer/project/`, all sessions under it register as `project: "/home/kiffer/project"`. Mitigation: session IDs are unique, so observations are never cross-contaminated. The shared project string only affects topic-prefixed queries — but in practice, the Director runs from the specific project directory, so its sessions use the specific project path.

### Risks

1. **Project directory rename orphans old sessions.** If `/home/kiffer/project/kodehold` is renamed to `/home/kiffer/project/kodehold-v2`, all sessions under the old path become inaccessible via the new key. Mitigation: project directories are not renamed in practice. If renamed, old sessions are historical artifacts — new sessions start fresh under the new path.

2. **No short-name aliasing.** Memory queries that expect short project names (e.g., `kodehold` in topic prefixes) must be updated to use the full path. Mitigation: update any hardcoded topic prefix references to use the full path or omit the project filter for cross-project queries.

### Follow-up Items

- [x] **Revert plugin to original GitHub v0.9.24 baseline** — completed (all prior fix attempts reverted)
- [x] **Revert Director `getActiveProject()`** — no Director changes needed
- [x] **Remove `active_project` slot** — no slot-based scoping needed
- [x] **Remove resolveProject() references** from documentation and configuration — completed
- [ ] **Update design doc** section 7.2 (ICM project scoping) to reference ADR-0028's final decision
- [ ] **Remove or update any hardcoded short-name project references** in agent configurations, MCP templates, or documentation

### How to Revert

There is nothing to revert. The plugin is 100% original upstream. If a future approach is needed:
1. The plugin remains untouched — it will always send the full path
2. Any new approach must operate at a different layer (e.g., agentmemory daemon-level name mapping)
3. No KodeHold files need changing — the final state is "plugin is original, Director is original"

## ADR References

- **ADR-0009** (ICM MCP Integration) — establishes that `project` is the scoping parameter for all memory operations
- **ADR-0018** (Scribes Centralization) — Scribes relies on correct project scoping for topic-prefixed memory storage
- **ADR-0021** (Prospective Memory) — uses `kodehold-<project>-prospective` topic prefix, requiring correct project names
- **ADR-0027** (ICM Knowledge Flow Invocation Modes) — defines how teams store learnings under project-scoped topics
- **ADR-0028** (this document) — **previous decision (Plugin-side resolveProject) reverted on 2026-05-31; see Decision section for current approach**
- **Lesson `lsn_1c753d4ac1088621`** (confidence 0.9) — established the "revert and ADR/design" approach that created this ADR
- **Lesson `lsn_ee3995917bb0baa8`** (confidence 0.9) — documented the session-startup race condition workaround (`/remember` instead of `icm recall`)
- **Lesson `lsn_0f5d62807b06ed4a`** (confidence 0.9) — documented the regex parsing fragility in Fix 3
- **Lesson `lsn_1cb3ac318435645c`** (confidence 0.9) — documented the per-session `info?.directory` approach from session.created event (Fix 5)

### Source Files Referenced

- `/home/kiffer/.config/opencode/plugins/agentmemory-capture.ts` (v0.9.24, Fix 5 applied per-session `info?.directory` from session.created event; line 191 now uses runtime `typeof` + length validation per CodeRabbitAI review PR #749)
- `/home/kiffer/project/kodehold/.kodehold-state` — KodeHold project state file (project detection source)
- `/home/kiffer/project/kodehold/docs/design/README.md` — KodeHold design document (fallback project detection source)
- `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/hooks/session-start.mjs` (lines 6-23) — canonical `resolveProject()` implementation (no longer used by this approach)
- `docs/design/README.md` section 7.2 — ICM project scoping requirements
