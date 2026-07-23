---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0001: KodeHold v2.0 Foundation and Principles

## Status

Accepted

## Context

KodeHold v1.0 (v1.0.0 through v1.25.0) was designed around a metaphor: a human
software organization with a Director and 6 specialized teams (Architects,
Engineers, Reviewers, Testers, Scribes, FLS). This metaphor proved powerful for
structured development but carries fundamental costs:

1. **Token overhead**: Every delegation requires context re-injection. A simple
   task that could be solved deterministically still costs thousands of tokens
   for "discussion" between agents.

2. **Non-determinism**: The same prompt to the same agent on different days
   produces different output. This makes debugging, auditing, and CI
   reproducibility difficult.

3. **Persona hallucination**: Agents role-playing as "Architects" or "Reviewers"
   sometimes produce persona-consistent but technically-incorrect output.

4. **Fragile state**: Marker files (`.design_reviewed`, `.testers_done`) are
   filesystem artifacts that don't survive `git clean`, conflict resolution, or
   workspace resets.

5. **Python distribution overhead**: `scripts/` requires venv, pip, and Python
   3.12+. Not self-contained.

6. **Polling waste**: 7 cron-based loop patterns poll repeatedly regardless of
   whether anything changed.

KodeHold v2.0 takes the lessons from v1.0's 25 versions and builds a
fundamentally different system. The metaphor shifts from "org chart" to
"compiler."

## Decision

We establish 8 design directions for v2.0. Each is documented in its own ADR:

| ADR | Direction | Replaces |
|-----|-----------|----------|
| 0002 | Compiler-pipeline architecture | Director + 6 teams |
| 0003 | Git-native state machine | Marker files (.design_reviewed, etc.) |
| 0004 | Executable specifications | Markdown ADR documents |
| 0005 | Plugin architecture | Monolithic workspace.py |
| 0006 | Constraint-solver dispatch | Agent delegation for routine tasks |
| 0007 | Event-driven architecture | Cron-based loop patterns |
| 0008 | Single binary (Rust) | Python scripts + venv |
| 0009 | DSL for software specification | AI as primary code generator |

### Core Principles

1. **Determinism over Conversation**: Equivalent input must produce equivalent
   output. AI is a fallback, not the primary path.

2. **Git as Source of Truth**: All state lives in git. No side-channel files.
   Tags, commits, and branches are the state machine.

3. **Zero Runtime Dependencies**: One binary. No Python, no Node.js, no pip.
   Static compilation.

4. **Events over Polling**: The system reacts to changes, it does not ask "did
   anything change?".

5. **Specification over Delegation**: Describe what you want, don't negotiate
   with agents about how to build it.

### V1.0 Coexistence

V1.0 is NOT deprecated. It continues to manage all existing workspaces. V2.0
is built inside a v1.0 workspace. When v2.0 reaches feature parity with v1.0's
core workflows, a migration path will be defined. Until then, both versions
operate independently.

## Consequences

### Positive

- Deterministic builds: same spec → same code, every time
- Token cost proportional to problem novelty, not problem count
- Self-contained distribution: download one file, run it
- State survives git operations naturally
- Real-time response to events

### Negative

- V1.0's social metaphor (teams, reviews, gatekeepers) — which some users find
  intuitive — is gone
- The DSL spec language is a new thing to learn
- Pattern database needs seeding before the solver is useful
- Rust development velocity is slower than Python for prototyping
- Two versions to maintain during the transition period

### Risks

- **Pattern DB cold start**: Until the known-patterns database has sufficient
  coverage, most tasks will fall back to AI, negating the solver benefit.
  Mitigation: seed with patterns from kodehold v1.0's existing workspaces.
- **DSL expressiveness**: If the spec language can't express real-world
  requirements, users will bypass it. Mitigation: start minimal, expand based
  on real usage.
- **V1.0 abandonment**: If v2.0 takes too long to become useful, momentum may
  be lost. Mitigation: ship Phase 1 (core binary) within 4 weeks.
