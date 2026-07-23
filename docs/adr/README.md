# ADR Index — KodeHold v2.0

## Active ADRs (v2.0)

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-0001](ADR-0001-kodehold-v2-foundation.md) | KodeHold v2.0 Foundation and Principles | Accepted |
| [ADR-0002](ADR-0002-compiler-pipeline-architecture.md) | Compiler-Pipeline Architecture | Proposed |
| [ADR-0003](ADR-0003-git-native-state-machine.md) | Git-Native State Machine | Proposed |
| [ADR-0004](ADR-0004-executable-specifications.md) | Executable Specifications as Design Documents | Proposed |
| [ADR-0005](ADR-0005-plugin-architecture.md) | Plugin Architecture for Per-Workspace Customization | Proposed |
| [ADR-0006](ADR-0006-constraint-solver-dispatch.md) | Constraint-Solver Dispatch for Pattern Matching | Proposed |
| [ADR-0007](ADR-0007-event-driven-architecture.md) | Event-Driven Architecture over Cron Polling | Proposed |
| [ADR-0008](ADR-0008-single-binary-rust.md) | Single Binary Distribution in Rust | Proposed |
| [ADR-0009](ADR-0009-dsl-software-specification.md) | DSL for Software Specification and Compilation | Proposed |

## Historical ADRs (v1.0)

The following ADRs are from KodeHold v1.0 (the agent-orchestrator design).
They are preserved for reference. V2.0 supersedes them with a fundamentally
different architecture. See `../v1.25.0-seed` in git history for the full
v1.0 context.

All v1.0 ADRs are in-process of being migrated to executable specifications
per ADR-0004. Until then, they remain in this directory for historical context.

The v1.0 ADRs (ADR-0001 through ADR-0060 in the Nygard-format filenames) cover:
- ADR-0001: Foundation (agent-based design)
- ADR-0002: Director + 6 teams
- ADR-0003: Design document lifecycle
- ADR-0005: LLM support
- ADR-0006: Second opinion system
- ADR-0008: Project lifecycle (marker-based)
- ADR-0010: FLS front line support
- ADR-0012: Adopted projects (symlink)
- ADR-0017: Reviewers as gatekeepers
- ADR-0049: Ponytail "Lazy Senior Dev" philosophy
- ADR-0054: Graphify knowledge graph
- ADR-0060: Loop engineering
- ... and 27 more (37 active, 23 inactive)

## ADR Format

KodeHold v2.0 uses a hybrid format:
- **Executable ADRs**: Design decisions expressed as property-based tests in
  `tests/adr/`. These are machine-checkable.
- **Non-executable ADRs**: Policy decisions, trade-offs, and contextual
  decisions that resist computable expression. These use the Nygard format
  (Status, Context, Decision, Consequences) with an `executable: false` header.

See ADR-0004 for the full migration plan from markdown ADRs to executable
specifications.
