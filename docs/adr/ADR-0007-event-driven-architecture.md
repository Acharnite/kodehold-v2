---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0007: Event-Driven Architecture over Cron Polling

## Status

Proposed

## Context

KodeHold v1.0 uses 7 cron-based loop patterns:

| Pattern | Schedule | What it does |
|---------|----------|-------------|
| daily-triage | 09:00 daily | Check repo health, uncommitted files, PRs, issues |
| pr-babysitter | Every 30 min | Check open PRs for staleness, conflicts |
| ci-sweeper | Every 15 min | Check CI status on latest commit |
| dependency-sweeper | 06:00 weekly | Check for outdated dependencies |
| changelog-drafter | On release | Draft CHANGES.md from commits |
| post-merge-cleanup | On merge | Clean up feature branches |
| issue-triage | Every 2 hours | Label and prioritize new issues |

Each pattern polls regardless of whether anything changed:

```
if nothing_happened():
    sleep(until_next_poll)   ← wasted
else:
    do_work()
```

Problems:

1. **Latency**: A PR pushed at 09:01 waits until 09:30 for the pr-babysitter.
2. **Waste**: 99% of cron ticks find nothing to do.
3. **Complexity**: `scripts/workspace.py` has ~200 lines of cron management.
4. **Overlap**: A long-running triage can overlap with the next cron tick.
5. **Locality**: Cron only works on the machine where it's installed. Remote
   events (GitHub PR, issue creation) require polling the API.

## Decision

Replace cron-based polling with event-driven dispatch.

### Event Sources

| Source | Type | Use Case |
|--------|------|----------|
| GitHub webhook | Remote | push, PR, issue, release events |
| GitLab webhook | Remote | push, MR, issue events |
| inotify | Local | File changes in workspace |
| Message queue (NATS) | Internal | Inter-component events |
| Schedule (fallback) | Local | Optional cron for legacy/offline use |

### Event Flow

```
Webhook ──▶ HTTP Server ──▶ Authenticate ──▶ Validate ──▶ Enqueue
                                                              │
inotify ──▶ File Watcher ──▶ Debounce ──▶ Enqueue ───────────┤
                                                              │
                                                     ┌────────▼────────┐
                                                     │   Event Queue    │
                                                     │   (NATS/Redis)   │
                                                     └────────┬────────┘
                                                              │
                                              ┌───────────────▼───────────────┐
                                              │       Event Router            │
                                              │                               │
                                              │ push → pipeline:full          │
                                              │ PR opened → pipeline:verify   │
                                              │ issue labeled → dispatch:triage│
                                              │ file changed → dispatch:lint  │
                                              └───────────────────────────────┘
```

### kodehold.yaml Event Configuration

```yaml
events:
  sources:
    github:
      webhook_secret: ${GITHUB_WEBHOOK_SECRET}
      events: [push, pull_request, issues, release]
    local:
      watch:
        - path: src/
          events: [modify, create, delete]
        - path: tests/
          events: [modify, create]
    schedule:            # Optional cron fallback
      - cron: "0 9 * * *"
        action: daily_triage

  routing:
    - match: {source: github, event: push}
      pipeline: full
    - match: {source: github, event: pull_request, action: opened}
      pipeline: verify
    - match: {source: local, path: src/**}
      pipeline: verify
      debounce: 5s
    - match: {source: schedule, cron: "0 9 * * *"}
      dispatch: daily_triage
```

### Debouncing

Local file watchers debounce by 5 seconds. Multiple rapid file saves (e.g.,
`cargo fmt` touching 10 files) trigger one pipeline run, not 10.

### Idempotency

Every event has a UUID. The event queue deduplicates by UUID. A webhook
redelivery doesn't trigger a duplicate pipeline run.

## Consequences

### Positive

- Sub-second response to events (vs. 15-30 min cron latency)
- Zero polling waste
- Remote events (GitHub) work without API polling
- Local events trigger instantly on file save
- Queue decouples event ingestion from processing (surge protection)
- Self-contained: no external cron daemon dependency

### Negative

- Requires a running daemon (`kodehold listen`) — not just cron
- Webhook setup: each workspace needs GitHub webhook configuration
- Queue dependency: NATS or Redis must be running (or embedded)
- Offline mode: without a network, remote events are missed
- Complexity: event routing rules can become hard to debug

### Risks

- **Webhook secret management**: Secrets must be provisioned per workspace.
  Mitigation: `kodehold init` auto-generates secrets, stores in `.git/kodehold/`.
- **Queue persistence**: If the queue process dies, events are lost.
  Mitigation: NATS JetStream or Redis Streams for persistent queues.

### Migration

V1.0 cron entries are preserved during migration. A `kodehold migrate-events`
command reads existing crontab, generates equivalent `kodehold.yaml` event
rules, and offers to remove the cron entries after verification.
