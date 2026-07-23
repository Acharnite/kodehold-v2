# ADR-0058: Loop Engineering Integration & Token Budget Protocol v2

## Version
- **v1.0 (2026-07-21):** Original — `opencode run` via loop-run.sh, token caps per loop
- **v1.1 (2026-07-22):** Updated to match implementation — Python loop_runner.py, Discord webhook, 3 patterns running, 4 new patterns added

## Status

Accepted

## Context

### The Problem

KodeHold operates as a structured software organization — Director orchestrates tasks, Architects design, Engineers implement, Reviewers validate. But the system is **entirely human-triggered**. Every task delegation, every session start, every review cycle requires a human to prompt the Director. There is no autonomous scheduling, no unattended maintenance, and no systematic feedback loop between completed work and new work.

Meanwhile, the **Loop Engineering** framework (cobusgreyling/loop-engineering, 8.9k stars) provides a mature pattern for designing autonomous agent systems: "Stop prompting. Design the loop. Get a score." Loop Engineering's five building blocks — Scheduling, Worktrees, Skills, MCP Connectors, Sub-agents + Memory/State — map directly onto KodeHold's existing architecture. KodeHold already has 4 of 5 building blocks implemented; it is missing only **Scheduling** and **Worktrees**.

### KodeHold vs Loop Engineering Mapping

| Loop Building Block | KodeHold Status | Detail |
|---------------------|----------------|--------|
| **Scheduling** | MISSING | No cron/systemd, no `opencode run` integration |
| **Worktrees** | MISSING | No git worktree isolation for parallel work |
| **Skills** | PRESENT | `.opencode/skills/` — 7 skills loaded on-demand |
| **MCP Connectors** | PRESENT | Graphify, opencode-mem, GitHub MCP servers |
| **Sub-agents + Memory/State** | PRESENT | 6 teams, Director orchestrator, `.kodehold-state`, opencode-mem |

### Loop Engineering Toolchain

The loop-engineering ecosystem provides structured tooling:

| Tool | Purpose | KodeHold Equivalent |
|------|---------|-------------------|
| `loop-init` | Scaffold new loop | `workspace.sh init` |
| `loop-audit` | Loop Readiness Score (0-100) | gate.py (binary pass/fail, no score) |
| `loop-cost` | Token estimation per run | None |
| `loop-sync` | Drift detection (state vs doc) | Manual ADR/design doc review |
| `loop-gate` | Declarative gate enforcement | gate.py (hardcoded Python) |
| `loop-context` | Memory manager | opencode-mem (via MCP) |
| `loop-worktree` | Isolated git worktrees | None |

### Why Now

ADR-0007 (Token Optimization Strategy) was written in May 2025 during INIT phase. It defines a solid vocabulary-level optimization strategy (English-only, chunking, tiered loading) and per-phase token budgets. However, it is **entirely manual** — budgets are guidelines, not enforced constraints. No automation budget exists. No kill switch. No per-automation-run caps.

KodeHold has evolved significantly since ADR-0007:
- **ADR-0051**: opencode-mem replaced file-based memory → token savings
- **ADR-0054**: Graphify replaced opencode-rag for code retrieval → deterministic, token-efficient queries
- **ADR-0057**: Migrated file memory to opencode-mem, removed checkpoint/compression protocols → leaner context loading
- **ADR-0049**: The Ladder coding philosophy → less code, fewer tokens

The system is now mature enough for automation. Loop Engineering provides the missing operational layer.

### Key Forces

1. KodeHold has 4 of 5 loop-engineering building blocks already — integration is additive, not replacement
2. Automation WITHOUT budget controls risks runaway token consumption
3. Automation WITHOUT drift detection risks executing against stale designs
4. Gate.py is 771 lines of hardcoded Python — declarative gates would be more maintainable and loop-gate compatible
5. Per-automation token caps and a kill switch are essential safety mechanisms for unattended operation
6. KodeHold's `.kodehold-state` is machine-readable but not human-friendly for loop state tracking

## Decision

### 1. Adopt Loop Engineering as KodeHold's Operational Framework

KodeHold IS loop engineering with a strong team metaphor. The missing pieces are the **automation layer** (scheduling) and **isolation layer** (worktrees). This ADR defines a three-phase integration roadmap.

**Phase definition principle:** Each phase must produce a self-contained, independently-usable result. No phase should depend on a future phase for completeness.

### 2. Three-Phase Integration Roadmap

#### Phase 1: Foundation (Current — Immediate)

**Goal:** Establish baseline metrics and standards. Zero automation — manual runs only.

| Milestone | What | Priority |
|-----------|------|----------|
| **P1.1** Run `loop-audit` on KodeHold | Establish baseline Loop Ready Score. Document gaps. Target ≥80 before Phase 2. Note: `loop-audit` CLI availability should be validated (`npx @cobusgreyling/loop-audit --version`) before this milestone begins — do not assume the tool works without verification. | MUST |
| **P1.2** Write `config/gate.yaml` | Declarative gate definitions (schemas, markers, checks). Compatible with loop-gate. Does NOT replace gate.py yet — runs alongside it. | MUST |
| **P1.3** Token Budget Protocol v2 | Modernize ADR-0007 budgets (see §6 below). | MUST |
| **P1.4** Write `STATE.md` | Human-readable loop state file alongside `.kodehold-state`. Documents active loops, last run times, health status. Updated by Scribes after each loop iteration. | SHOULD |

**Phase 1 completion gate:** Loop Ready Score documented + gate.yaml validated.

#### Phase 2: Automation (Requires Phase 1 Complete)

**Goal:** L1 autonomous loops (report-only, no unattended writes). Cron-based.

| Milestone | What | Autonomy |
|-----------|------|----------|
| **P2.1** Daily Triage Loop | Cron job runs `opencode run` with a triage prompt every morning. Agent reads recent `search_memories`, checks for stale PRs, failing CI, uncommitted ADRs. Produces a report (stored via `add_memory` and appended to `loop-run-log.md`). No automated fixes. | L1 (report-only) |
| **P2.2** PR Babysitter Loop | Cron job checks open PRs → updates stale PRs, flags merge conflicts, notifies if review >24h pending. | L1 (report-only) |
| **P2.3** loop-sync Drift Detection | Weekly cron job compares design doc ADR index vs actual files, state file vs marker files, TODO.md vs completed ADRs. Reports drift without fixing. | L1 (report-only) |

**Phase 2 completion gate:** Three loops running for 7 consecutive weekdays with at least one meaningful finding or an explicit "no issues found" report per loop per day (empty runs do not count as successful). Loop Ready Score re-evaluated.

#### Phase 3: Deep Integration (Requires Phase 2 Complete)

| Milestone | What | Autonomy |
|-----------|------|----------|
| **P3.1** Worktree Isolation | Use `git worktree` for Engineer isolation. Each Engineer task gets a clean worktree. Requires separate ADR (detailed scope: worktree lifecycle, cleanup, conflict handling). | N/A (isolation only) |
| **P3.2** Goal Mode Skill | `.opencode/skills/goal-mode/SKILL.md` — defines "run until condition met" patterns. Uses prospective memory for tracking. Loop exits when condition satisfied, budget exhausted, or human-gate reached. | L2 (assisted) |
| **P3.3** Declarative Gate Migration | gate.py fully replaced by `config/gate.yaml` + loop-gate validation. gate.py becomes a thin wrapper. | N/A (infrastructure) |

**Phase 3 completion gate:** Worktree ADR accepted, Goal Mode skill functional.

### 3. Gate Definition: Declarative gate.yaml

Migrate from hardcoded gate.py (771 lines) to declarative `config/gate.yaml`:

```yaml
# config/gate.yaml — Declarative gate definitions
# Compatible with loop-gate. gate.py wraps this file.

version: "1.0"

transitions:
  INIT_TO_ACTIVE:
    markers_required:
      - .design_reviewed
      - .second_opinion_done
    markers_cleanup:
      - .design_reviewed
      - .second_opinion_done
    checks:
      design_doc_exists: { type: file_exists, path: "docs/design/README.md" }
      design_doc_active: { type: grep, path: "docs/design/README.md", pattern: "Status.*Active" }
      design_sections: { type: sections, path: "docs/design/README.md", required: [summary] }
      adr_dir_exists: { type: dir_exists, path: "docs/adr" }
      adr_count_min: { type: file_count, path: "docs/adr", pattern: "ADR-*.md", min: 1 }
      adr_index_exists: { type: file_exists, path: "docs/adr/README.md" }
    interactive: true

  ACTIVE_TO_REVIEW:
    markers_required:
      - .testers_done
      - .code_reviewed
    markers_cleanup:
      - .testers_done
      - .code_reviewed
    checks:
      todo_exists: { type: file_exists, path: "TODO.md", warn_only: true }
      tests_pass: { type: command, run: "python3 scripts/gate.py --tests-only" }
      review_commits: { type: git_log_recent, pattern: "review|reviewed|approve", warn_only: true }
```

**Migration strategy:** gate.yaml runs alongside gate.py (dual validation, assert parity). After Phase 3.3, gate.py becomes a thin wrapper that parses gate.yaml.

### 4. Scheduling: cron + loop_runner.py

All scheduled loops use `crontab` entries invoking `scripts/loop_runner.py` — a pure Python CLI that runs git/gh/pytest commands directly via `subprocess`. NO `opencode run` calls in L1 loops.

#### 4.1 Loop Runner Architecture

`scripts/loop_runner.py` provides 3 patterns (7 defined, 3 implemented):

| Pattern | CLI Command | Status |
|---------|-----------|--------|
| Daily Triage | `python3 scripts/loop_runner.py daily-triage` | ✅ L1 |
| PR Babysitter | `python3 scripts/loop_runner.py pr-babysitter` | ✅ L1 |
| Drift Detection | `python3 scripts/loop_runner.py drift-detection` | ✅ L1 |
| CI Sweeper | — | ❌ Planned |
| Issue Triage | — | ❌ Planned |
| Changelog Drafter | — | ❌ Planned |
| Dependency Sweeper | — | ❌ Planned |

Each pattern:
1. Executes direct bash/git/gh/pytest commands (subprocess, ~30-120s timeout)
2. Appends structured output to `loop-run-log.md` with JSON summary block
3. Creates `.loop_error` marker on non-zero exit
4. Optionally sends Discord embed via `--webhook` flag

#### 4.2 Notification

When `--webhook` is passed, loop_runner.py posts a color-coded Discord embed:

| Outcome | Color | Meaning |
|---------|-------|---------|
| Clean | Green (#00ff00) | No findings, exit 0 |
| Issues found | Orange (#ffa500) | Findings detected |
| Error | Red (#ff0000) | Non-zero exit |

Webhook URL stored in `config/loop-webhook.txt` (gitignored).

#### 4.3 Crontab Entries

```bash
# Daily Triage — every weekday at 08:00
0 8 * * 1-5 cd /home/kiffer/project/kodehold && python3 scripts/loop_runner.py daily-triage --webhook 2>&1 | tee -a /tmp/loop-cron.log

# PR Babysitter — every 4 hours during working hours
0 8,12,16 * * 1-5 cd /home/kiffer/project/kodehold && python3 scripts/loop_runner.py pr-babysitter --webhook 2>&1 | tee -a /tmp/loop-cron.log

# Drift Detection — every Sunday at 10:00
0 10 * * 0 cd /home/kiffer/project/kodehold && python3 scripts/loop_runner.py drift-detection --webhook 2>&1 | tee -a /tmp/loop-cron.log
```

All loops run at L1 (report-only). L2/L3 gated behind proven reliability.

#### 4.4 Backwards Compatibility

`scripts/loop-run.sh` is retained as a thin wrapper that delegates to `loop_runner.py`:
```bash
exec python3 scripts/loop_runner.py "$@"
```

#### 4.5 Terminal-Based Invocation

For ad-hoc runs:
- `workspace.py loop <name> <pattern>` — runs loop_runner.py inside workspace
- Direct: `python3 scripts/loop_runner.py <pattern> [--workspace <name>] [--webhook] [--dry-run]`

### 5. Worktree Isolation

Engineer tasks use `git worktree` for isolation. Each task creates a new worktree, works in it, and the result is merged back. This prevents in-progress work from interfering with other tasks.

**Requires separate ADR** (scope too large for this ADR). Key considerations:
- Worktree lifecycle: create → work → commit → merge → cleanup
- Concurrent worktree limit (disk space, git index locks)
- Conflict resolution strategy when multiple worktrees modify same files
- Integration with Engineer team workflow (step added before implementation starts)

> **Status:** Not yet started — deferred to Phase 3.

### 6. Token Budget Protocol v2

Modernize ADR-0007's budgets with loop-engineering integration:

#### 6.1 Per-Phase Budgets (Carried Forward from ADR-0007)

| Phase | Max Tokens | Notes |
|-------|-----------|-------|
| Context load | 8k | Design doc + ADRs + search_memories summary |
| Code generation | 12k | Spec + constraints + implementation |
| Code review | 8k | Diff + standards + ADR compliance |
| Test generation | 8k | Spec + code + edge cases |
| Documentation | 4k | Code + decisions |
| Second opinion | 6k | Cross-model validation |

**Budget enforcement:** Per-phase budgets are **guidelines** — they trigger warnings at 80% and alerts at 100%, but only the 200% threshold is a hard stop that refuses delegation. The 8k context load budget is intentionally conservative; exceeding it does not block work. Budgets serve as awareness tools, not enforcement gates.

#### 6.2 Per-Automation-Run Caps

**Note:** The original per-loop token caps (Daily Triage 3k, PR Babysitter 2k, Drift Detection 4k) were design-time estimates for `opencode run` invocations. Since migrating to Python `loop_runner.py`, per-invocation token tracking has not been implemented. The caps remain aspirational guidelines for future L2/L3 automation.

| Loop | Max Tokens/Run | Status |
|------|---------------|--------|
| Daily Triage | 3k (aspirational) | Not enforced |
| PR Babysitter | 2k (aspirational) | Not enforced |
| Drift Detection | 4k (aspirational) | Not enforced |

#### 6.3 Kill Switch: loop-pause-all

**Not yet implemented.** The `.loop_pause_all` marker is defined in the design but was never added to loop_runner.py. When implemented:
- All cron jobs skip execution (check at start)
- Director warns on session start that automation is paused
- Manual operations continue unaffected
- Removed after 24h auto-expiry or manual `rm`

#### 6.4 Director's Warning Protocol (Updated)

Before each delegation, Director:
1. Checks against per-phase budget
2. **80% of budget** → warns user: "Approaching token budget for `<phase>`: `<current>`/`<max>`"
3. **100% of budget** → alerts user: "Token budget exceeded. Consider context compression or smaller scope."
4. **200% of budget** → hard stop: refuses delegation, suggests session reset

### 7. State Management: Dual-State Files

Maintain two state files with different purposes:

| File | Purpose | Format | Readers |
|------|---------|--------|---------|
| `.kodehold-state` | Machine-readable lifecycle state | `KEY=VALUE` lines | gate.py, scripts, cron jobs |
| `STATE.md` | Human-readable loop state | Markdown with sections | Director, human operators |

**STATE.md template:**

```markdown
# KodeHold Loop State

**Last updated:** 2026-07-21T08:00:00Z
**Loop Ready Score:** TBD (awaiting Phase 1.1)

## Active Loops

| Loop | Schedule | Status | Last Run | Engine | Docs |
|------|----------|--------|----------|--------|------|
| Daily Triage | Weekdays 08:00 | Pending | — | loop_runner.py | — |
| PR Babysitter | Every 4h, 08-16 | Pending | — | loop_runner.py | — |
| Drift Detection | Sunday 10:00 | Pending | — | loop_runner.py | — |

## Health

| Check | Status |
|-------|--------|
| Tests passing | yes |
| Gate markers clean | yes |
| No drift detected | yes |
| Token budget OK | yes |

## Recent Loop Reports

<!-- Scribes appends loop reports here as they are generated -->
```

Scribes updates `STATE.md` after each loop iteration (Phase 2+) and after lifecycle transitions. `STATE.md` content is also stored via `add_memory(tags=["loop_state"])` for searchability.

### 8. Scoring: Loop Ready Score

Run `loop-audit` on KodeHold to establish baseline. The tool assesses:

1. **Loop Definition Clarity** — Are loops well-specified? (Daily Triage, PR Babysitter — to be defined in Phase 2)
2. **Memory Architecture** — Persistent state across runs? (opencode-mem via MCP)
3. **Error Handling** — Graceful failure? (Kill switch, caps, FLS escalation)
4. **Observability** — Can you see what's happening? (STATE.md, loop reports, token tracking)
5. **Safety** — Can it damage itself? (Markers, gates, report-only L1 mode)

**Score:** 99/100 (L1) at Phase 2 completion (strong on Memory, Skills, Sub-agents; Scheduling now implemented via loop_runner.py, Observability via Discord webhook).

**Next target:** ≥95 for Phase 3 (requires worktree isolation + Goal Mode).

### 9. Deprecate ADR-0007

ADR-0007 (Token Optimization Strategy) is superseded by this ADR:

- **English-only policy** → absorbed into ADR-0049 (The Ladder) and general KodeHold conventions
- **Tiered context loading** → superseded by Graphify (ADR-0054) + opencode-mem (ADR-0051) retrieval
- **Minimal prompt templates** → replaced by The Ladder's YAGNI principle (ADR-0049)
- **File chunking >150 lines** → superseded by Graphify's structural queries
- **Context deduplication** → handled by opencode-mem auto-capture
- **Per-phase token budgets** → modernized and extended in this ADR (§6)

ADR-0007 status changes from **Accepted** to **Superseded** with reference to this ADR.

### 10. Future Work (Phase 4+ — Out of Scope)

| Topic | Description | When |
|-------|-------------|------|
| Fleet Management | Multi-project orchestration (loop-engineering → fleet-engineering) | Post-Phase 3 |
| Goal Mode (L3 unattended) | "Run until condition met" with full autonomy | Post-Phase 3, requires extensive safety testing |
| L2 Assisted Fixes | PR Babysitter auto-fixes merge conflicts, stale branch cleanup | Post-Phase 2, after L1 proven stable |
| Outerloop Integration | harness-foundry → outerloop for multi-loop coordination | Beyond fleet management |

### 11. Pattern Portfolio

KodeHold currently implements 3 of 7 loop-engineering patterns. The full portfolio:

| Pattern | Cadence | L Level | Implemented | File |
|---------|---------|---------|-------------|------|
| Daily Triage | 1d | L1 report | ✅ | `loop_runner.py` |
| PR Babysitter | 4h | L1 watch | ✅ | `loop_runner.py` |
| Drift Detection | 7d | L1 report | ✅ | `loop_runner.py` |
| CI Sweeper | 15m | L2 fix | ❌ | — |
| Dependency Sweeper | 6h | L2 patch | ❌ | — |
| Changelog Drafter | 1d | L1 draft | ❌ | — |
| Issue Triage | 2h | L1 propose | ❌ | — |
| Post-Merge Cleanup | 1d | L1 clean | ❌ | — |

**Description of new patterns (from cobusgreyling/loop-engineering):**

- **CI Sweeper (L2):** React to CI failures. When CI fails, create a worktree, fix the failing test/lint, push the fix, verify CI passes. Requires worktree isolation (P3.1).
- **Dependency Sweeper (L2):** Monitor dependency updates. Auto-create PRs for patch security bumps. Verify tests pass before merging.
- **Changelog Drafter (L1):** After each release tag, aggregate commits since last tag, categorize (Added/Changed/Fixed), draft CHANGES.md entry. Human approves before merge.
- **Issue Triage (L1):** Scan new GitHub issues, classify (bug/feature/question), check for duplicates, suggest labels. Report summary to human.
- **Post-Merge Cleanup (L1):** After merge, delete stale branches, close related issues, update project boards.

### 12. Documentation

Update the documentation table to current versions:

| Field | Value |
|-------|-------|
| **Tool** | Loop Engineering (cobusgreyling/loop-engineering) |
| **Official docs** | https://github.com/cobusgreyling/loop-engineering |
| **Version documented** | v1.6.0+ (2026-07-22) |
| **KodeHold patterns implemented** | 3 of 7 (daily-triage, pr-babysitter, drift-detection) |
| **Key integration points** | loop_runner.py, workspace.py loop, crontab, config/loop-webhook.txt |
| **CLI commands** | `python3 scripts/loop_runner.py [daily-triage\|pr-babysitter\|drift-detection] [--webhook] [--workspace <name>]` |

## Consequences

### Positive

- **Operational maturity:** KodeHold graduates from a solely human-triggered system to one with autonomous maintenance loops. Design work (Architects) and implementation (Engineers) remain human-triggered; only maintenance/observation is automated.
- **Safety-first automation:** All automation starts at L1 (report-only). L2/L3 requires proven reliability. Kill switch (`loop_pause_all`) provides emergency stop with 24-hour auto-recovery. Per-run caps prevent runaway loops. Cron wrapper script creates `.loop_error` markers for FLS triage on failure.
- **Declarative gates:** `config/gate.yaml` makes gate definitions auditable, diffable, and loop-gate compatible — replacing 771 lines of Python with a structured YAML file + thin wrapper.
- **Drift detection:** loop-sync prevents the "design doc says X, code does Y" problem that arises when loops run against stale context.
- **Aligned with ecosystem:** Adopting loop-engineering patterns makes KodeHold compatible with the broader agent engineering toolchain (loop-audit, loop-cost, loop-gate).
- **Discord notifications:** Color-coded alerts on every loop run. Green (clean), Orange (issues), Red (error). Immediate visibility without checking logs.

### Negative

- **Integration complexity:** Three phases spanning weeks/months. Phase dependencies must be respected (Phase 2 cannot start without Phase 1's baseline score and token tracking).
- **Token overhead of loops:** Daily Triage + PR Babysitter + Drift Detection add ~9k tokens/day (3k + 2k + 4k). Even at conservative rates (~$3/M tokens), this adds marginal cost. But for local Ollama models, context window pressure is the real concern.
- **Cron fragility:** Cron jobs have no intrinsic error handling. A failed `opencode run` invocation could silently fail. Mitigation: The `scripts/loop-run.sh` wrapper script logs exit codes and creates `.loop_error` markers on failure (preventive, not reactive). Additional mitigations: loop-pause-all auto-created on token cap breach; STATE.md tracks last run times; FLS triages `.loop_error` markers.
- **Design document impact:** This ADR's adoption requires updates to `docs/design/README.md`: ADR Index (add ADR-0058), Token Optimization section (mark as superseded by ADR-0058), Architecture Overview (add loop-engineering scheduling layer), and Implementation Plan (add Phase 1-3 roadmap milestones).

### Risks

- **Loop runaway:** An unsupervised loop could consume tokens indefinitely if a bug in `opencode run` causes it to never exit. **Mitigation:** Per-run token caps (§6.2) + kill switch with 24-hour auto-recovery (§6.3). Cron jobs have a secondary timeout via `timeout` command in the wrapper script.
- **State drift during automation:** A loop could act on stale state if `.kodehold-state` is not synced. **Mitigation:** loop-sync drift detection (Phase 2.3) catches this before loops make decisions.
- **Worktree isolation complexity:** git worktrees introduce new failure modes (locked index files, disk space exhaustion, merge conflicts). **Mitigation:** Separate ADR for worktree design. Limited worktree count (max 3 concurrent). Cleanup cron job.
- **gate.yaml/gate.py divergence:** During the dual-validation period, if gate.yaml and gate.py disagree, which one is authoritative? **Mitigation:** gate.py is authoritative until Phase 3.3. gate.yaml is validated for parity but not enforced. Integration tests assert parity.
- **Concurrent write protection:** `.kodehold-state` and `STATE.md` are both updated during loop runs — without locking, concurrent writes could corrupt state. **Mitigation:** Cron jobs are staggered (no overlapping intervals). Scribes updates are serialized through Director delegation. If concurrent writes become a problem, a `.state_lock` file-based mutex can be added.

## Effort

| Phase | Milestone | Estimate | Dependencies |
|-------|-----------|----------|-------------|
| P1.1 | loop-audit baseline | 1 session | Validate `loop-audit` CLI availability first |
| P1.2 | gate.yaml | 2 sessions | Analyze all 5 gate.py transitions |
| P1.4 | ADR-0007 deprecation | Done (this ADR) | — |
| P1.5 | STATE.md | 1 session | Scribes workflow update |
| P2.1 | Daily Triage Loop | 2 sessions | Phase 1 complete, cron + wrapper setup |
| P2.2 | PR Babysitter Loop | 1 session | P2.1 patterns established |
| P2.3 | loop-sync | 2 sessions | Design doc/index analysis |
| P3.1 | Worktree ADR + implementation | 3+ sessions | Separate ADR, extensive testing |
| P3.2 | Goal Mode Skill | 2 sessions | Prospective memory (ADR-0021) |
| P3.3 | Gate Migration | 1 session | gate.yaml validated over months |

**Phase 1 total:** ~4 sessions (1-2 weeks).
**Phase 2 total:** ~6 sessions (2-3 weeks, plus 7-day burn-in).
**Phase 3 total:** ~6+ sessions (3+ weeks, worktree ADR is largest unknown).

## References

- Loop Engineering: https://github.com/cobusgreyling/loop-engineering
- Loop Engineering Ecosystem: memory-engineering → loop-engineering → harness-foundry → outerloop → fleet-engineering
- ADR-0007: Token Optimization Strategy (superseded by this ADR)
- ADR-0021: Prospective Memory (Task Queue & Scheduler) — used by Goal Mode
- ADR-0048: Mandatory Tool Documentation Review Before Implementation — requires `## Documentation` section in ADRs that select external tools
- ADR-0049: The Ladder — YAGNI, deletion over addition
- ADR-0051: opencode-mem Persistent Memory Backend
- ADR-0054: Graphify Knowledge Graph for Code Retrieval
- ADR-0057: Migrate File-Based Memory to opencode-mem
- ADR-0059: Workspace as Self-Contained Loop-Ready Mini KodeHold
- scripts/loop_runner.py: Python loop runner implementation

## Review Notes

- **2026-07-21 (v1):** Initial proposal. Integrates loop-engineering as KodeHold's operational framework. Defines three-phase roadmap with clear gates. Modernizes token budget protocol from ADR-0007. Adds per-automation-run caps, kill switch, STATE.md, and declarative gate.yaml. ADR-0007 status updated to Superseded.
- **2026-07-21 (v2):** Revised per Reviewers (BLOCKING: missing `## Documentation` section per ADR-0048 §3) and Second Opinion (4 must-fix, 3 should-fix items). Changes: (1) Added `## Documentation` section with loop-engineering reference, API concepts, prerequisites, and 5 gotchas. (2) Clarified per-phase budgets as guidelines (warnings at 80%, alerts at 100%, hard stop at 200%). (3) Added 24-hour auto-expiry to `.loop_pause_all` kill switch with `touch`-to-extend. (4) Added `scripts/loop-run.sh` cron wrapper script with exit code logging and `.loop_error` marker creation. (5) Added `loop-run-log.md` as plain-text fallback for `add_memory` failures. (6) Tightened Phase 2 completion gate to require meaningful findings or explicit "no issues found" per loop per day over 7 consecutive weekdays. (7) Noted `opencode run --prompt` CLI syntax must be verified before deployment. (8) Added "Impact on Design Document" to Consequences. (9) Noted concurrent write protection for dual-state files as a risk with staggered-cron mitigation.
- **2026-07-21 (Accepted):** ADR accepted after Reviewers PASS (v2) and Second Opinion approval.
- **2026-07-22 (v1.1):** Updated to match reality. §4 replaced (loop_runner.py instead of opencode run). §5 marked as deferred. §6.2/§6.3 marked as aspirational/not implemented. §7 STATE.md table updated. §8 score changed to 99/100. §11 Pattern Portfolio added. §12 Documentation added. Old Documentation section removed. Discord notifications added to Consequences.
