---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0002: Compiler-Pipeline Architecture

## Status

Proposed

## Context

KodeHold v1.0 uses an agent-based delegation model:

```
User → Director → Architects (design)
                → Engineers (implement)
                → Reviewers (review)
                → Testers (test)
                → Scribes (document)
```

Each delegation round-trips through an LLM. Each agent receives a "fresh"
context window with the task + relevant design doc sections. This means:

- A simple code change costs ~5 LLM round-trips minimum (design, implement,
  review, test, document)
- Each round-trip is non-deterministic — the same task can produce different
  results
- Context window inflation: the Director's prompt alone exceeds 500 lines
- Token waste: routine tasks (lint fix, import sort, format) go through the
  same expensive pipeline as complex features

The agent model makes sense for novel, ambiguous work that requires judgment.
It does NOT make sense for deterministic, mechanical work.

## Decision

Replace the agent delegation model with a compiler-pipeline architecture:

```
Source ──▶ Parse ──▶ Analyze ──▶ Generate ──▶ Verify ──▶ Optimize ──▶ Output
              │          │           │           │           │
              ▼          ▼           ▼           ▼           ▼
           tree-      type         solver      test        clippy
           sitter     checker      + AI        suite       fmt
```

### Phase Definitions

| Phase | Tool | Deterministic | Uses AI |
|-------|------|:---:|:---:|
| 1. Parse | Tree-sitter grammar | Yes | No |
| 2. Analyze | Type checker, IR builder | Yes | No |
| 3. Generate | Pattern solver → AI fallback | Partial | Fallback |
| 4. Verify | Test runner, linter, typecheck | Yes | No |
| 5. Optimize | Formatter, dead-code elim | Yes | No |

### AI Integration Point

AI enters ONLY in Phase 3 (Generate), and ONLY when the constraint solver
(ADR-0006) cannot find a matching pattern. Even then, the AI receives:
- The typed IR (not a natural-language task description)
- The closest-matching patterns as examples
- A constrained output format (must produce valid AST)

This is more like "guided code completion" than "autonomous agent."

### Pipeline Configuration

```yaml
# kodehold.yaml
pipeline:
  phases:
    parse:
      grammar: tree-sitter-khold
    analyze:
      type_check: strict
    generate:
      solver: pattern_db
      fallback: ollama/qwen3.5:9b
      max_fallback_attempts: 3
    verify:
      commands: [cargo test, cargo clippy, cargo fmt --check]
    optimize:
      commands: [cargo fmt, cargo fix --allow-dirty]
```

### Parallelism

Phases 1-3 and 4-5 are sequential, but within Verify and Optimize, sub-tasks
run in parallel (tests + lint + fmt simultaneously).

## Consequences

### Positive

- Deterministic paths for all non-AI phases — reproducible in CI
- Token cost proportional to problem novelty, not task count
- Pipeline phases are independently testable
- Clear separation: mechanical work (pipeline) vs. creative work (AI fallback)
- Pipeline stages can be swapped per language/project

### Negative

- No "discussion" or "negotiation" between phases — must be right the first time
- The AI fallback has less context than a v1.0 agent (no design doc, no history)
- Pipeline failures are binary — no "partial success" or human-in-the-loop
- Loses the social metaphor that some users found valuable

### Migration

V1.0 agent-based workflows continue for existing projects. New projects created
with `kodehold init` use the pipeline. An `--agent-mode` flag on `kodehold init`
can opt back into v1.0-style delegation if needed.
