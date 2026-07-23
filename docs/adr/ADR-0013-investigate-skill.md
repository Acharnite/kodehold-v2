---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0013: Investigate Skill — Systematic Debugging

## Status

Accepted

## Context

KodeHold has three skills in `.opencode/skills/`: `icm-knowledge-flow`, `state-awareness`, and now `investigate`. The investigate skill was adapted from gstack's investigate skill ([gstack-opencodeai](https://github.com/Acharnite/gstack-opencodeai)) to provide systematic debugging within KodeHold's architecture.

Without a dedicated debugging protocol:

- Bugs are fixed by trial and error ("fix symptoms first, understand later")
- Root cause analysis is skipped in favor of "quick fixes"
- There is no structured escalation when debugging hits dead ends (3-strike rule)
- Debugging knowledge is lost between sessions — no structured storage of findings
- FLS and Engineers have no shared methodology for bug investigation

The key forces are:

- The skill must be generic enough for both FLS (hotfix triage) and Engineers (implementation debugging)
- It must not depend on gstack-specific infrastructure (gbrain, telemetry, artifacts sync, config system)
- It must integrate with KodeHold's ICM for storing investigation results
- It must be simple enough to load on-demand (0 token cost until invoked)
- The 4-phase methodology (investigate → analyze → hypothesize → implement) must be prescriptive but not over-engineered

## Decision

We create `.opencode/skills/investigate/SKILL.md` — a 157-line skill implementing the 4-phase systematic debugging protocol from gstack, adapted to KodeHold:

### Core Methodology

| Phase | Name | Purpose |
|-------|------|---------|
| Iron Law | — | No fixes without root cause investigation first |
| 1 | Root Cause Investigation | Collect symptoms, trace code, check git log, reproduce, check ICM |
| 2 | Pattern Analysis | Match against 8 known bug patterns or search externally |
| 3 | Hypothesis Testing | Confirm hypothesis, 3-strike rule, red flags |
| 4 | Implementation | Minimal diff, regression test, full test suite |
| 5 | Verification & Report | Structured DEBUG REPORT, store findings in ICM |

### Adaptations from gstack

| gstack feature | KodeHold adaptation |
|---------------|-------------------|
| gbrain / telemetry / artifacts sync | Removed entirely |
| Freeze scope lock system | Removed (no freeze system in KodeHold) |
| Plan mode / routing hooks | Removed |
| Config system (`gstack-config`) | Removed |
| Learning storage via `gstack-learnings-log` | Replaced with `icm_memory_store` in central ICM |
| Prior investigation search via gbrain | Replaced with `icm_memory_recall` |
| Preamble bash script (~80 lines) | Removed — skill loads clean with no runtime overhead |
| 8 bug patterns (race, null, state, integration, config, cache, off-by-one, resource) | Kept as-is |

### Integration into KodeHold

Four agents receive `skill: allow` permission and workflow references:

| Agent | When to use investigate |
|-------|----------------------|
| **FLS** | During minor hotfix triage when root cause is unclear |
| **Engineers** | When implementation task involves fixing a bug |
| **Reviewers** | When investigating reported code issues |
| **Director** | When orchestrating root cause analysis (via trigger mapping) |

The Director's trigger mapping gains: `Investigate / root cause → engineers or fls via investigate skill`.

AGENTS.md quick reference gains: `Investigate/Debug | engineers or fls via investigate skill | Root cause first, fix second`.

### Discoverability

OpenCode auto-discovers skills from `.opencode/skills/<name>/SKILL.md`. No registration in `opencode.json` is needed. The skill appears in the `<available_skills>` tool description when agents have `skill: allow` permission, and can be loaded via `skill({ name: "investigate" })`.

### Agent Permission Configuration

All subagents previously had no `skill` key in their YAML frontmatter `permission` block. The following agents now explicitly set `skill: allow`:

- Director
- FLS
- Engineers
- Reviewers

Architects, Testers, and Scribes do not need the skill (they design, test, and document — they do not debug).

## Consequences

- Positive: Systematic debugging methodology reduces trial-and-error fixes
- Positive: 3-strike rule prevents infinite debugging loops
- Positive: Structured DEBUG REPORT is stored in ICM for future reference
- Positive: Shared methodology across FLS and Engineers
- Positive: On-demand loading means 0 token cost when not debugging
- Negative: Skill depends on agent remembering to load it proactively (mitigated by workflow references in FLS and Engineers agent definitions)
- Neutral: Gstack's rich preamble (update checks, telemetry, config) is removed for simplicity — users who want those features should use gstack directly
