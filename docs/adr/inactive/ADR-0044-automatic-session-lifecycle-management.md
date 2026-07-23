# ADR-0044: Automatic Session Lifecycle Management in agentmemory-capture Plugin

## Status

Deprecated

**Version:** 1.2
**Last Updated:** 2026-06-06

## Context

### The Problem: Stale Active Sessions

Agentmemory sessions remain in "active" status indefinitely after an OpenCode session ends. OpenCode does not reliably fire `session.deleted` when a user archives or closes a session — the `session.deleted` event handler in the agentmemory-capture plugin (line 302-317) exists but is rarely triggered in practice.

This causes three problems:

1. **Stale summary leakage.** When a new session starts, agentmemory's `/session/start` endpoint may return summaries from stale active sessions, polluting the new session's context with irrelevant data.

2. **Resource waste.** Active sessions accumulate in the agentmemory database. Each active session holds observation data, file stashes, and tool call tracking maps in the plugin's in-memory state (`stashedFiles`, `seenSubtaskIds`, `seenToolCallIds`). These maps are never cleaned up for sessions that are never explicitly ended.

3. **Cron dependency.** The current mitigation is a cron-based cleanup script (`scripts/agentmemory-session-cleanup.sh` + `scripts/agentmemory-session-cleanup.py`) that polls agentmemory every N minutes, finds sessions idle for >60 minutes, and calls `POST /agentmemory/session/end` for each. This is an external workaround — not a proper lifecycle solution.

### Current Architecture

**Cron-based cleanup (being replaced):**

```
┌─────────────┐     poll every N min     ┌──────────────────┐
│  cron job   │ ──────────────────────►  │ cleanup script   │
│ (systemd)   │                          │ (bash + python)  │
└─────────────┘                          └────────┬─────────┘
                                                   │
                                                   │ POST /session/end
                                                   ▼
                                           ┌──────────────────┐
                                           │  agentmemory     │
                                           │  daemon          │
                                           └──────────────────┘
```

*Note: The cron job triggers the cleanup script, which then polls agentmemory's active sessions and calls `/session/end` for each idle session found.*

**Problems with the cron approach:**

| Issue | Detail |
|-------|--------|
| **External dependency** | Requires systemd timer or crontab configuration. Not portable across environments. |
| **Polling latency** | Sessions can remain stale for up to N minutes (typically 5-15 min between cron runs). |
| **ARG_MAX crash** | The Python engine previously crashed with "Argument list too long" when sessions JSON exceeded environment variable size limits (fixed in a later iteration by fetching directly via URL, but the fragility existed). |
| **No process awareness** | The cron job cannot distinguish between "session is idle because the user walked away" and "session is idle because the OpenCode process died." Both are treated the same. |
| **No exit handling** | If the OpenCode process crashes or is killed, the cron job eventually cleans up — but there is a window where the session remains active. |
| **Cross-process interference** | The cron job operates on ALL active sessions, not just sessions from the current OpenCode process. It could theoretically end a session that is still in use by another process. |

### The agentmemory-capture Plugin

The agentmemory-capture plugin (`~/.config/opencode/plugins/agentmemory-capture.ts`) is an OpenCode plugin that already handles session lifecycle events:

- **`session.created`** (line 209): Calls `POST /session/start` with the new session ID.
- **`session.deleted`** (line 302): Calls `POST /session/end` — but this event is rarely fired by OpenCode.
- **`session.status`** (line 251): Handles idle status by calling `POST /summarize`.
- **`session.updated`** (line 275): Observes session metadata changes.

The plugin already has the infrastructure to manage session lifecycle. The missing piece is automatic detection of session end conditions.

### Key Forces

1. **Reliability.** Session cleanup must happen reliably — not depend on a cron job that may not be configured.
2. **Timeliness.** Sessions should be ended promptly when they are no longer active, not after a polling delay.
3. **Process isolation.** Each OpenCode process manages its own sessions. No cross-process interference.
4. **No data loss.** Ending a session triggers agentmemory's reflection pipeline (`slot-reflect`, consolidation). This must happen before the process exits.
5. **Backward compatibility.** Existing active sessions must be handled. The transition from cron to in-plugin must be smooth.
6. **Simplicity.** The solution should be in the plugin itself — no new infrastructure, no new dependencies.

## Decision

Replace the cron-based session cleanup (`scripts/agentmemory-session-cleanup.sh` + `.py`) with automatic in-plugin session lifecycle management in the agentmemory-capture plugin. The plugin will handle three session-end mechanisms.

### Mechanism 1: Archive Detection (session.updated)

When a user archives a session in OpenCode, OpenCode fires `session.updated` with `time.archived` set in the event properties. The plugin already handles `session.updated` (line 275-286) but only observes metadata — it does not check for archival.

**Decision:** Extend the `session.updated` handler to detect `time.archived` and call `POST /session/end` immediately.

**Implementation sketch:**

```typescript
// In the session.updated handler (around line 275):
if (type === "session.updated") {
  const info = props.info as Record<string, unknown> | undefined;
  const sid = (info?.id as string) || props.sessionID || activeSessionId;
  if (!sid) return;

  // Detect archival — OpenCode sets time.archived when user archives a session
  if (info?.time && typeof info.time === "object" && (info.time as any).archived) {
    await post("/session/end", { sessionId: sid });
    if (sid === activeSessionId) activeSessionId = null;
    pruneSessionMaps(sid);
    startContextCache.delete(sid);
    contextInjectedSessions.delete(sid);
    return;
  }

  await observe(sid, "session_updated", { /* existing */ });
}
```

**Why this works:** Archival is the user's explicit signal that a session is done. This is the most reliable trigger — it fires immediately when the user archives, with no polling delay.

### Mechanism 2: Per-Process Idle Timer

Each OpenCode process maintains its own in-process 24-hour timer for its own `activeSessionId`. The timer resets on activity (tool calls, prompts, commands). After 24 hours of inactivity, the plugin calls `POST /session/end` for that specific session only.

**Decision:** Add a module-level idle timer that resets on every observed activity event and fires after 24 hours of inactivity.

**Implementation sketch:**

```typescript
// Module-level state
let idleTimer: ReturnType<typeof setTimeout> | null = null;
const IDLE_TIMEOUT_MS = 24 * 60 * 60 * 1000; // 24 hours

function resetIdleTimer(): void {
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = setTimeout(async () => {
    if (activeSessionId) {
      await post("/session/end", { sessionId: activeSessionId });
      const sid = activeSessionId;
      activeSessionId = null;
      pruneSessionMaps(sid);
      startContextCache.delete(sid);
      contextInjectedSessions.delete(sid);
    }
  }, IDLE_TIMEOUT_MS);
}

// Call resetIdleTimer() at the end of every event handler that processes
// user activity: prompt_submit, tool.execute.before, command.executed, etc.
```

**Activity events that reset the timer:**

| Event | Why |
|-------|-----|
| `chat.message` (prompt_submit) | User sent a prompt — clearly active |
| `tool.execute.before` | Agent is executing tools — active |
| `command.executed` | User ran a command — active |
| `session.status` (idle) | OpenCode reports idle — do NOT reset (this is the absence of activity) |

**Why 24 hours:** This is a safety net, not the primary cleanup mechanism. The primary mechanism is archive detection (instant) and process exit (instant). The 24-hour timer catches edge cases where:
- The user closes their laptop without archiving
- The OpenCode process is suspended (not killed)
- The user walks away for an extended period

**Why per-process:** Each OpenCode process has its own JavaScript runtime. The `idleTimer` variable is scoped to the plugin instance. There is no shared state between processes. No cross-process interference.

### Mechanism 3: Process Exit Handler

When the OpenCode process exits (SIGTERM, SIGINT), the plugin should call `POST /session/end` for the current `activeSessionId`.

**Decision:** Register process signal handlers that end the active session before exit.

**Implementation sketch:**

```typescript
// In the plugin factory function, after setting projectPath:
function registerExitHandlers(): void {
  const handleExit = async () => {
    if (activeSessionId) {
      await post("/session/end", { sessionId: activeSessionId });
      // Note: fire-and-forget — we cannot await in signal handlers
      // reliably, but the POST is synchronous enough for our purposes.
    }
  };

  process.on("SIGTERM", handleExit);
  process.on("SIGINT", handleExit);
  process.on("exit", handleExit);
}
```

**Why this matters:** Without exit handlers, a killed OpenCode process leaves the session active until the cron job (or now, the idle timer) cleans it up. With exit handlers, the session is ended immediately, triggering agentmemory's reflection pipeline (`slot-reflect`, consolidation) before the process dies.

**Caveat:** Signal handlers cannot reliably perform async operations. The `post()` function uses `fetch` with a 5-second timeout. In practice, this is sufficient for the agentmemory daemon (localhost, no network latency). If the daemon is down, the session remains active — the idle timer serves as a fallback.

### New Architecture

```
┌─────────────────────────────────────────────┐
│           OpenCode Process                   │
│                                              │
│  ┌─────────────────────────────────────┐     │
│  │  agentmemory-capture plugin         │     │
│  │                                     │     │
│  │  session.updated ──► archive detect │     │
│  │  idle timer (24h) ──► auto-end     │     │
│  │  SIGTERM/SIGINT  ──► exit handler  │     │
│  │                                     │     │
│  │         │ POST /session/end         │     │
│  └─────────┼───────────────────────────┘     │
└────────────┼─────────────────────────────────┘
             │
             ▼
  ┌──────────────────────┐
  │  agentmemory daemon  │
  │  (localhost:3111)    │
  │                      │
  │  slot-reflect runs   │
  │  consolidation runs  │
  │  crystals auto runs  │
  └──────────────────────┘
```

**What is removed:**

| Artifact | Action |
|----------|--------|
| `scripts/agentmemory-session-cleanup.sh` | Delete — no longer needed |
| `scripts/agentmemory-session-cleanup.py` | Delete — no longer needed |
| Systemd timer or crontab entry for cleanup | Remove from setup docs |

### Transition Strategy

1. **Deploy the plugin update** with all three mechanisms.
2. **Keep the cron job running** for one week (grace period).
3. **After one week**, verify no stale sessions accumulate.
4. **Remove cron job and cleanup scripts.**

During the grace period, both mechanisms operate simultaneously. The cron job may attempt to end sessions that the plugin already ended — this is safe (agentmemory's `/session/end` is idempotent for already-ended sessions).

## Consequences

### Positive

1. **Instant cleanup on archive.** Sessions are ended the moment the user archives them — no polling delay.

2. **Process exit safety.** SIGTERM/SIGINT handlers ensure sessions are ended even if the process crashes or is killed. The reflection pipeline runs before exit.

3. **No cron dependency.** Removes an external infrastructure dependency. The plugin is self-contained.

4. **No cross-process interference.** Each process manages its own sessions. The idle timer is per-process, not global.
5. **Cleaner codebase.** Two scripts (bash + python) removed. Logic lives in the TypeScript plugin where it belongs.

6. **Idempotent operations.** All `/session/end` calls are safe to repeat. The grace period with the cron job causes no harm.

### Negative

1. **Async limitations in signal handlers.** `SIGTERM`/`SIGINT` handlers cannot reliably perform async operations. If the agentmemory daemon is slow or down, the session may not be ended before the process exits. Mitigation: the idle timer (24h) serves as a fallback.

2. **Timer drift.** The 24-hour idle timer uses `setTimeout`, which can drift on a busy event loop. This is acceptable — the timer is a safety net, not a precision instrument. ±a few minutes on a 24-hour window is irrelevant.

3. **Plugin reload loses timer state.** If the plugin is reloaded (e.g., OpenCode restart), the idle timer is reset. The new plugin instance starts with a fresh 24-hour timer. This is acceptable — the previous session was already ended by the exit handler.

4. **Memory overhead.** The idle timer is a single `setTimeout` handle (~few bytes). Negligible.

5. **No cross-process session recovery.** If the OpenCode process is killed with SIGKILL (cannot be caught), the exit handler does not run. The session remains active until the idle timer would have fired — but the timer was in the killed process. Mitigation: this is an acceptable edge case. The cron job's grace period covers this during transition. Post-transition, manual cleanup via the agentmemory API is the fallback.

### Neutral

1. **The 24-hour idle timeout is arbitrary.** It could be 12h or 48h. 24h was chosen as a reasonable "definitely abandoned" threshold. This can be adjusted via a constant in the plugin.

2. **Archive detection depends on OpenCode firing `session.updated` with `time.archived`.** If OpenCode changes this behavior, the mechanism breaks. Mitigation: the idle timer and exit handler still provide coverage.

## Compliance

| Requirement | Mechanism | Verification |
|-------------|-----------|-------------|
| Session ends when user archives | Archive detection | Unit tests: 25 tests in `test/archive-detection.test.ts` covering happy path, negative case, and edge cases. All pass. Manual: archive a session, check agentmemory sessions list |
| Session ends after 24h inactivity | Idle timer | Manual: set IDLE_TIMEOUT_MS to 10s, wait, check sessions list |
| Session ends on process exit | Exit handler | Manual: SIGTERM the process, check sessions list |
| No cross-process interference | Per-process timers | Architectural: each plugin instance has its own scope |
| Cron job can be removed | All three mechanisms cover all cases | Verify no stale sessions after 1 week without cron |

## Notes

### Related ADRs

- **ADR-0029** (Agentmemory Migration Strategy) — established the migration from ICM to agentmemory. This ADR improves agentmemory session lifecycle management.
- **ADR-0043** (Agentmemory Slot Integration) — documented agentmemory's `slot-reflect` mechanism that runs on `session::stopped`. This ADR ensures `session::stopped` fires reliably by ending sessions promptly.
- **ADR-0031** (Actions + Crystals) — crystals auto-generation is triggered by `/session/end`. Reliable session ending improves crystal generation.

### Open Questions

1. **Should the idle timeout be configurable?** Currently hardcoded at 24h. Could be exposed via environment variable (`AGENTMEMORY_IDLE_TIMEOUT_MINUTES`) if needed. Deferred because: (a) the 24h default is a safety net, not a primary mechanism — archive detection and process exit handlers cover the vast majority of cases; (b) adding configuration adds maintenance burden (validation, documentation, migration) for a value that will almost never need tuning; (c) if a user needs a different timeout, they can change the constant in the plugin source — the code is local and editable.

2. **Should we also handle `session.deleted` more aggressively?** The existing handler (line 302) already calls `/session/end` on `session.deleted`. The problem is that OpenCode rarely fires this event. Archive detection (`session.updated` with `time.archived`) is the practical replacement.

3. **Should the exit handler attempt consolidation/crystals?** Currently, the exit handler only calls `/session/end`. Agentmemory's triggers handle `slot-reflect` and consolidation automatically on `session::stopped`. No additional calls needed.

### Deferred / Future Work

1. **SIGKILL orphan handling.** SIGKILL (signal 9) cannot be caught by any process handler. If the OpenCode process is killed with SIGKILL, the exit handler does not run and the session remains active. Mitigation during transition: the cron job's grace period covers this. Post-transition: manual cleanup via the agentmemory API is the fallback. If this becomes a frequent problem, a future enhancement could add a startup-time check that ends orphaned sessions from the same project — but this adds complexity (distinguishing "orphaned" from "still in use by another process") that is not justified by the rarity of SIGKILL in practice.

### Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.2 | 2026-06-06 | Archive detection verified with 25 unit tests in `test/archive-detection.test.ts`. Compliance table updated with unit test verification. |
| 1.1 | 2026-06-06 | Status promoted from Proposed → Accepted after Reviewers approval and second opinion. Clarified cron diagram with note that cron triggers cleanup script which polls agentmemory. Moved SIGKILL orphan question to new "Deferred / Future Work" section. Strengthened rationale for deferred configurable timeout (3 reasons: safety net role, maintenance burden, local editability). |
| 1.0 | 2026-06-06 | Initial proposal |