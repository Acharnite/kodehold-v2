# KodeHold v2.0 — Design Document
**Version:** 0.2.0
**Status:** Active
**Design Authority:** Architects
**Last Reviewed:** 2026-07-23
**Origin:** Forked from kodehold v1.25.0

> KodeHold v2.0 is a fundamental redesign. V1.0 simulates a human software organization
> (Director + 6 teams). V2.0 replaces that metaphor with machine-native patterns:
> compiler pipelines, constraint solvers, event-driven dispatch, and a DSL for
> software specification. The goal: deterministic, auditable, token-efficient
> code generation where AI is a tool, not a persona.

## 1. Purpose & Scope

KodeHold v2.0 produces software from specifications through deterministic,
machine-native pipelines. It does not simulate an org chart. It compiles specs
to code.

### What v2.0 is NOT

- NOT an AI agent orchestrator
- NOT a chat-based workflow engine
- NOT a marker-file state machine
- NOT a Python-only tool

### What v2.0 IS

- A specification compiler that generates code deterministically
- A constraint solver that matches problems to known solutions
- An event-driven system that reacts to git events, not cron ticks
- A single static binary with zero runtime dependencies

### Scope

V2.0 covers the entire lifecycle: spec → generate → verify → ship. It does not
replace v1.0 — both coexist during development. V1.0 continues to manage the
workspace where v2.0 is built.

## 2. Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | Deterministic output for equivalent input | Must |
| R2 | Token cost proportional to problem novelty, not problem count | Must |
| R3 | Zero runtime dependencies (single binary) | Must |
| R4 | Git-native state — no marker files, no side-channel state | Must |
| R5 | Plugin-extensible per workspace | Should |
| R6 | Real-time event response (webhooks, file watchers) | Should |
| R7 | Executable specifications that serve as both tests and design docs | Should |
| R8 | DSL for describing software design (types, interfaces, constraints) | Nice-to-have |
| R9 | Solver library of known implementation patterns | Nice-to-have |

## 3. Architecture Overview

```
                                  ┌──────────────┐
                                  │  kodehold.yaml│  ← Per-workspace config
                                  │  hooks/       │  ← Per-workspace plugins
                                  └──────┬───────┘
                                         │
  ┌──────────┐    ┌──────────┐    ┌─────▼──────┐    ┌──────────┐    ┌──────────┐
  │  Events   │───▶│ Dispatch │───▶│  Pipeline   │───▶│  Verify   │───▶│  Ship    │
  │           │    │          │    │             │    │           │    │          │
  │ webhooks  │    │ solver   │    │ 1.parse     │    │ tests     │    │ git tag  │
  │ inotify   │    │ router   │    │ 2.analyze   │    │ lint      │    │ changelog│
  │ msg queue │    │          │    │ 3.generate  │    │ typecheck │    │ notify   │
  └──────────┘    └──────────┘    │ 4.verify    │    └──────────┘    └──────────┘
                                  │ 5.optimize  │
                                  └─────────────┘
                                         │
                                  ┌──────▼───────┐
                                  │  Constraint   │
                                  │  Solver       │
                                  │  ───────────  │
                                  │  Known        │
                                  │  Patterns DB  │
                                  └──────────────┘
```

### Pipeline Phases

| Phase | Input | Output | AI? |
|-------|-------|--------|-----|
| 1. Parse | `.khold` spec file | AST | No |
| 2. Analyze | AST | Typed IR | No |
| 3. Generate | Typed IR + patterns | Source code | Yes (fallback) |
| 4. Verify | Source code + tests | Pass/Fail | No |
| 5. Optimize | Source code | Optimized code | No |

AI is used ONLY in the Generate phase, and ONLY when the constraint solver
cannot find a matching pattern in the known-solutions database.

### State Machine (Git-Native)

```
git tag v2-init    ←─ after kodehold init
git tag v2-active  ←─ pipeline running
git tag v2-review  ←─ verification complete, awaiting human
git tag v2-closed  ←─ shipped
```

No `.kodehold-state` file. No `.design_reviewed` markers. State lives in git
tags and conventional commit messages.

## 4. Component Design

### 4.1 Core Engine (`kodehold-core`)

The single binary. Written in Rust. Commands:

```
kodehold init <name>       # Create new project from template
kodehold parse <spec.khold> # Parse spec to AST
kodehold generate          # Generate code from spec
kodehold verify            # Run test suite + lint + typecheck
kodehold ship              # Bump version, tag, changelog, push
kodehold listen            # Start event listener (webhook server + inotify)
kodehold plugin <cmd>      # Plugin management
```

### 4.2 Constraint Solver (`kodehold-solve`)

Maintains a database of known implementation patterns (tagged, searchable).
Given a typed IR, returns the best-matching pattern. Falls back to AI generation
when no pattern matches.

Pattern database format:
```json
{
  "id": "pat-001",
  "signature": "fn(Vec<Price>) -> Regime",
  "tags": ["trading", "classification", "time-series"],
  "complexity": "O(n)",
  "template": "...",
  "tests": ["..."],
  "usage_count": 47
}
```

### 4.3 Event System (`kodehold-events`)

```yaml
# kodehold.yaml
events:
  sources:
    - type: github_webhook
      endpoint: /hooks/github
      actions: [push, pull_request, issue]
    - type: inotify
      paths: [src/, tests/]
    - type: schedule
      cron: "0 9 * * *"      # Optional cron fallback for legacy
  handlers:
    on_push:
      - pipeline: full       # parse → generate → verify
    on_pull_request:
      - pipeline: verify     # verify only
    on_issue:
      - dispatch: issue_triage
```

### 4.4 Plugin System (`kodehold-plugin`)

Each workspace has:
```
my-project/
├── kodehold.yaml       # Config + event definitions
├── hooks/
│   ├── pre-generate    # Executable script, runs before generate phase
│   ├── post-verify     # Executable script, runs after verify phase
│   └── on-ship         # Executable script, runs on ship
└── spec.khold          # DSL specification (optional)
```

Plugins are executables (any language) that receive JSON on stdin, write JSON
to stdout. The core engine only cares about exit codes and stdout.

### 4.5 DSL (`khold` format)

```
// spec.khold — KodeHold Specification Language

project "my-trading-bot" {
  version = "0.1.0"
  language = "rust"
}

type Price = f64
type Timestamp = u64
type OHLCV = struct { open: Price, high: Price, low: Price, close: Price, volume: u64 }

interface MarketDataFeed {
  fn fetch(symbol: String, from: Timestamp, to: Timestamp) -> Result<Vec<OHLCV>, Error>
}

interface RegimeClassifier {
  fn classify(data: Vec<OHLCV>) -> Regime
}

enum Regime { Trending, Ranging, Volatile, Unknown }

// Constraint: classify must complete in O(n) time, O(1) auxiliary space
constraint classify: time <= O(n), space <= O(1)

// Constraint: output must be pure (no side effects)
constraint classify: pure

// Test: known classification
test "trending_up": classify(mock_trending_up()) == Regime::Trending
```

## 5. Data Model

### Project State (in `.git/kodehold/`)

```
.git/kodehold/
├── state            # Current state tag name
├── pipeline.log     # Last pipeline run output
├── pattern_cache/   # Cached solver results
└── events.db        # SQLite event log
```

### Pattern Database (global, shared)

```
~/.kodehold/
├── patterns.db      # SQLite: known implementation patterns
├── config.yaml      # Global config
└── credentials/     # API keys, webhook secrets
```

### Event Schema

```json
{
  "id": "evt-abc123",
  "type": "github.push",
  "timestamp": "2026-07-23T04:30:00Z",
  "workspace": "my-project",
  "payload": {
    "ref": "refs/heads/main",
    "commits": [...]
  }
}
```

## 6. API Design

### CLI (primary interface)

```
kodehold <command> [flags] [args]
```

All output: JSON lines to stdout, logs to stderr. `--json` for machine-readable.

### HTTP API (for event listener)

```
POST /hooks/github       GitHub webhook endpoint
POST /hooks/gitlab       GitLab webhook endpoint
GET  /health             Health check
GET  /api/v1/workspaces  List workspaces
GET  /api/v1/workspaces/:name/state
POST /api/v1/workspaces/:name/trigger   Manually trigger pipeline
```

### Plugin Interface

```
stdin:  JSON event or context
stdout: JSON result or modified context
stderr: logs
exit 0: success
exit non-zero: failure (pipeline aborts)
```

## 7. Implementation Plan

### Phase 1: Core Binary (ADR-0008)
- Rust project setup, CLI framework (clap)
- `kodehold init`, `kodehold verify`, `kodehold ship`
- Git-native state (ADR-0003)

### Phase 2: Pipeline (ADR-0002)
- Parse → analyze → generate → verify → optimize passes
- Constraint solver with initial pattern DB (ADR-0006)

### Phase 3: Events (ADR-0007)
- Webhook server, inotify watcher
- Event dispatch → pipeline routing

### Phase 4: Plugins (ADR-0005)
- Per-workspace kodehold.yaml + hooks/
- Plugin interface specification

### Phase 5: DSL (ADR-0009)
- khold language parser
- Spec → IR → code compilation
- Executable spec integration (ADR-0004)

### Phase 6: Migration
- v1.0 workspaces auto-adopted by v2.0
- Bridge mode: v2.0 delegates to v1.0 agents as fallback

## 8. Testing Strategy

### Test Layers

| Layer | Tool | Scope |
|-------|------|-------|
| Unit | Rust `#[test]` | Per function/module |
| Integration | Rust test + test fixtures | Pipeline runs |
| Executable specs | Property-based (proptest) | Design assertions |
| Pattern DB | Snapshot tests | Known patterns produce expected output |
| CLI | assert_cmd / trycmd | End-to-end CLI |
| Events | Mock webhook server | Event flow |

### Executable Specs as Tests

Every ADR produces executable tests that must pass. Example:
```rust
#[test]
fn adr_0003_git_state_is_immutable_on_reopen() {
    // If a project is CLOSED and reopened, the original git tag history
    // must remain intact — no tag deletion, only new tags appended.
}
```

## 9. ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| ADR-0001 | KodeHold v2.0 Foundation and Principles | Accepted |
| ADR-0002 | Compiler-Pipeline Architecture | Proposed |
| ADR-0003 | Git-Native State Machine | Proposed |
| ADR-0004 | Executable Specifications as Design Documents | Proposed |
| ADR-0005 | Plugin Architecture for Per-Workspace Customization | Proposed |
| ADR-0006 | Constraint-Solver Dispatch for Pattern Matching | Proposed |
| ADR-0007 | Event-Driven Architecture over Cron Polling | Proposed |
| ADR-0008 | Single Binary Distribution in Rust | Proposed |
| ADR-0009 | DSL for Software Specification and Compilation | Proposed |

## 10. Open Questions

1. **Rust vs Go**: Rust chosen for zero-cost abstractions and type system, but Go
   has faster compile times. Benchmark before Phase 1 commit.
2. **Pattern DB format**: SQLite sufficient, or need vector search for
   similarity matching of code patterns?
3. **DSL syntax**: Custom parser or reuse existing (Tree-sitter grammar,
   WASM plugin, Dhall/CUE for config)?
4. **V1.0 bridge**: How deeply should v2.0 integrate with v1.0's agent system?
   Full delegation or only state sync?
5. **LLM provider for fallback**: Which model for the generate fallback?
   Must be local-first (Ollama) but capable of code generation.
6. **Multi-language support**: V1.0 supports any language via test framework
   detection. Should v2.0 start with Rust-only and add languages later?

## 11. Changelog

- **2026-07-23 (v0.2.0)**: Complete rewrite — v2.0 design with 8 ADRs
- 2026-07-23 (v0.1.0): Adopted by KodeHold v1.0, seeded from v1.25.0
