# Architecture Decision Log

This directory contains Architecture Decision Records (ADRs) for the KodeHold project.

Each ADR follows the Nygard format: **Status | Context | Decision | Consequences**.

## Active ADRs

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-0001](ADR-0001-kodehold-foundation.md) | KodeHold Foundation and Principles | Accepted | 2026-05-25 |
| [ADR-0002](ADR-0002-architecture-and-teams.md) | Organizational Structure — Director and Teams | Accepted | 2026-05-25 |
| [ADR-0003](ADR-0003-design-document-lifecycle.md) | Design Document Lifecycle | Accepted | 2026-05-25 |
| [ADR-0005](ADR-0005-llm-support.md) | LLM Support and Light Mode | Accepted | 2026-05-25 |
| [ADR-0006](ADR-0006-second-opinion.md) | Second Opinion Protocol | Accepted | 2026-05-25 |
| [ADR-0008](ADR-0008-project-lifecycle.md) | Project Lifecycle and Reopening | Accepted | 2026-05-25 |
| [ADR-0010](ADR-0010-fls-front-line-support.md) | FLS — Front Line Support Team | Accepted | 2026-05-27 |
| [ADR-0011](ADR-0011-team-meeting.md) | Team Meeting — Collective Project Review | Accepted | 2026-05-27 |
| [ADR-0012](ADR-0012-adopted-projects.md) | Adopted Projects — Existing Codebases in KodeHold | Accepted | 2026-05-27 |
| [ADR-0013](ADR-0013-investigate-skill.md) | Investigate Skill — Systematic Debugging | Accepted | 2026-05-27 |
| [ADR-0015](ADR-0015-director-delegation-enforcement.md) | Director Delegation Enforcement via Tool Permissions | Accepted | 2026-05-28 |
| [ADR-0016](ADR-0016-early-review-gates.md) | Early Review Gates in ACTIVE Phase | Accepted | 2026-05-28 |
| [ADR-0017](ADR-0017-reviewers-gatekeeper-and-mandatory-second-opinion.md) | Reviewers as Gatekeeper + Mandatory Second Opinion | Accepted | 2026-05-28 |
| [ADR-0018](ADR-0018-centralize-documentation-under-scribes.md) | Centralize All Documentation Work Under Scribes | Accepted | 2026-05-28 |
| [ADR-0021](ADR-0021-prospective-memory.md) | Prospective Memory (Task Queue & Scheduler) | Accepted | 2026-05-29 |
| [ADR-0032](ADR-0032-routine-templates.md) | Routine Templates for Standard Flows | Accepted | 2026-05-31 |
| [ADR-0034](ADR-0034-workflow-monitor-interface.md) | Workflow Monitor Interface | Accepted | 2026-06-01 |
| [ADR-0036](ADR-0036-project-slug-convention.md) | Project Slug Convention — Stable Canonical Identifiers | Accepted | 2026-06-02 |
| [ADR-0037](ADR-0037-yaml-configuration.md) | YAML-Based Agent and Task Configuration | Accepted | 2026-06-02 |
| [ADR-0038](ADR-0038-knowledge-recall.md) | Knowledge Recall Protocol | Accepted | 2026-06-03 |
| [ADR-0039](ADR-0039-pre-flight-enforcement.md) | Pre-Flight Knowledge Check Enforcement | Accepted | 2026-06-03 |
| [ADR-0041](ADR-0041-procedural-consolidation-fix.md) | Procedural Consolidation Tier — Bridge Pattern Detection to Pipeline | Accepted | 2026-06-04 |
| [ADR-0042](ADR-0042-adr-implementation-phase-board.md) | ADR Implementation Phase Board | Accepted | 2026-06-05 |
| [ADR-0045](ADR-0045-memory-remember-relations-patch.md) | Patch mem::remember to Create KV.relations Entry on Supersede | Accepted | 2026-06-06 |
| [ADR-0046](ADR-0046-automatic-git-init-workspace.md) | Automatic Git Repository Initialization for Workspace Management | Accepted | 2026-06-13 |
| [ADR-0047](ADR-0047-universal-test-execution-standard.md) | Universal Test Execution Standard | Accepted | 2026-06-13 |
| [ADR-0048](ADR-0048-mandatory-documentation-review.md) | Mandatory Tool Documentation Review Before Implementation | Accepted | 2026-06-13 |
| [ADR-0049](ADR-0049-lazy-senior-dev-philosophy.md) | Lazy Senior Dev Philosophy | Accepted | 2026-06-19 |
| [ADR-0050](ADR-0050-agentmemory-to-opencode-rag-migration.md) | Agentmemory → OpenCode RAG Migration | Accepted | 2026-06-27 |
| [ADR-0051](ADR-0051-opencode-mem-persistent-memory.md) | opencode-mem as KodeHold Persistent Memory Backend | Accepted | 2026-07-01 |
| [ADR-0052](ADR-0052-structured-durable-execution.md) | Structured Durable Execution — Formal Checkpoint Schema and Auto-Checkpoint | Accepted | 2026-07-02 |
| [ADR-0053](ADR-0053-replace-ollama-with-vllm.md) | Hybrid Embedding Strategy — sentence-transformers + Ollama | Accepted | 2026-07-09 |
| [ADR-0054](ADR-0054-replace-opencode-rag-with-graphify.md) | Replace opencode-rag with Graphify Knowledge Graph | Accepted | 2026-07-14 |
| [ADR-0055](ADR-0055-kodehold-improvements.md) | KodeHold Improvement Opportunities | Accepted | 2026-07-21 |
| [ADR-0056](ADR-0056-agent-config-cleanup.md) | Agent Configuration Cleanup | Accepted | 2026-07-21 |
| [ADR-0057](ADR-0057-migrate-file-memory-to-opencode-mem.md) | Migrate File-Based Memory to opencode-mem | Accepted | 2026-07-21 |
| [ADR-0060](ADR-0060-loop-engineering-integration.md) | Loop-Engineering Integration for Workspace Management | Accepted | 2026-07-22 |

**37 Accepted** — total 37 active ADRs.

## Inactive ADRs

23 ADRs have been moved to [inactive/](./inactive/) as they are superseded,
deprecated, or archived (ICM/agentmemory era designs, loop-engineering rollback). See [inactive/README.md](./inactive/README.md)
for the full index with reasons.
