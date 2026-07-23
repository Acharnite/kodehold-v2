# ADR-0049: Adopt Ponytail "Lazy Senior Dev" Philosophy as KodeHold Coding Philosophy

## Status

Accepted

## Context

### The Problem

KodeHold currently has eight design principles (documented in §2 of the design doc) that govern project structure, lifecycle, token optimization, and safety. However, there is **no explicit coding philosophy** for the implementation teams. When Engineers write code, they do so without a structured decision framework that asks:

1. Does this need to be built at all?
2. Does the standard library already cover this?
3. Does a platform feature already exist?
4. Can this be expressed as one line?
5. Only then — write the minimum code that works.

The absence of this ladder leads to concrete problems: unnecessary abstractions, dependency bloat, over-engineering, unrequested generality, and boilerplate. These are the natural output of LLM agents trained to be comprehensive and anticipatory.

### The Opportunity

The [Ponytail project](https://github.com/DietrichGebert/ponytail) defines a coding philosophy called **"The Ladder"** that fills this gap precisely. Its core principles have been independently benchmarked:

| Metric | vs No-Skill Baseline |
|--------|---------------------|
| LOC reduction | −54% mean |
| Token reduction | −22% |
| Cost reduction | −20% |
| Speed improvement | −27% |
| Safety maintained | 100% |

### Why This Fits KodeHold

KodeHold and Ponytail are **complementary**: KodeHold provides organizational structure (6 teams, Director, lifecycle, gates), while Ponytail provides the coding philosophy (what to build and how). KodeHold is the orchestrator; Ponytail is the coding stance.

### What We Are NOT Adopting

We adopt **philosophy only**, NOT infrastructure:

| Feature | Decision | Rationale |
|---------|----------|-----------|
| "The Ladder" (YAGNI → stdlib → platform → deps → one line → minimum) | **Adopt** | Core decision framework |
| Companion rules (no unrequested abstractions, no new deps, boring over clever) | **Adopt** | Actionable heuristics |
| "Not lazy about" list (security, validation, error handling) | **Adopt** | Essential safety boundary |
| `ponytail:` comments | **Adopt** | Lightweight shortcut notation |
| Plugin system, hooks, lifecycle events | **Skip** | Overlaps OpenCode's plugin system |
| Mode system (lite/full/ultra/off) | **Skip** | Conflicts with KodeHold light/full mode |
| Config file (`~/.config/ponytail/config.json`) | **Skip** | Against self-contained design |
| MCP server | **Skip** | KodeHold doesn't use MCP servers |
| 14-agent portability | **Skip** | KodeHold is its own orchestrator |
| `/ponytail-review` / `/ponytail-audit` commands | **Defer** | Can become a skill in future |

## Decision

### 1. Add Principle #9 to Design Document

Insert into `docs/design/README.md` §2 Principles table:

```
| 9 | **Lazy Senior Dev** | Before writing code, ascend "The Ladder": YAGNI → stdlib → platform → existing deps → one line → minimum code. Boring over clever, deletion over addition. Never lazy about security, validation, error handling, or accessibility. |
```

### 2. Integrate "The Ladder" into engineers.md

Insert into `.opencode/agents/engineers.md` after workflow step 2b (documentation reading), before step 3:

```markdown
2c. **Apply "The Ladder" (ADR-0049)** — before writing any code, ascend these rungs. Stop at the first that holds:
    1. **Does this need to exist?** (YAGNI) — if no, skip it entirely.
    2. **Does the standard library already do this?** Use it.
    3. **Does a native platform feature cover it?** Use it (e.g., `<input type="date">` over a date picker library).
    4. **Does an already-installed dependency solve it?** Use it before adding new ones.
    5. **Can this be one line?** Make it one line.
    6. **Only then:** write the minimum code that works.
    - No abstractions that were not explicitly requested.
    - No new dependency if it can be avoided.
    - No boilerplate nobody asked for.
    - Deletion over addition. Boring over clever. Fewest files possible.
    - Pick edge-case-correct when two stdlib approaches are the same size.
    - Mark intentional simplifications with a `ponytail:` comment — name the ceiling and upgrade path.
    - **NOT lazy about:** trust-boundary input validation, error handling that prevents data loss, security, accessibility, anything explicitly requested in the design doc.
```

Add to Constraints section:
```markdown
- **The Ladder (ADR-0049)** — ascend before every implementation.
```

### 3. Add Over-Engineering Checks to reviewers.md

Add to Review Checklist:
```markdown
- [ ] **The Ladder compliance (ADR-0049)** — verify implementation ascends the ladder:
  - Could this have been done with stdlib? If yes, why was a dependency introduced?
  - Are there abstractions not explicitly requested in the design doc?
  - Are there `ponytail:` comments documenting intentional shortcuts?
  - Does every new dependency have clear stdlib justification?
  - Edge-case-correctness verified — if stdlib offered two same-sized approaches, was the more correct one chosen?
- [ ] **"Not lazy about" check** — minimal code must still handle: trust-boundary validation, data-loss error handling, security, accessibility.
```

### 4. Add Philosophy Reference to director.md

Add to Delegation Pattern Task prompt template (after Relevant files):
```markdown
- **Coding philosophy:** The Ladder (ADR-0049) — ascends before implementation. Reviewers check for compliance.
```

Update Trigger → Team Mapping table:
```
| Implementation | `engineers` → `scribes` (post-task) | Apply The Ladder (ADR-0049) |
| Code/design review | `reviewers` → `scribes` (post-task) | Verify Ladder compliance (ADR-0049) |
```

### 5. Decision Framework for Conflicts

The Ladder prioritizes minimal code, but code size is not the only virtue. When a
minimal solution conflicts with another priority, use this framework:

```
┌─────────────────────────────────────────────────────┐
│                    CONFLICT DETECTED                 │
│  Minimal code vs. [Maintainability / Performance /   │
│                      Readability / Correctness]      │
└───────────────────────┬─────────────────────────────┘
                        ▼
┌─────────────────────────────────────────────────────┐
│   Q1: Is the minimal solution demonstrably wrong?   │
│      (wrong output, crashes, data loss, security     │
│       hole, or accessibility failure)                │
└───────────────────────┬─────────────────────────────┘
                        │
           ┌────────────┴────────────┐
           ▼                         ▼
        YES ═══════════════╗      NO ═══════════════╗
   ┌──────────────────┐    │  ┌──────────────────┐  │
   │ FIX correctness  │    │  │ Q2: Does the     │  │
   │ first. Then      │    │  │ conflict arise   │  │
   │ re-evaluate size.│    │  │ from a *measured*│  │
   └──────────────────┘    │  │ bottleneck?      │  │
                           │  │ (profiler data,  │  │
                           │  │ latency p99,     │  │
                           │  │ memory heap)     │  │
                           │  └────────┬─────────┘  │
                           │           │             │
                           │  ┌────────┴────────┐    │
                           │  │ YES             NO  │
                           │  ▼                  ▼  │
                           │  ╔══════════════╗  ╔═══╩════════════╗
                           │  ║ Apply minimal ║  ║ Keep minimal  ║
                           │  ║ *optimization* ║  ║ solution.     ║
                           │  ║  that solves  ║  ║ Add ponytail: ║
                           │  ║  the measured ║  ║ comment with  ║
                           │  ║  bottleneck.  ║  ║ the ceiling   ║
                           │  ║  Document why ║  ║ and why the   ║
                           │  ║  minimal was  ║  ║ optimization  ║
                           │  ║  insufficient ║  ║ is deferred.  ║
                           │  ╚══════════════╝  ╚════════════════╝
                           │
                           │  For maintainability/readability conflicts:
                           │  If two solutions are functionally equivalent
                           │  and similar in size, prefer the one that is
                           │  easier to read and modify — even if slightly
                           │  more verbose. "Boring over clever" wins ties.
                           │
                           │  For performance conflicts WITHOUT profiler data:
                           │  Keep minimal. Premature optimization is the
                           │  root of all evil (Knuth). Add ponytail: comment
                           │  noting the performance ceiling.
                           └────────────────────────────────────────────────
```

**Key rules of thumb:**
- **Correctness always wins.** A minimal solution that is wrong is not minimal — it's broken.
- **Profile before optimizing.** Without profiler data, assume the minimal solution is fast enough.
- **Maintainability ties go to boring.** When two approaches are the same size, pick the one a junior engineer can understand.
- **`ponytail:` comments are the escape hatch.** Use them to document when and why the minimal path was diverged from, and under what conditions you would return to it.

### 6. Communication and Training

Adopting The Ladder is a cultural shift for agent teams accustomed to
writing comprehensive, anticipatory code. To make the philosophy effective:

**Communication:**
- The Director introduces The Ladder in the project kickoff (INIT→ACTIVE transition)
  so all teams hear it from the same source before implementation begins.
- Principles are referenced directly in `engineers.md`, `reviewers.md`, and
  `director.md` (Sections 2-4 above) — no agent can start work without
  encountering them in its workflow.
- The `ponytail:` comment convention provides a shared vocabulary for naming
  tradeoffs, making philosophical disagreements concrete and reviewable.

**Training:**
- Engineers onboard by reviewing two before/after examples from the Ponytail
  benchmark suite (linked in Documentation section), showing the same task
  implemented with and without The Ladder.
- Reviewers practice by auditing a sample implementation for Ladder compliance
  during onboarding — this builds muscle memory for the review checklist
  (Section 3).
- The first project after adoption serves as a "training wheels" period:
  reviewers are more explicit about Ladder feedback, and engineers are
  encouraged to err on the side of `ponytail:` comments.
- After three projects, the philosophy becomes procedural memory — agents
  internalize the rungs without needing to consciously recite them.

## Documentation

| Field | Value |
|-------|-------|
| **Tool/Philosophy** | Ponytail "Lazy Senior Dev" philosophy |
| **Source** | https://github.com/DietrichGebert/ponytail |
| **Version documented** | Latest (2026-06-19, ~40K stars) |
| **Key concepts** | "The Ladder" — 6-rung decision hierarchy. Safety boundary — never lazy about security/validation/error handling. `ponytail:` comments for intentional shortcuts. |
| **Known Gotchas** | • **Benchmarks are from Claude, not OpenCode** — the −54% LOC / −22% token / −20% cost figures were measured against Claude Sonnet 4.5, not OpenCode agents. OpenCode agents may respond differently. Treat benchmarks as *directional confidence*, not guaranteed results. Separate measurement can be done if needed. |
| | • **Ponytail is a prompt-only philosophy** — no runtime, parser, or enforcement layer. Compliance depends entirely on agent training and reviewer gate discipline. |
| | • **`ponytail:` comments require reviewer validation** — a comment claiming "this is the minimal correct approach" must be verifiable. Reviewers should spot-check the claim, not trust it reflexively. |

## Metrics

Adherence to The Ladder is qualitative but observable. Track these leading
indicators per project:

| Metric | What It Measures | How to Measure | Target |
|--------|-----------------|----------------|--------|
| **Lines of code (LOC)** | Total implementation size | `cloc --by-file src/` after implementation | Within 20% of Ponytail-benchmarked baseline for comparable scope |
| **Dependency count** | New external dependencies added | Count additions to `requirements.txt` / `package.json` / `Cargo.toml` (excluding test/dev) | 0 new dependencies per project on average; max 1 per project with `ponytail:` justification |
| **`ponytail:` comment density** | Intentional shortcuts documented | `grep -r "ponytail:" src/ \| wc -l` | ≥1 per file with non-trivial simplifications; 0 = likely missed shortcut |
| **Abstraction depth** | Layers of indirection not in the design doc | Review diff for classes, interfaces, or modules not present in the design doc's component diagram | 0 unrequested abstractions per PR (reviewer gate) |
| **Reviewer Ladder passes** | Code review compliance rate | Fraction of PRs where "The Ladder compliance" checkbox passes on first review | ≥80% by project 3; ≥95% steady-state |
| **Escalation rate** | How often second opinion is needed for ladder disputes | Count of ladder-related escalations from blocked reviews | <1 per 10 PRs steady-state |

**Measurement cadence:** Metrics are reviewed at the project REVIEW→CLOSED gate
as part of the post-mortem. Trends are evaluated quarterly by the Director.
No automated dashboard — current volume does not justify it.

**What we will NOT measure:**
- Token counts per implementation (too noisy, depends on prompt structure)
- Agent response times (a fast wrong answer is worse than a slow right one)
- Cost per project (fluctuates with model pricing; not a reliable Ladder signal)

## Consequences

### Positive

1. **Less code** — ~54% smaller implementations (per benchmark). Fewer lines = fewer bugs = less maintenance.
2. **Fewer dependencies** — stdlib-first reduces supply chain risk, build times, CVE surface.
3. **More maintainable** — "boring over clever" code is easier to understand and modify. `ponytail:` comments make tradeoffs explicit.
4. **Lower token consumption** — less code means fewer tokens across all lifecycle phases.
5. **Clearer reviews** — Reviewers have an objective standard: "does this need to exist?"

### Negative

1. **Risk of naive algorithms** — choosing a one-liner O(n²) over a more verbose O(1) solution.
   *Mitigation:* The Ladder says "pick edge-case-correct when two stdlib approaches are the same size." Reviewer checklist catches this.
2. **Over-zealous deletion** — "fewest files" taken to extremes.
   *Mitigation:* "Boring over clever" limits the golfing. Reviewers check readability.
3. **Initial slowdown as engineers internalize the ladder** — agents accustomed
   to writing comprehensive code may take 2-3 projects before the rungs become
   procedural memory. During this period, implementation may be slower as
   engineers consciously pause to ascend the ladder.
   *Mitigation:* The ladder is a mental checklist, not a slowdown. Most rungs
   are instant decisions ("does this need to exist?" takes <1 second). The
   training period described in Section 6 compresses the learning curve. After
   3 projects the effect reverses — the ladder prevents wasted work, so total
   time-to-correct-implementation *decreases*.

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Ladder used as excuse to stop thinking early | Medium | High | "Not lazy about" boundary + Reviewer checks |
| Ponytail philosophy changes incompatibly | Low | Low | We adopt as of 2026-06-19. Future changes need new ADR. |
| Benchmark results don't transfer from Claude to OpenCode | Medium | Medium | Directional confidence. Measure separately if needed. |

## ADR References

- **ADR-0001** (KodeHold Foundation) — principles table amended
- **ADR-0017** (Reviewers as Gatekeeper) — review/validation path for this ADR
- **ADR-0048** (Tool Documentation Review) — complementary safeguard

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.0 | 2026-06-19 | Initial ADR — Adopt Ponytail "Lazy Senior Dev" Philosophy |
