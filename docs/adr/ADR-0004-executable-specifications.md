---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0004: Executable Specifications as Design Documents

## Status

Accepted

## Context

KodeHold v1.0 uses Architecture Decision Records (ADRs) in markdown format.
There are 37 active ADRs, each 30-100 lines, written in Nygard format (Status,
Context, Decision, Consequences). These documents serve as the project's design
memory.

Problems observed over 25 versions:

1. **Document drift**: ADRs describe decisions made months ago. The code may
   have diverged. Nobody knows until someone manually cross-references.

2. **Review burden**: ADR-0048 made documentation review mandatory. This adds
   a step to every workflow but doesn't guarantee accuracy.

3. **Human-only verification**: "ADR-0003 says the design doc lifecycle is X."
   How do you verify this? You read the ADR and manually compare with the code.

4. **ADR inflation**: 37 active ADRs. 23 inactive. The directory grows
   monotonically. Finding relevant decisions becomes search, not browse.

5. **No enforcement**: ADRs are advisory. Nothing breaks if they're wrong.

The fundamental problem: ADRs are documentation, not code. They can lie.

## Decision

Replace markdown ADRs with executable specifications. Every design decision
that CAN be expressed as a computable assertion MUST be.

### Format: Property-Based Tests

Instead of:
```markdown
# ADR-0003: Design Document Lifecycle
## Decision
The design doc must be read before any implementation work begins
and updated after any implementation work completes.
```

Write:
```rust
#[test]
fn design_doc_read_before_implementation() {
    let ctx = TestContext::new();
    let impl_phase = ctx.pipeline.run_phase("generate");
    assert!(
        impl_phase.preceding_phases.contains(&"parse_design_doc"),
        "Implementation phase must be preceded by design doc parsing"
    );
}

#[test]
fn design_doc_updated_after_implementation() {
    let ctx = TestContext::with_mock_changes();
    let result = ctx.pipeline.run_to_completion();
    assert!(
        result.design_doc_last_modified > result.impl_start_time,
        "Design doc must be modified after implementation completes"
    );
}
```

### Hybrid: Decision Records for Non-Computable Decisions

Some decisions are inherently non-computable (e.g., "we chose MIT license").
These remain as markdown ADRs but are tagged `[non-executable]` and placed in
a separate directory. The expectation: 80%+ of design decisions become
executable.

### Relationship to Test Suite

Executable specs live in `tests/adr/` and run as part of the standard test
suite. CI enforces them. A failing ADR test is a failing build.

```
tests/
├── adr/
│   ├── adr_0001_foundation.rs
│   ├── adr_0002_pipeline.rs
│   ├── adr_0003_git_state.rs
│   └── ...
├── unit/
└── integration/
```

### Non-Executable ADR Format

```markdown
---
executable: false
reason: Policy decision with no code enforcement path
---
# ADR-0010: License Selection
...
```

## Consequences

### Positive

- Design doc is always truthful: if code diverges, tests fail
- No separate "documentation review" step needed — tests ARE the review
- ADRs become part of CI, not a separate concern
- New contributors can read executable specs to understand expected behavior
- Refactoring safety: ADR tests catch unintended design violations

### Negative

- Writing executable specs takes more effort than writing markdown
- Some design decisions resist computable expression (policies, trade-offs)
- Test suite grows significantly (37 ADRs → 37+ test files)
- Requires test infrastructure for design-level assertions (mocking pipeline, etc.)

### Risks

- **Test rot**: If ADR tests are fragile or slow, developers will disable them.
  Mitigation: ADR tests must be fast (<100ms each) and deterministic.
- **False precision**: Computable assertions may oversimplify nuanced decisions.
  Mitigation: non-executable ADRs remain available for policy decisions.
