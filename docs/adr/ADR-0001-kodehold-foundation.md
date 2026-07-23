---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0001: KodeHold Foundation and Principles

## Status

Accepted

## Context

KodeHold is a new coding orchestrator that applies conscious team-based software engineering to AI-assisted development. The project needs a clear foundation: a set of principles that guide all future decisions, tool choices, and architectural direction. Without explicit principles, trade-off decisions become inconsistent and the system drifts from its original intent.

The key forces are:
- AI development is token-expensive; every operation must be efficient
- AI agents lack long-term memory across sessions without persistent storage
- LLMs hallucinate less when guided by structured, reviewed design documents
- Different LLMs have different strengths; the system should not be locked to one provider
- Projects need to be long-lived — closed today, reopened months later with full context

## Decision

We establish six core principles that govern all KodeHold development:

1. **Design-First** — Every project starts with and revolves around a living design document. No implementation work begins without an approved design.
2. **Separation of Concerns** — Distinct teams handle design, implementation, review, testing, and memory. No single agent performs all roles.
3. **Token-Conscious** — Every operation is evaluated for token cost. RTK is the default CLI interface. Token budgets are tracked and enforced.
4. **Persistent Memory** — ICM stores all project context, decisions, and rationale across sessions. Context is never lost when a project is closed.
5. **LLM-Agnostic** — Core architecture works with any LLM. Ollama is the primary provider. The system supports switching models per team and requesting second opinions from different models.
6. **Traceable Decisions** — All architectural decisions are recorded as ADRs in `docs/adr/` following the Nygard format. Every significant choice has a rationale trail.

These principles are immutable. Any future ADR that conflicts with a principle must explicitly state the override and its justification.

## Consequences

- Positive: Clear decision framework; consistent architecture; easier onboarding
- Positive: Token costs are managed proactively rather than reactively
- Positive: Design documents become the single source of truth, reducing ambiguity
- Negative: Design-first approach adds overhead before any code is written
- Negative: Separation of concerns means more orchestration steps per task
- Neutral: Principles may need revisiting as the project matures; override mechanism exists via explicit ADR
