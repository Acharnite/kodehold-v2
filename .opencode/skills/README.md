# KodeHold Skills

This directory contains reusable KodeHold skills — packaged workflows that can be invoked by the Director or any team.

## Structure

Each skill lives in its own subdirectory with a `SKILL.md` entry point:

```
skills/
├── README.md               # This file — skill index
├── preflight/
│   └── SKILL.md            # Pre-task knowledge retrieval via Graphify + opencode-mem
├── investigate/
│   └── SKILL.md            # Systematic debugging with root cause investigation
├── kodehold-routines/
│   └── SKILL.md            # Standard workflow routine definitions
├── ponytail-review/
│   └── SKILL.md            # Over-engineering analysis for code review
└── state-awareness/
    └── SKILL.md            # Lifecycle state checking + mismatch protocol
```

## Available Skills

| Skill | Description |
|-------|-------------|
| [preflight](preflight/SKILL.md) | Pre-task knowledge retrieval via Graphify + opencode-mem cross-reference. |
| [investigate](investigate/SKILL.md) | Systematic debugging with root cause investigation. Iron Law: no fixes without root cause. 4 phases: investigate → analyze → hypothesize → implement. Adapted from gstack. |
| [ponytail-review](ponytail-review/SKILL.md) | Over-engineering analysis for code review. Companion to The Ladder (ADR-0049). Tags diffs with delete:/stdlib:/native:/yagni:/shrink: tags. Loaded by Reviewers during Ladder compliance checks. |
| [kodehold-routines](kodehold-routines/SKILL.md) | Standard workflow routine step tables for ADR creation, implementation, bugfix, shipping gate, and GitHub PR flows. Load when the Director needs to instantiate a routine. |
| [state-awareness](state-awareness/SKILL.md) | Lifecycle state check preamble and mismatch reporting protocol. Used by all team subagents. |
