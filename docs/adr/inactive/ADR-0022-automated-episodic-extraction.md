---
status: Superseded
superseded-by: agentmemory (agentmemory-capture.ts plugin)
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0022: Automated Episodic Extraction

## Status

Superseded

Replaced by agentmemory-capture.ts plugin (automatic observation capture via tool.execute.after hooks) — tool.execute.after hooks capture observations automatically. Agentmemory's memory_recall and memory_smart_search provide structured retrieval without manual extraction.

## Context

KodeHold requires manual `icm_memory_store` calls to persist session context. Scribes handles documentation post-task, but there is no automated extraction of episodic memories — what happened, when, and why. Session summaries (ADR-0019) compress chat but don't extract structured episodic memories.

The current approach has these limitations:

- Session summaries are prose — not structured, queryable episodic memories
- No automatic extraction of decisions, errors, or successes from delegation rounds
- Manual memory storage relies on Scribes remembering to capture important events
- Historical context is lost if Scribes doesn't explicitly store it
- Cross-session pattern recognition is impossible without structured episodic data
- The ICM knowledge flow skill provides manual extraction steps, but no automation

The key forces are:

- Episodic memory (what happened) is distinct from semantic memory (what is true)
- Session context compression (ADR-0019) compresses chat but doesn't extract events
- ICM already supports structured storage — natural fit for episodic data
- Automated extraction reduces burden on Scribes and ensures completeness
- Too much extraction creates noise — must filter for significant events only

## Decision

Implement post-session hooks that automatically extract episodic memories (decisions, errors, successes) and store them in ICM with structured tags.

### Event Types

| Type | Description | Example |
|------|-------------|---------|
| **Decision** | Architectural or design decision made | "Chose hierarchical memory over flat storage" |
| **Error** | Bug encountered and resolved | "Fixed gate.sh --yes flag ordering" |
| **Success** | Feature completed or milestone reached | "ADR-0019 implemented and tested" |
| **Transition** | State change occurred | "INIT → ACTIVE gate passed" |
| **Delegation** | Team work completed | "Architects completed design doc update" |
| **Conflict** | Disagreement or issue resolved | "Second opinion reconciled design choice" |

### Extraction Triggers

| Trigger | When | What to Extract |
|---------|------|-----------------|
| Delegation round complete | After each Task tool call returns | Team work summary, files changed, decisions |
| State transition | After `gate.sh --transition` succeeds | Transition details, gate results, team status |
| Error encountered | After `investigate` skill completes | Root cause, fix applied, lessons learned |
| Session end | Before context compression | Full session event summary |
| Explicit request | User or Director requests | On-demand extraction of specific events |

### Storage Structure

Each episodic memory is stored in ICM with:

```
Topic: kodehold-<project>-episodes
Tags: ["episodic", "type:<decision|error|success|transition|delegation|conflict>"]
Importance: high (for decisions and errors), medium (for others)
Metadata: {
  timestamp: <when>,
  teams_involved: [<team names>],
  files_changed: [<file paths>],
  outcome: <result>,
  context: <what led to this>
}
```

### Filtering Rules

Not every event becomes an episodic memory. Filtering criteria:

| Include | Exclude |
|---------|---------|
| Decisions with rationale | Routine file reads |
| Errors with root cause | Expected warnings |
| Feature completions | Individual line changes |
| State transitions | Debug/test output |
| Conflicts resolved | Successful routine operations |

### Integration Points

- **ADR creation (ADR-0023):** New ADRs trigger semantic memory extraction (separate from episodic)
- **Session compression (ADR-0019):** Episodic memories provide structured data for summaries
- **Hierarchical memory (ADR-0020):** Episodic memories follow tier classification rules
- **Prospective memory (ADR-0021):** Failed episodic events can spawn deferred tasks

### Implementation Plan

| File | Change |
|------|--------|
| scribes.md | Add episodic extraction workflow, event type taxonomy, filtering rules |
| director.md | Add extraction triggers at delegation rounds and state transitions |
| investigate SKILL.md | Add episodic extraction step after root cause analysis |
| design doc | Add section 7.8 — Automated Episodic Extraction |

## Consequences

- Positive: Richer memory history — every significant event is captured with structure
- Positive: Reduces Scribes burden — automation handles routine extraction
- Positive: Structured data enables cross-session pattern recognition
- Positive: Episodic memories complement session summaries (ADR-0019) with event-level detail
- Negative: May create noise if filtering rules are too aggressive — needs tuning
- Negative: Extraction overhead adds tokens to each delegation round (~50-100 tokens)
- Negative: ICM storage grows faster with automated extraction
- Neutral: Filtering thresholds may need adjustment based on actual project patterns
- Note: Automated extraction now handled by agentmemory-capture.ts plugin.
