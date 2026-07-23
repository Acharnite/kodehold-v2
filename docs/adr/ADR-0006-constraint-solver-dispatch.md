---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0006: Constraint-Solver Dispatch for Pattern Matching

## Status

Proposed

## Context

In KodeHold v1.0, every task — from "fix this import" to "design a new trading
strategy" — goes through the same delegation pipeline:

```
Director reads task → delegates to Engineers → Engineers implement → Director
delegates to Reviewers → Reviewers review → Director delegates to Testers →
Testers test → Director delegates to Scribes → Scribes document
```

This is appropriate for novel, ambiguous work. It is wasteful for:

- **Boilerplate generation**: CRUD endpoints, config parsing, error wrapping
- **Mechanical refactors**: Rename variable, extract function, move module
- **Pattern-repeating tasks**: "Add the same logging to all 12 handlers"
- **Known-solution tasks**: "Sort this list", "Parse this CSV", "Hash this
  password" — solutions known for decades

In v1.25.0, a simple task like "add a healthcheck endpoint" costs ~15k tokens.
The same task via copy-paste from a pattern library costs 0 tokens.

## Decision

Introduce a constraint-solver dispatch layer between events and the pipeline.

### Architecture

```
Event ──▶ Solver ──▶ Match found? ──▶ Apply pattern → Verify → Done
                │
                └── No match? ──▶ AI fallback (Generate phase) → Verify → Done
                                         │
                                         └── New pattern learned → Store in DB
```

### Pattern Database

A searchable, tag-indexed database of known implementation patterns:

```json
{
  "id": "pat-a1b2c3",
  "signature": "fn(Vec<OHLCV>) -> Regime",
  "tags": ["trading", "classification", "time-series", "financial"],
  "constraints": {
    "time_complexity": "O(n)",
    "space_complexity": "O(1)",
    "purity": true
  },
  "language": "rust",
  "template": "match (first, last) { ... }",
  "test_fixtures": ["mock_trending_up", "mock_ranging"],
  "provenance": {
    "source": "ai-generated",
    "original_task": "Classify market regime from OHLCV data",
    "verified_by": "test suite pass + human review",
    "usage_count": 47,
    "success_rate": 0.98
  }
}
```

### Solver Algorithm

1. **Signature matching**: Does a pattern exist with the same type signature?
2. **Constraint filtering**: Eliminate patterns that violate constraints
   (complexity, purity, dependencies).
3. **Tag scoring**: Rank remaining patterns by tag overlap with the task.
4. **Template application**: Apply the best pattern's template with the
   current task's identifiers.
5. **Verify**: Run the test suite. If tests pass, done. If not, try next pattern.

The solver is deterministic: same input → same pattern selection every time.

### Pattern Learning

When the AI fallback generates a solution that passes verification, and the
task is not unique (usage_count > 3 for similar signatures), the solution is
automatically templated and stored as a new pattern. This means the database
grows with usage.

### Library Sources

Initial seeding from:
1. Kodehold v1.0's existing workspace commit history (extract patterns from
   past implementations)
2. Rosetta Code (curated implementations of common tasks)
3. Standard library idioms (e.g., Rust's `Entry` API pattern)
4. Design patterns catalogued as templates (Strategy, Observer, etc.)

## Consequences

### Positive

- 80%+ of routine tasks solved without token cost
- Deterministic output for known patterns — reproducible in CI
- Pattern DB improves with usage (network effect)
- Learning from past implementations instead of re-deriving
- Token budget reserved for genuinely novel work

### Negative

- Cold start: empty DB means everything falls back to AI
- Pattern quality: auto-generated templates may encode bugs
- Template rigidity: patterns solve the EXACT problem they were created for,
  not variations
- Maintenance: stale patterns need pruning (e.g., deprecated API usage)

### Risks

- **Pattern overfitting**: The solver picks a pattern that matches the
  signature but not the semantics. Mitigation: test fixtures must pass.
- **DB pollution**: Low-quality AI-generated patterns accumulate.
  Mitigation: success_rate threshold for promotion to "trusted" patterns.
- **Security**: A malicious pattern could inject vulnerable code.
  Mitigation: patterns go through the same verify pipeline as generated code.

### Metrics

- Solver hit rate: % of tasks matched from DB (target: >60% after 3 months)
- Token savings: tokens NOT spent vs. v1.0 baseline
- Pattern quality: % of solver-applied patterns that pass verify on first try
