---
status: Superseded
superseded-by: agentmemory (memory_signal with handoff type)
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0025: A2A Protocol (Agent-to-Agent Coordination)

## Status

Superseded

Replaced by agentmemory memory_signal (handoff/request/response types) + Director coordination. Task tool provides structured handoff. Sequential pipeline + Team Meeting implement the relevant workflow patterns. Parallel execution and specialized roles are over-engineering for current needs. GitHub issue #31 closed as over-engineering.

## Context

KodeHold uses the Task tool for delegation but lacks advanced multi-agent coordination patterns. The Director delegates to one team at a time, and teams return results sequentially. There is no mechanism for:

- Parallel agent work with result aggregation
- Agent-to-agent handoff without Director mediation
- Specialized agent roles beyond the 6 team types
- Coordinated multi-step workflows across agents
- Research agents that gather context before implementation agents use it

The current approach has these limitations:

- All coordination goes through the Director — single point of orchestration
- No parallel execution — teams work sequentially even when independent
- No structured handoff protocol — context is passed through Director summarization
- Limited to 6 team types — no specialization within teams
- Complex workflows require multiple Director rounds

The key forces are:

- The Director is the natural orchestrator, but adds latency for simple handoffs
- Some tasks are naturally parallel (e.g., writing tests while implementing features)
- Research → Implementation → Review is a common pattern that could be streamlined
- Agent-to-agent communication must be structured to prevent context drift
- The A2A (Agent-to-Agent) protocol from AI agent research provides proven patterns

## Decision

Implement an A2A protocol with specialized agent roles, coordinator patterns, and structured handoff protocols.

### Agent Role Extensions

| Role | Purpose | Trigger | Output |
|------|---------|---------|--------|
| **Research Agent** | Gather context before implementation | Design doc update | Research brief with findings |
| **Support Agent** | Assist main agent with subtasks | Complex delegation | Partial results |
| **Coding Agent** | Specialized implementation | Feature implementation | Code + tests |
| **Data Agent** | Data analysis and transformation | Data-related tasks | Analysis + recommendations |

### Coordinator Pattern

The Director acts as coordinator for multi-agent workflows:

```
Director (Coordinator)
  ├── Research Agent → findings
  ├── Coding Agent → implementation (uses findings)
  ├── Test Agent → test results (uses implementation)
  └── Review Agent → approval (reviews all above)
```

### Structured Handoff Protocol

When Agent A hands off to Agent B:

| Field | Description | Example |
|-------|-------------|---------|
| `context_summary` | What Agent A determined | "API needs 3 new endpoints" |
| `artifacts` | Files/data produced | `docs/api-spec.md` |
| `decisions` | Choices made | "Used REST over GraphQL" |
| `constraints` | Limitations to respect | "Must work with existing DB schema" |
| `next_steps` | What Agent B should do | "Implement endpoints per spec" |

### Parallel Execution Model

| Scenario | Parallel Agents | Aggregation |
|----------|----------------|-------------|
| Feature + Tests | Engineers + Testers | Director merges results |
| Research + Design | Research Agent + Architects | Director integrates findings |
| Multi-file implementation | Multiple Engineers | File-level independence |

### Workflow Patterns

| Pattern | Description | Agents | Director Role |
|---------|-------------|--------|---------------|
| **Pipeline** | Sequential handoff | Research → Coding → Testing → Review | Coordinator |
| **Fan-out** | Parallel independent tasks | Multiple Engineers | Aggregator |
| **Fan-in** | Multiple inputs, single output | Research + Coding → Review | Coordinator |
| **Hierarchical** | Agent delegates to sub-agents | Director → Team Lead → Team | Mediator |

### Communication Protocol

Agent-to-agent messages follow a structured format:

```yaml
from: <agent_id>
to: <agent_id>
type: handoff|request|response|notification
context:
  summary: <what this is about>
  artifacts: [<file paths>]
  decisions: [<decisions made>]
  constraints: [<limitations>]
priority: critical|high|medium|low
requires_response: true|false
```

### Integration Points

- **Task tool (existing):** A2A protocol extends Task tool with structured handoff
- **ICM (ADR-0004):** Agent messages are stored as episodic memories (ADR-0022)
- **Session compression (ADR-0019):** Multi-agent workflows are summarized in session context
- **Shared memory (ADR-0024):** Agent coordination respects lock state

### Implementation Plan

| File | Change |
|------|--------|
| director.md | Add coordinator pattern, parallel execution management, handoff protocol |
| engineers.md | Add sub-agent delegation, result aggregation |
| architects.md | Add research agent integration, design handoff |
| scribes.md | Add multi-agent event extraction, workflow summarization |
| design doc | Add section 7.11 — A2A Protocol |

## Consequences

- Positive: More sophisticated workflows — pipeline, fan-out, fan-in patterns
- Positive: Reduced Director latency for simple handoffs
- Positive: Specialized agent roles enable deeper expertise per task
- Positive: Parallel execution improves throughput for independent tasks
- Negative: Increases system complexity — more coordination overhead
- Negative: Structured handoff adds tokens to agent communication (~100-200 per handoff)
- Negative: Debugging multi-agent workflows is harder than sequential delegation
- Negative: Risk of context drift in long agent chains
- Neutral: Parallel execution benefits may be limited by sequential gate requirements
- Note: agentmemory memory_signal provides structured inter-agent communication (handoff, request, response, alert types).
