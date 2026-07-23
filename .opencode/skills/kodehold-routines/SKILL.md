---
name: kodehold-routines
description: Standard workflow routine definitions for KodeHold. Load this skill when you need the step-by-step tables for ADR creation, implementation, bugfix, shipping gate, or GitHub PR flows.
---

# KodeHold Routines

## How to Use

The Director loads this skill, reads the relevant step table, and delegates each step sequentially via the Task tool.

1. **Trigger detection** — User says a trigger phrase matching a routine (see Detection Triggers in director.md)
2. **Load this skill** — `skill("kodehold-routines")`
3. **Read the table** — Find the matching routine below
4. **Delegate sequentially** — Each step via Task tool, respecting `Depends On` constraints
5. **Track progress** — Use `todowrite` after each step
6. **Branching (bugfix-flow only)** — Evaluate triage result at branch point: minor → hotfix path, major → REOPEN path

### Mandatory Pre-Step for All Routines

**Before delegating Step 1 of any routine, the Director MUST perform a memory search:**

```
search_memories(query="<routine-topic> <project> lessons bugs", scope="project")
```

Capture the results and pass them as `Relevant Context` in the Step 1 Task prompt.
This ensures the team receives prior learnings before starting work.

The memory search is NOT listed as a numbered step in each table — it is an
invisible pre-step that applies to ALL routines. Skipping it defeats the purpose
of persistent memory.

---

## `kodehold-adr-flow` (5 steps)

**Version:** 1.1 | **Category:** design | **Author:** Architects

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | architects | research + write-adr | (none) | 8 | No |
| 2 | scribes | design-doc-update | step 1 | 5 | No |
| 3 | reviewers | review-adr | step 1 | 7 | No |
| 4 | second-opinion | cross-validate | step 1 | 7 | Yes |
| 5 | scribes | finalize | steps 3, 4\* | 5 | No |

\*If step 4 is skipped, step 5 depends only on step 3.

**Parameters:** `title` (required), `require_second_opinion` (boolean, default true)

**Note:** Steps 1-2 collapsed per user feedback — Architects receives all context in a single delegation to eliminate information loss between sequential calls.

---

## `kodehold-implement-flow` (6 steps)

**Version:** 1.0 | **Category:** implement | **Author:** Architects

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | architects | design | (none) | 8 | No |
| 2 | reviewers | design-review | step 1 | 7 | Yes |
| 3 | engineers | implement | step 2\* | 8 | No |
| 4 | reviewers | code-review | step 3 | 7 | No |
| 5 | testers | test | step 3 | 6 | No |
| 6 | reviewers | gate-validation | steps 4, 5 | 9 | No |

\*If step 2 is skipped, step 3 depends on step 1 directly. Steps 4-5 are parallel (fan-out). Step 6 is fan-in.

**Parameters:** `feature_description` (required), `skip_design_review` (boolean, default false)

---

## `kodehold-bugfix-flow` (5 steps, branching)

**Version:** 1.0 | **Category:** bugfix | **Author:** FLS

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | fls | triage | (none) | 7 | No |
| 2a | fls | hotfix | step 1 | 8 | No (minor path) |
| 3 | scribes | document | step 2a | 5 | No (minor path) |
| 4 | reviewers | verify | step 2a | 7 | No (minor path) |
| 2b | — | → REOPEN + implement-flow | step 1 | — | No (major path) |

**Branching:** The Director evaluates the triage result and chooses the path. If `severity < threshold`, follow steps 2a → 3 → 4 (minor fix). If `severity >= threshold`, step 2b triggers a REOPEN state transition followed by `kodehold-implement-flow` (major fix). The template cannot auto-branch — the Director must evaluate and choose.

**Parameters:** `issue_ref` (required), `severity_threshold` (int, default 7)

---

## `kodehold-ship-gate` (7 steps)

**Prerequisite:** Step 0 (Team Meeting, ADR-0011) must be completed before instantiation.

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | director | version-check | (none) | 9 | No |
| 2 | director | changelog-check | (none) | 9 | No |
| 3 | director | todo-check | (none) | 9 | No |
| 4 | testers | test-suite | (none) | 9 | No |
| 5 | director | git-status | (none) | 9 | No |
| 6 | director | branch-check | (none) | 9 | No |

All 6 steps have no dependencies — they can run in parallel (fan-out).

**Parameters:** `version` (required), `project` (required)

---

## `kodehold-github-pr-flow` (8 steps)

**Version:** 1.0 | **Category:** pr | **Author:** Engineers

| Step | Team | Action | Depends On | Priority | Optional? |
|------|------|--------|------------|----------|-----------|
| 1 | engineers | create-branch | (none) | 8 | No |
| 2 | engineers | push-commits | step 1 | 8 | No |
| 3 | engineers | create-pr | step 2 | 8 | No |
| 4 | reviewers | request-copilot-review | step 3 | 7 | Yes |
| 5 | engineers | address-feedback | step 3 | 8 | Yes |
| 6 | engineers | update-branch | step 5\* | 7 | Yes |
| 7 | reviewers | merge-pr | steps 3, 6\* | 9 | No |
| 8 | scribes | cleanup | step 7 | 5 | No |

\*If step 6 is skipped, step 7 depends only on step 3.

**Parameters:** `base_branch` (required), `branch_name` (required), `pr_title` (required), `pr_body` (required)
