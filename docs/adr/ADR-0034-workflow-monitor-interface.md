---
phase:
  current: 1
  total: 1
  names:
    1: "Implement workflow monitor"
  status:
    1: not-started
---

# ADR-0034: Workflow Monitor Interface

## Status

Accepted

**Phase:** Future — builds on Phases 3-5 (Actions, Routines, Crystals + Signals) to add observability.

## Context

KodeHold's Director now operates a sophisticated action-based delegation system: 10 action types (ADR-0031), 4 routine templates (ADR-0032), inter-agent signals (ADR-0033), and auto-crystallization. But the only way to observe live state is via CLI MCP tool calls — there is no visual UI.

**What exists:**
- Agentmemory viewer (port 3113) — generic database browser with Actions/Crystals tabs, but no frontier or delegation view
- *(Dashboard removed — ADR-0014 was a static HTML overview pattern)*

**The gap:** The Director must mentally fuse data from `memory_frontier`, `memory_signal_read`, `memory_diagnose`, and `memory_recall` to understand "what is happening right now."

## Decision

Build a workflow monitor — a Python script generating a self-contained HTML page at `docs/dashboard/workflow.html`, displaying 5 views queried from agentmemory's HTTP API (localhost:3111).

### Views

| View | Purpose | API Source |
|------|---------|------------|
| **Frontier Board** | Next unblocked actions, sorted by priority | `GET /api/frontier` |
| **Active Delegations** | Currently leased/in-progress actions | `GET /api/actions?status=active,leased` |
| **Recent Crystals** | Completed work chain digests | `GET /api/crystals` |
| **Signal Feed** | Threaded inter-agent messaging | `GET /api/signals` |
| **Session Status** | Lifecycle state, system health | `GET /api/sessions/current` + `/api/diagnose` |

### Script Design

- **Language:** Python 3 (stdlib only)
- **Output:** `docs/dashboard/workflow.html` — self-contained HTML, dark theme, inline CSS
- **Serve mode:** `--serve` flag enables auto-regeneration on a configurable timer
- **Error resilience:** Each endpoint is fetched independently; failures show "Data unavailable" per view

### Integration

- Companion to ADR-0014 dashboard — both live in `docs/dashboard/` with cross-links
- Read-only — observes agentmemory state, never modifies
- Zero LLM token cost — deterministic script

## Consequences

### Positive
1. Visual operational awareness — one page shows frontier, delegations, crystals, signals, session
2. Pattern reuse — follows `token-report.py` pattern exactly *(tool removed)*
3. Zero token cost — runs deterministically without LLM inference
4. Debugging aid — session status includes system health checks
5. Serve mode for continuous monitoring during long sessions

### Negative
1. Script maintenance — API schema changes require updating data collection functions
2. No real-time push — snapshot-based; --serve mode mitigates with auto-refresh
3. Depends on agentmemory daemon — degrades gracefully if daemon unreachable

### Follow-up
- [ ] Create `scripts/workflow-monitor.py`
- [ ] Implement 5 views + summary bar
- [ ] Implement `--serve` mode
- [ ] Add navigation links between ADR-0014 and ADR-0034 dashboards
- [ ] Register script in opencode.json as optional Director tool

## ADR References
- ADR-0014 (Status Dashboard) — foundational pattern for HTML generation
- ADR-0031 (Actions + Crystals) — action/frontier system
- ADR-0032 (Routine Templates) — template-generated action chains
- ADR-0033 (Crystals + Signals) — signal system
- *(Dashboard/`token-report.py` removed)*
