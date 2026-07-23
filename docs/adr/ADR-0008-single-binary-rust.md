---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0008: Single Binary Distribution in Rust

## Status

Proposed

## Context

KodeHold v1.0 is distributed as a set of Python scripts:

```
scripts/
├── gate.py          (743 lines)
├── workspace.py     (1261 lines)
├── ship.py          (407 lines)
├── discord-notify.py (134 lines)
├── detect_test_framework.py (117 lines)
└── lib/
    ├── __init__.py
    └── output.py    (175 lines)
```

Dependencies managed via `requirements.txt`, installed into `.venv/`.

Problems:

1. **Installation friction**: User must have Python 3.12+, pip, venv. Then
   `python3 -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt`.
2. **Startup latency**: Python interpreter startup (~50ms) + import resolution
   adds latency to every CLI invocation.
3. **Distribution size**: `.venv/` with dependencies is ~200MB (pytest, cov, yaml).
4. **Cross-platform**: Python is portable in theory, but `os.path`, subprocess,
   and signal handling differ between Linux and macOS. CI must test on both.
5. **Type safety**: Python's dynamic typing means `AttributeError` at runtime
   for what a compiler would catch at compile time.
6. **Dependency conflicts**: Workspace projects may need different Python
   versions — conflicts with kodehold's own dependencies.
7. **Script scattering**: Multiple `.py` files with overlapping concerns.
   `gate.py` and `workspace.py` both import from `lib/output.py`.

## Decision

Rewrite KodeHold v2.0 as a single statically-compiled binary in Rust.

### Binary Surface

```
kodehold
├── init        Create new project
├── adopt       Adopt existing project
├── list        List workspaces
├── state       Show workspace state
├── gate        Run lifecycle gate transition
├── verify      Run project verification (tests + lint + typecheck)
├── ship        Shipping gate + version bump + changelog
├── listen      Start event listener daemon
├── plugin      Plugin management (install, list, remove)
├── cost        Estimate token cost for pattern/operation
├── config      Manage global and per-workspace config
└── help        Show help
```

### Why Rust

| Factor | Rust | Go | Python |
|--------|------|----|--------|
| Single binary | Yes (musl) | Yes (static) | No (needs interpreter) |
| Startup time | <1ms | <1ms | ~50ms |
| Memory | ~2MB baseline | ~2MB baseline | ~20MB baseline |
| Type safety | Compile-time | Compile-time | Runtime |
| Cross-compile | Excellent (cross) | Excellent (GOOS/GOARCH) | Poor (needs target Python) |
| CLI ecosystem | clap (mature) | cobra (mature) | click/argparse |
| Async runtime | tokio (mature) | Built-in goroutines | asyncio |
| Tree-sitter bindings | First-class | Via CGo | Via binding |
| Git library | git2 (libgit2) | go-git (pure Go) | GitPython |
| Learning curve | Steep (borrow checker) | Shallow | None (v1.0 is Python) |

Rust is chosen over Go primarily for:
- **Tree-sitter**: Rust is the native language of tree-sitter. Bindings are
  first-class and synchronous.
- **WASM plugin support**: Future plugin sandboxing via WASM runtime (wasmtime)
  is native in Rust.
- **Correctness guarantees**: The borrow checker eliminates data races and
  use-after-free — valuable for a tool that runs arbitrary hooks.
- **Pattern matching**: Rust's exhaustive match simplifies pattern DB querying.

### Internal Architecture

```
src/
├── main.rs              Entry point, CLI dispatch
├── cli/
│   ├── mod.rs
│   ├── init.rs
│   ├── gate.rs
│   ├── verify.rs
│   ├── ship.rs
│   ├── listen.rs
│   └── state.rs
├── pipeline/
│   ├── mod.rs           Pipeline orchestration
│   ├── parse.rs         Tree-sitter spec parsing
│   ├── analyze.rs       Type checking, IR building
│   ├── generate.rs      Pattern solver + AI fallback
│   ├── verify.rs        Test runner, lint integration
│   └── optimize.rs      Formatter, dead code elimination
├── solver/
│   ├── mod.rs           Constraint solver engine
│   ├── pattern_db.rs    Pattern database (SQLite)
│   └── template.rs      Template application
├── events/
│   ├── mod.rs           Event system
│   ├── webhook.rs       HTTP webhook server
│   ├── watcher.rs       inotify/kqueue file watcher
│   └── router.rs        Event → pipeline dispatch
├── state/
│   ├── mod.rs           Git-native state machine
│   └── tag.rs           Git tag conventions
├── plugin/
│   ├── mod.rs           Plugin manager
│   └── sandbox.rs       Plugin execution sandbox
├── workspace/
│   ├── mod.rs           Workspace management
│   └── catalog.rs       Workspace catalog (~/.kodehold/)
├── dsl/
│   ├── mod.rs           Khold language parser
│   ├── ast.rs           AST types
│   ├── ir.rs            Intermediate representation
│   └── codegen.rs       Code generation from IR
└── config/
    ├── mod.rs           Configuration loading
    └── schema.rs        kodehold.yaml schema validation
```

### Distribution

```
# Linux (musl, static)
curl -L https://kodehold.dev/releases/v2.0.0/kodehold-x86_64-unknown-linux-musl.tar.gz | tar xz

# macOS (Apple Silicon)
curl -L https://kodehold.dev/releases/v2.0.0/kodehold-aarch64-apple-darwin.tar.gz | tar xz

# Via cargo
cargo install kodehold

# Docker
docker run -v $(pwd):/workspace ghcr.io/acharnite/kodehold:latest gate
```

## Consequences

### Positive

- One file to install. No Python, no pip, no venv.
- Instant startup. `time kodehold state` < 5ms.
- Type safety catches bugs at compile time, not in production.
- Cross-compile to 15+ targets from one CI job.
- WASM sandbox for plugin execution (defense in depth).
- tree-sitter grammars load as native code, no FFI overhead.

### Negative

- Rust has a steep learning curve for contributors.
- Compile times (2-5 min for clean build) slow down development.
- Python scripts were easy to hot-patch; Rust requires recompilation.
- No runtime scripting: can't `exec()` user Python like v1.0 plugins could.
- Binary size: 15-30MB for a static Rust binary vs. ~50KB of Python scripts.

### Risks

- **Prototyping velocity**: Rust development is slower than Python. Phase 1
  may take longer than the 4-week target. Mitigation: start with just `init`,
  `gate`, `verify`, `ship`, `state` — defer solver/DSL/events to later phases.
- **Library maturity**: Rust's webhook server (axum/actix) and git (git2) are
  mature. But the pattern DB query engine may need custom SQL.
- **Contributor pool**: Fewer Rust developers than Python developers.
  Mitigation: excellent documentation, plugin system allows any language.

### Migration

V1.0 scripts continue to work. V2.0 binary operates alongside them. When v2.0
reaches feature parity, `scripts/` is archived to `scripts-v1/` and the
Python dependency is dropped from `requirements.txt`.
