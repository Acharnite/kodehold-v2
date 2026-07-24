# ADR Index — KodeHold v2.0

## Active ADRs (v2.0)

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-0001](ADR-0001-kodehold-v2-foundation.md) | KodeHold v2.0 Foundation and Principles | Accepted |
| [ADR-0002](ADR-0002-compiler-pipeline-architecture.md) | Compiler-Pipeline Architecture | Accepted |
| [ADR-0003](ADR-0003-git-native-state-machine.md) | Git-Native State Machine | Accepted |
| [ADR-0004](ADR-0004-executable-specifications.md) | Executable Specifications as Design Documents | Accepted |
| [ADR-0005](ADR-0005-plugin-architecture.md) | Plugin Architecture for Per-Workspace Customization | Accepted |
| [ADR-0006](ADR-0006-constraint-solver-dispatch.md) | Constraint-Solver Dispatch for Pattern Matching | Accepted |
| [ADR-0007](ADR-0007-event-driven-architecture.md) | Event-Driven Architecture over Cron Polling | Accepted |
| [ADR-0008](ADR-0008-single-binary-rust.md) | Single Binary Distribution in Rust | Accepted |
| [ADR-0009](ADR-0009-dsl-software-specification.md) | DSL for Software Specification and Compilation | Accepted |

All ADRs are Accepted as of v2.0.0 — the v1.0 Python code, scripts, tests,
config, and loop infrastructure have been removed. KodeHold is now a pure
Rust single-binary tool.

V1.0 ADRs are available in git history (commit `eb2f7b7` — the initial seed).
They were removed to keep the v2.0 project clean.

## ADR Format

KodeHold v2.0 uses a hybrid format:
- **Executable ADRs**: Design decisions expressed as property-based tests in
  `tests/adr/`. These are machine-checkable.
- **Non-executable ADRs**: Policy decisions, trade-offs, and contextual
  decisions that resist computable expression. These use the Nygard format
  (Status, Context, Decision, Consequences) with an `executable: false` header.

See ADR-0004 for the full migration plan from markdown ADRs to executable
specifications.
