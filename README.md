# KodeHold — /ˈkəʊd həʊld/ → "Code Hold"

**A specification compiler and code generation toolchain.**

KodeHold produces software from DSL specifications through deterministic,
machine-native pipelines. It compiles specs to code using a constraint solver,
a typed intermediate representation, and plugin-extensible event-driven
execution — all in a single Rust binary with zero runtime dependencies.

> **⚠️ WORK IN PROGRESS — EXPERIMENTAL**
> KodeHold v2.0 is under active development. Expect breaking changes.

## Philosophy

- **Specification-first** — describe *what* you want in the `.khold` DSL, not *how* to build it
- **Deterministic by default** — equivalent input produces equivalent output; AI is a fallback, not the primary path
- **Git as source of truth** — all state in git tags, no marker files, no side channels
- **Events over polling** — webhooks and file watchers, not cron ticks
- **Zero runtime dependencies** — one static binary, no Python, no Node.js, no pip

## Architecture

```
Events → Dispatch → Pipeline (5 phases) → Verify → Ship
                        │
                   Constraint Solver
                   (Pattern Database)
```

### Pipeline Phases

| Phase | Input | Output | AI? |
|-------|-------|--------|-----|
| 1. Parse | `.khold` spec file | AST | No |
| 2. Analyze | AST | Typed IR | No |
| 3. Generate | Typed IR + patterns | Source code | Fallback only |
| 4. Verify | Source code + tests | Pass/Fail | No |
| 5. Optimize | Source code | Optimized code | No |

AI is used **only** in the Generate phase, and only when the constraint solver
cannot find a matching pattern in the known-solutions database.

### State Machine (Git-Native)

```
git tag v2-init    ←─ after kodehold init
git tag v2-active  ←─ pipeline running
git tag v2-review  ←─ verification complete, awaiting human
git tag v2-closed  ←─ shipped
```

State lives in git tags and conventional commit messages — no marker files.

## CLI Commands

```
kodehold init <name>       # Create new project from template
kodehold spec <file.khold> # Compile a .khold specification file
kodehold verify            # Run project verification (tests + lint)
kodehold ship              # Bump version, tag, changelog
kodehold pipeline          # Run the full compilation pipeline
kodehold state             # Show workspace state from git tags
kodehold listen            # Start event daemon (webhooks + file watcher)
kodehold plugin <cmd>      # Manage hooks and plugins
kodehold pattern <cmd>     # Manage the pattern database
kodehold migrate           # Migrate from v1.0 workspace
```

## The `.khold` DSL

```
// spec.khold — KodeHold Specification Language

project "my-trading-bot" {
  version = "0.1.0"
  language = "rust"
}

type Price = f64
type Timestamp = u64

interface MarketDataFeed {
  fn fetch(symbol: String, from: Timestamp, to: Timestamp) -> Result<Vec<OHLCV>, Error>
}

interface RegimeClassifier {
  fn classify(data: Vec<OHLCV>) -> Regime
}
```

## Quick Start

```bash
# Prerequisites: Rust toolchain
cargo build --release
./target/release/kodehold init my-project
cd my-project
# Write your spec.khold, then:
kodehold spec spec.khold
kodehold verify
kodehold ship
```

## Project Layout

```
├── src/              → Rust source (the single binary)
│   ├── cli.rs        → CLI argument definitions
│   ├── cmd/          → Command implementations
│   ├── dsl/          → Parser, AST, IR, codegen
│   ├── pipeline/     → 5-phase compilation pipeline
│   ├── solver/       → Constraint solver + pattern DB
│   ├── events/       → Webhook server + file watcher
│   ├── plugin/       → Hook system
│   ├── state.rs      → Git-native lifecycle
│   └── config.rs     → kodehold.yaml config
├── docs/
│   ├── design/       → Living design document
│   └── adr/          → Architecture Decision Records
├── Cargo.toml
└── kodehold.yaml
```

## Documentation

| Path | Description |
|------|-------------|
| `docs/design/README.md` | Full architecture, lifecycle, constraints |
| `docs/adr/` | ADR-0001 through ADR-0009 (all Accepted) |
| `AGENTS.md` | Development rules and conventions |

## Development

```bash
cargo build           # Compile the binary
cargo test            # Run Rust test suite
cargo run -- init my-project  # Try it out
```

## Requirements

- **Rust toolchain** (edition 2021)
- No Python, no Node.js, no external services
