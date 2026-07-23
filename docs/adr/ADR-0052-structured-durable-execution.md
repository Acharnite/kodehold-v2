# ADR-0052: Structured Durable Execution — Formal Checkpoint Schema and Auto-Checkpoint

## Status

Accepted

**Accepted Date:** 2026-07-02
**Proposed Date:** 2026-07-02

## Context

### The Problem

KodeHold has an ad-hoc checkpoint protocol defined in `director.md` and `scribes.md`, but it has never been formalized, enforced, or made machine-validatable. The result is **zero working checkpoint infrastructure** — the `.opencode/memory/checkpoints/` directory exists but is **empty** (confirmed: 0 files). Resume (`/resume` skill) reads from this directory and cannot function.

The existing protocol has fundamental gaps:

1. **No formal schema.** The checkpoint format is a loose mix of YAML frontmatter (`type`, `project`, `state`, `created`) and freeform markdown body. There is no schema_version, no field validation, and no machine-parseable contract. Agents produce inconsistent checkpoints that cannot be reliably consumed by the resume skill.

2. **Manual trigger only.** The Director must count delegation rounds in-head (every 8 for checkpoint, every 4 for compression). This is fragile — a single missed count means no checkpoint is stored. There is no automatic checkpoint-on-delegation-boundary mechanism.

3. **No thread tracking.** Checkpoints are stored flat in `.opencode/memory/checkpoints/<session-id>.md` with no concept of a `thread_id` to group related checkpoints across a delegation chain. A complex delegated task that spawns sub-tasks creates orphaned checkpoints with no parent-child linkage.

4. **No incomplete work detection.** When the Director delegates a task and the session ends before the task completes, there is no way to detect this on resume. The checkpoint shows "in-progress" items in the body, but there is no structured `status` field that a machine can query to determine "this task was interrupted."

5. **Resume is non-functional.** The resume skill (`.opencode/skills/resume/SKILL.md`) reads `.opencode/memory/checkpoints/*.md`, but no files exist. Even if files existed, the skill has no way to determine which checkpoint to load (no ordered thread history), and no way to detect incomplete work.

6. **No retention policy enforcement.** The protocol says "max 10 checkpoint files" but there is no code or automation to enforce this. In practice, checkpoints accumulate indefinitely (or never get written at all).

### Key Forces

1. **Checkpoints are the safety net for small-context models.** KodeHold supports Ollama (32K context) where every delegation round consumes ~1-2K tokens. After 10-15 rounds, context overflows. Checkpoints are the only mechanism to safely restart a session without losing work-in-progress.

2. **Scribes should write checkpoints automatically.** The Director should not have to remember to trigger checkpoints. The act of delegation (Task tool call) should implicitly trigger a checkpoint write. This is analogous to a database WAL — every transaction writes a log entry.

3. **Machine-validatable schema is essential for cross-agent consumption.** The resume skill, the Director, and Scribes all read checkpoints. A formal schema with defined fields, types, and validation rules ensures that any agent can reliably parse any checkpoint.

4. **Thread context must survive session boundaries.** When a complex task is interrupted (session overflow, model timeout, user interruption), the delegation chain must be reconstructable from checkpoint history. A `thread_id` + `parent` reference achieves this without requiring a graph database.

5. **Detection of incomplete work is critical.** The Director must know, on session start, whether the previous session ended with a task still in progress. Without this, the user must manually describe where they left off.

6. **Checkpoints must be lightweight.** A checkpoint write must not add significant latency to delegation. The format must be concise (200-400 tokens), and the write must be delegated to Scribes asynchronously.

7. **Backward compatibility.** The existing checkpoint directory structure (`.opencode/memory/checkpoints/`) must be preserved. Existing files (if any) must remain readable. The schema evolves, not abandons, the existing format.

### Relationship to Existing ADRs

| ADR | Status | Relationship |
|-----|--------|--------------|
| **ADR-0019** (Session Context Compression) | **Superseded** | Proposed schema replaces the unstructured summary template from ADR-0019. The session summary sections (Completed, InProgress, Decisions, Files, Teams, Blockers, CarryForward, TokenUsage) are retained as structured body fields, not loose markdown. |
| **ADR-0050** (RAG Migration) | Accepted | §5 (File-Based Persistent Storage) defined checkpoints as flat markdown. This ADR supersedes §5's checkpoint format with formal schema + thread_id + auto-trigger. The storage path remains `.opencode/memory/checkpoints/`. |
| **ADR-0051** (opencode-mem) | Accepted | opencode-mem handles runtime memory (auto-capture, semantic search). Checkpoints are **complementary** — they provide durable, structured session history that opencode-mem does not. Checkpoints are the source of truth for session state; opencode-mem provides fuzzy recall over that state. |
| **Resume Skill** | Existing | Must be updated to use new schema, thread-aware ordering, and incomplete-work detection. |

## Decision

### 1. Formal YAML Frontmatter Schema

Every checkpoint file MUST begin with YAML frontmatter enclosed in `---` delimiters. The frontmatter defines the checkpoint's metadata and is machine-validatable against a defined schema.

#### Schema Definition

```yaml
---
# REQUIRED FIELDS
type: checkpoint                   # Fixed: "checkpoint"
schema_version: 1                  # Integer, increments on schema changes
thread_id: "adr-0052-20260702"     # Unique delegation chain identifier
step: 3                            # Integer, sequential step within thread
parent: "adr-0052-20260702/2"      # Previous step's path (null/empty for step 1)
created: "2026-07-02T14:30:00Z"    # ISO 8601 timestamp

# REQUIRED — project context
project: "kodehold"                # Project slug (ADR-0036)
state: "ACTIVE"                    # Lifecycle state: INIT|ACTIVE|REVIEW|CLOSED|REOPEN
status: "completed"                # "in_progress" | "completed" | "failed" | "consolidated"

# OPTIONAL — delegation metadata
delegation:
  team: "scribes"                  # Team that performed the work
  task: "Write ADR-0052"          # Brief description of the delegated task
  duration_minutes: 45             # Approximate duration of the delegation

# OPTIONAL — parent session context
session:
  round: 12                        # Global delegation round counter
  compression_round: 3             # Compression cycle number (resets on state transition)
---
```

**Field Descriptions:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Must be `"checkpoint"`. Discriminator for parsing. |
| `schema_version` | integer | Yes | Schema version number. Start at 1. Increment on breaking changes. |
| `thread_id` | string | Yes | Unique identifier for the delegation chain. Format: `<project>-<purpose>-<yyyymmdd>` or a UUID. Grouped by delegation context. |
| `step` | integer | Yes | Sequential step number within the thread. Starts at 1. |
| `parent` | string | Yes | Path to the previous checkpoint in the thread. Format: `<thread_id>/<step-1>`. Empty string for step 1. |
| `created` | string | Yes | ISO 8601 UTC timestamp of checkpoint creation. |
| `project` | string | Yes | Project slug per ADR-0036. |
| `state` | string | Yes | KodeHold lifecycle state at time of checkpoint. One of: INIT, ACTIVE, REVIEW, CLOSED, REOPEN. |
| `status` | string | Yes | Delegation status. One of: `in_progress` (task delegated but not yet completed), `completed` (task done), `failed` (task failed/errored), `consolidated` (this checkpoint was merged into a consolidated summary). |
| `delegation.team` | string | No | Team slug that received the delegated task. |
| `delegation.task` | string | No | One-line description of the delegated task. |
| `delegation.duration_minutes` | integer | No | Approximate elapsed time for the delegation. |
| `session.round` | integer | No | Global delegation round counter since last state transition. |
| `session.compression_round` | integer | No | Compression cycle counter, resets on state transition. |

**Validation Rules:**

1. `type` MUST be exactly `"checkpoint"`. Any other value is invalid.
2. `schema_version` MUST be a positive integer. Future versions MUST increment this field.
3. `thread_id` MUST be non-empty and match `^[a-zA-Z0-9_-]+$`.
4. `step` MUST be a positive integer. Within a thread, steps MUST be sequential (1, 2, 3, ...).
5. `parent` MUST be either empty string (step 1) or match `<thread_id>/<step-1>`.
6. `status` MUST be one of the four defined values.
7. `state` MUST be one of the five defined lifecycle states.
8. Timestamps MUST be ISO 8601 UTC (ending in `Z`).

#### Body Sections

After the frontmatter, the body contains these sections in order. Each section is optional but SHOULD be populated when relevant.

```markdown
## Completed
- Issue #59: Defined formal checkpoint schema (ADR-0052)
- Issue #35: Auto-checkpoint flow documented in director.md

## InProgress
- Issue #35: Resume skill update pending schema approval

## Decisions
- YAML frontmatter chosen over JSON for human readability
- thread_id uses slug-based naming for traceability

## Files
- docs/adr/ADR-0052-structured-durable-execution.md (new)

## Teams
- scribes: Wrote checkpoint file
- architects: Designed schema

## Blockers
- None

## CarryForward
- Resume skill must be updated to use schema_version-aware parsing
- Director delegation flow must be updated for auto-checkpoint

## TokenUsage
- Total: ~4,500
- scribes: 2,100
- architects: 2,400
```

**Section Descriptions:**

| Section | Required | Content |
|---------|----------|---------|
| `## Completed` | No | Bullet list of tasks completed in this delegation step |
| `## InProgress` | No | Bullet list of tasks started but not yet completed |
| `## Decisions` | No | Bullet list of decisions made during this delegation step |
| `## Files` | No | Bullet list of files created or modified, with change type |
| `## Teams` | No | Bullet list of teams involved and their contributions |
| `## Blockers` | No | Bullet list of blockers or open questions |
| `## CarryForward` | No | Bullet list of context needed for the next delegation step |
| `## TokenUsage` | No | Summary of token consumption (from token-usage.sh or inline estimates) |

**Consistency rule:** The body's `InProgress` section MUST be empty when frontmatter `status: completed`. The body's `Completed` section MUST be non-empty when frontmatter `status: completed`. These are logical invariants that parsers SHOULD verify.

### 2. Storage Path Convention

Checkpoints are stored at:

```
.opencode/memory/checkpoints/<thread_id>/<step>.md
```

Where:
- `<thread_id>` is the delegation chain identifier (from frontmatter)
- `<step>` is the zero-padded step number (e.g., `001`, `002`, ...)

**Examples:**

```
.opencode/memory/checkpoints/adr-0052-20260702/001.md
.opencode/memory/checkpoints/adr-0052-20260702/002.md
.opencode/memory/checkpoints/adr-0052-20260702/003.md
.opencode/memory/checkpoints/implement-auth-flow-20260702/001.md
```

**Rationale:**

1. **Thread grouping.** All checkpoints for a given delegation chain are in one directory, making it trivial to see the full history: `ls .opencode/memory/checkpoints/<thread_id>/`.

2. **Ordering.** Zero-padded step numbers ensure alphabetical ordering matches chronological ordering: `ls` returns steps in order.

3. **Directory-level operations.** Thread directories can be consolidated, archived, or deleted as units. The consolidation policy (max 10 per thread) operates at the thread directory level.

4. **Compatibility with existing tools.** `ls -t`, `search_semantic`, and `grep` work naturally with this structure.

**For backward compatibility**, existing checkpoints at `.opencode/memory/checkpoints/<session-id>.md` (flat format) remain readable. The resume skill MUST check both locations:
1. Check `.opencode/memory/checkpoints/` for thread directories
2. If none found, check `.opencode/memory/checkpoints/*.md` for legacy flat files

### 3. Auto-Checkpoint on Every Delegation

The Director implicitly stores a checkpoint on **every** Task tool delegation. This replaces the manual round-counting approach.

#### Flow

```
[Director delegates to Team via Task tool]
                        │
                        ▼
[Director sets status: "in_progress"]
                        │
                        ▼
[Team receives task, executes]
                        │
                        ▼
[Team returns result to Director]
                        │
                        ▼
[Director delegates to Scribes to write checkpoint]
  • Reads Task prompt for delegation metadata
  • Constructs YAML frontmatter:
    - thread_id: <generated or continued>
    - step: <sequential>
    - parent: <previous step path>
    - status: "completed"
    - delegation.team: <team name>
    - delegation.task: <task description>
  • Constructs body from:
    - Task result summary (Completed section)
    - Any new decisions made (Decisions section)
    - Files affected (Files section)
  • Writes to .opencode/memory/checkpoints/<thread_id>/<step>.md
                        │
                        ▼
[Director continues with next delegation]
```

**thread_id generation:**

- For the first task in a new delegation chain, the Director generates a `thread_id` using format: `<project>-<purpose>-<yyyymmdd>` where purpose is a 2-3 word slug of the task goal.
- For subsequent tasks in the same chain, the Director reuses the same `thread_id` and increments `step`.
- A new thread_id is created when the task purpose changes significantly (e.g., moving from design to implementation).

**parent field:**

- Step 1 in a thread has `parent: ""`.
- Step N in a thread has `parent: "<thread_id>/<N-1>"`.
- This creates a linked list through the delegation chain.

**status transition:**

1. Before delegating, the Director writes a checkpoint with `status: "in_progress"`. This is a **quick write** — only frontmatter, no body sections populated yet.
2. After the team returns, the Director delegates to Scribes to **update** the checkpoint: fill in body sections, set `status: "completed"`, add `delegation.duration_minutes`.
3. If the delegation fails (team returns error, tool timeout), the Director sets `status: "failed"` and includes error details in the `Blockers` section.

**Optimization for rapid delegations:**

For very short delegation rounds (under ~30 seconds, such as simple reads), the Director MAY skip the `in_progress` pre-checkpoint and write only the `completed` checkpoint. The threshold is: skip if the task is known to be quick (e.g., "read a file," "run a single command") and no state change is expected. The Director uses judgment — when in doubt, write the pre-checkpoint.

#### thread_id Lifecycle

```
User: "Implement login feature"
  → Director creates thread_id: "kodehold-implement-login-20260702"
  → Step 1: Architects design (status: completed)
  → Step 2: Engineers implement (status: in_progress)
    → [SESSION ENDS]
    → On resume: Director finds step 2 has status: "in_progress"
    → Director presents: "Previous session interrupted during Engineers implementation. Resume?"

User: "Fix the bug in auth"
  → Director creates thread_id: "kodehold-fix-auth-bug-20260702"
  → (Separate thread from the login feature above)
```

### 4. Incomplete Work Detection

The `status` field in the frontmatter is the key mechanism for detecting incomplete work.

#### Detection Flow

```
[Session starts]
        │
        ▼
[Director reads checkpoints]
  • Lists all threads in .opencode/memory/checkpoints/
  • For each thread, reads the highest-step file
  • Checks status field
        │
        ▼
[If any checkpoint has status: "in_progress"]
  → Collect all threads with incomplete work
  → For each thread, read the checkpoint body (InProgress section)
  → Present to user:
    "Found incomplete work from previous session:
     Thread: <thread_id>
     Step N: <delegation.task>
     Status: in_progress
     InProgress: <what was being worked on>
     Would you like to resume this work? [y/N]"
        │
        ▼
[User response]
  → Yes: Director loads the checkpoint context, continues delegation chain
  → No: Director marks checkpoints as status: "consolidated" to acknowledge abandonment
```

**Edge cases:**

| Scenario | Behavior |
|----------|----------|
| Multiple incomplete threads | List all, let user choose which to resume |
| Thread with status: "in_progress" but no body | Minimal context — present what's available from frontmatter |
| Thread with status: "failed" | Present as failed with error summary, ask if retry |
| User declines resume | Mark all incomplete checkpoints as `status: "consolidated"` to clear the detection |
| No checkpoints exist | Report "No previous session found. Starting fresh." |

### 5. Auto-Resume Protocol

On session start, the Director runs the auto-resume protocol BEFORE accepting any user input.

#### Protocol Steps

```
Step 1: Check for checkpoints
  ls .opencode/memory/checkpoints/*/   (detect thread directories)
  If none:
    ls .opencode/memory/checkpoints/*.md   (detect legacy flat files)
    Load legacy resume from resume skill
    Return

Step 2: Detect incomplete work
  For each thread directory:
    latest = last file by numeric sort
    read latest's frontmatter status field
    Collect all where status == "in_progress"

Step 3: Present to user
  If incomplete work found:
    "⚠️ Incomplete work detected from [timestamp]:
     Thread: <thread_id> — <task description>
     Status: in_progress at step N of M
     Previous: <parent checkpoint summary>
     
     Options:
     1. Resume — continue this thread
     2. Skip — mark as acknowledged, start fresh
     3. Inspect — show full checkpoint body"
  
  If all complete:
    "✅ All previous checkpoints completed. Last session: <thread_id> step N."
    Load latest checkpoint for context reconstruction.

Step 4: Load context
  If resuming:
    Load parent checkpoints (thread_id chain) for full context
    Set delegation state to continue from the incomplete step
  
  If starting fresh:
    Load latest checkpoint for summary only (no delegation state)

```

#### Resume skill update

The existing resume skill (`.opencode/skills/resume/SKILL.md`) MUST be updated to:

1. Support both legacy flat format (`<session-id>.md`) and new thread-based format (`<thread_id>/<step>.md`)
2. Parse YAML frontmatter for machine-readable status detection
3. Present incomplete work detection results
4. Fall back to legacy format gracefully

The resume skill update is integral to this ADR's end-to-end vision and is included in Phase 4 of the implementation plan.

### 6. Retention and Consolidation

#### Per-Thread Consolidation

When a thread exceeds 10 checkpoint files, Scribes consolidates the oldest 5 into a single consolidated file:

```
.opencode/memory/checkpoints/<thread_id>/001.md  (oldest, will be consolidated)
.opencode/memory/checkpoints/<thread_id>/002.md
.opencode/memory/checkpoints/<thread_id>/003.md
.opencode/memory/checkpoints/<thread_id>/004.md
.opencode/memory/checkpoints/<thread_id>/005.md
.opencode/memory/checkpoints/<thread_id>/006.md  ← becomes new step 1 after consolidation
.opencode/memory/checkpoints/<thread_id>/007.md
.opencode/memory/checkpoints/<thread_id>/008.md
.opencode/memory/checkpoints/<thread_id>/009.md
.opencode/memory/checkpoints/<thread_id>/010.md  ← newest, stays
```

**Consolidation process:**

1. Read checkpoints 001-005 (oldest 5).
2. Merge their body sections: combine `Completed` lists, keep all `Decisions`, keep latest `TokenUsage`.
3. Write a consolidated file at a new path: `.opencode/memory/checkpoints/<thread_id>/consolidated-001-005.md`.
4. Set frontmatter `status: "consolidated"` on the original 001-005 files (or delete them, per Director preference).
5. Renumber remaining active files (006-010 → 001-005) for sequential integrity.

**Alternative** (simpler, prefer this): Delete the oldest 5 files after consolidation. The consolidated file preserves all decisions and carry-forward context. Original files are not needed after consolidation.

#### Global Retention

- No hard global limit on total checkpoints (unlike the old "max 10" rule).
- Instead, consolidation is per-thread. Old threads naturally stop growing once the delegation chain completes.
- Orphaned threads (no activity for 30 days) MAY be archived to `.opencode/memory/checkpoints/archive/` by Scribes during maintenance.

#### Consolidation trigger

Scribes checks the checkpoint count on every write. If the thread directory exceeds 10 files, consolidation is triggered immediately (not deferred). This ensures the checkpoint directory never grows unbounded.

### 7. Implementation Plan

| Phase | Scope | Owner | Dependencies |
|-------|-------|-------|--------------|
| **Phase 1** | Write this ADR (ADR-0052) | Architects | None |
| **Phase 2a** | Update director.md checkpoint protocol — add auto-checkpoint on delegation, thread_id generation | Scribes | Phase 1 done |
| **Phase 2b** | Update director.md checkpoint protocol — add incomplete work detection and auto-resume protocol. **Also: create a validation script** (e.g., `.opencode/scripts/validate-checkpoint.sh`) that validates YAML frontmatter fields, required field presence, field types, and status enum values per Schema Definition §1. This script is used by the Director and Testers for verifying checkpoint correctness. | Scribes | Phase 2a done |
| **Phase 3** | Update scribes.md checkpoint workflow — new storage path, YAML frontmatter generation, consolidation policy | Scribes | Phase 1 done |
| **Phase 4** | Update resume skill — support new format, thread-aware parsing, incomplete work detection | Scribes | Phases 2a, 2b, 3 done |
| **Phase 5** | Update ADR-0019 status to Superseded (by this ADR) and ADR-0050 §5 note to acknowledge this ADR supersedes its checkpoint format | Scribes | Phase 1 done |
| **Phase 6** | Verification — end-to-end: delegate → checkpoint written → session end → resume → detect incomplete → continue | Testers + Reviewers | Phases 2a, 2b, 3, 4, 5 done |

**Migration:** The existing empty `.opencode/memory/checkpoints/` directory requires no migration. The first delegation after this ADR is accepted will create the first thread-based checkpoint. Legacy flat files (if any appear from manual creation) remain readable via the resume skill's fallback path.

### 8. Verification Criteria

The implementation is complete when:

1. **Schema validation:** A checkpoint file can be validated against the YAML frontmatter schema. Required fields are present. Field types match. Status values are in the defined set.

2. **Auto-checkpoint on delegation:** Every Task tool delegation produces a checkpoint file at `.opencode/memory/checkpoints/<thread_id>/<step>.md`.

3. **thread_id scoping:** New delegation chains produce new thread directories. Same-chain delegations reuse the same thread directory and increment step.

4. **Incomplete work detection:** A checkpoint with `status: "in_progress"` is detected on session start. The Director presents resume options to the user.

5. **Consolidation:** A thread with 11+ checkpoint files triggers consolidation of the oldest 5 into a consolidated summary.

6. **Resume flow:** The user can say "yes" to resume and the Director loads the thread context and continues delegation from the interrupted step.

7. **Backward compatibility:** Legacy flat-format checkpoints (if any) are still readable by the resume skill, which falls back gracefully.

## Consequences

### Positive

1. **Checkpoints work.** For the first time, `.opencode/memory/checkpoints/` will actually contain files. The resume skill will have data to read.

2. **No manual counting.** The Director no longer needs to count delegation rounds in-head. Every delegation implicitly produces a checkpoint via Scribes.

3. **Thread-aware session history.** The `thread_id` + `step` + `parent` structure creates a recoverable delegation chain. Complex workflows spanning multiple teams can be traced from start to finish.

4. **Incomplete work detection.** Session interruptions (context overflow, model timeout, explicit user request) are automatically detected on resume. The Director knows exactly where it left off.

5. **Machine-validatable schema.** The YAML frontmatter with defined fields, types, and validation rules can be parsed by any agent or script. The resume skill, the Director, Scribes, and third-party tools all consume the same format.

6. **Per-thread consolidation prevents unbounded growth.** Each thread is independently consolidated at 10 checkpoints. No global limit needed.

7. **Backward compatible.** Legacy flat-format checkpoints remain readable. The directory structure change (flat → thread-based) is additive, not breaking.

### Negative

1. **Increased delegation overhead.** Every delegation round now produces up to 2 checkpoint writes (in_progress pre-checkpoint + completed post-checkpoint). This adds ~1-2 seconds per delegation and consumes tokens for the checkpoint content.
   - *Mitigation:* Scribes writes are lightweight (200-400 tokens each). The in_progress pre-checkpoint is a minimal frontmatter-only write (~50 tokens). The benefit of durable execution far outweighs the token cost.

2. **thread_id generation is judgment-based.** The Director decides when to create a new thread vs. continue an existing one. Inconsistent thread boundaries could fragment session history.
   - *Mitigation:* Clear guidelines in director.md: new thread on task purpose change. The parent field ensures even fragmented threads are linked.

3. **Consolidation is destructive.** Oldest 5 checkpoints are deleted after consolidation. Full delegation history is lost for those steps.
   - *Mitigation:* The consolidated file preserves all decisions, carry-forward context, and a summary of completed work. For most use cases, this is sufficient. If full history is needed, the consolidation threshold can be increased (10 → 20) or deleted files can be archived instead.

4. **Resume skill must be updated.** The existing resume skill does not support the new format. Until Phase 4 is complete, resume is still broken.
   - *Mitigation:* Phases 1, 2a, 2b, and 3 must complete before Phase 4. The ADR formalizes the schema first; implementation follows.

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Schema adds too much delegation overhead | Low | Medium | in_progress pre-checkpoint is optional for short tasks; post-checkpoint is ~200-400 tokens |
| thread_id fragmentation makes history untraceable | Medium | Low | parent field links fragmented threads; Director guidelines are clear |
| YAML parsing fails on malformed frontmatter | Low | High | schema validation in resume skill; `---` delimiters are strict |
| Consolidation deletes important history | Low | Low | Consolidated file preserves all decisions; threshold can be adjusted |
| Director forgets to write pre-checkpoint before delegation | Medium | Medium | Task tool delegation CAN be wrapped in a protocol step in director.md; enforcement via agent prompt |
| Existing checkpoint directory with legacy files conflicts | Low | Low | Thread directories (name/) don't conflict with flat files (name.md). Namespace is disjoint. |

## Documentation

| Field | Value |
|-------|-------|
| **Concept** | Structured Durable Execution |
| **Key files** | `.opencode/memory/checkpoints/<thread_id>/<step>.md` |
| **Schema** | YAML frontmatter with 10 metadata fields + 8 body sections |
| **Validation** | `type: checkpoint`, `status` in {in_progress, completed, failed, consolidated}, ISO 8601 timestamps |
| **Known Gotchas** | • `parent` field must reference the PREVIOUS step in the same thread, not the thread directory |
| | • `status: "in_progress"` is set BEFORE delegation, `status: "completed"` AFTER. These are two separate writes. |
| | • Consolidation deletes the oldest 5 files. Archive to `.opencode/memory/checkpoints/archive/` if preservation is needed. |
| | • Legacy flat files at `.opencode/memory/checkpoints/*.md` are NOT migrated to thread format. The resume skill handles both. |

## ADR References

- **ADR-0019** (Session Context Compression) — **Superseded by this ADR.** The structured checkpoint schema replaces the unstructured session summary template. The compression frequency (per-4-rounds) is replaced by per-delegation auto-checkpoint. Consolidation policy is refined from global "max 10" to per-thread consolidation at 10.
- **ADR-0050** (Agentmemory → OpenCode RAG Migration) — §5 (File-Based Persistent Storage) checkpoint format is **superseded by this ADR.** The storage path remains `.opencode/memory/checkpoints/` but the format changes from flat `<session-id>.md` to `<thread_id>/<step>.md` with YAML frontmatter.
- **ADR-0051** (opencode-mem) — **Overrides §5.** ADR-0051 §5 deprecates `.opencode/memory/checkpoints/` in favor of opencode-mem for session context. This ADR **overrides that deprecation**. Rationale: opencode-mem handles runtime memory (auto-capture, semantic recall, cross-session context) while checkpoints handle **durable session execution state** (delegation chain tracking, thread_id scoping, incomplete work detection). These are complementary concerns — opencode-mem provides fuzzy recall over past context; checkpoints provide deterministic, machine-validatable session recovery. Both are needed; they do not conflict.

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.0 | 2026-07-02 | Initial ADR — Structured Durable Execution: formal checkpoint schema, auto-checkpoint on delegation, thread_id, incomplete work detection, auto-resume |
| 1.1 | 2026-07-02 | Review fixes: title aligned to "Auto-Checkpoint", ADR-0051 §5 override clarified, contradictory scope statement removed, Phase 2 sub-phased into 2a+2b with validation script added, dependencies updated |
