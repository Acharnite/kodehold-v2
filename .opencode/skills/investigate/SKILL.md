---
name: investigate
description: |
  Systematic debugging with root cause investigation. Four phases:
  investigate → analyze → hypothesize → implement.
  Iron Law: no fixes without root cause.
  Use when asked to debug, fix a bug, investigate an error, or perform
  root cause analysis. Proactively invoke when the user reports errors,
  stack traces, unexpected behavior, or "it was working yesterday".
---

# Investigate — Systematic Debugging

## Iron Law

**NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.**

Fixing symptoms creates whack-a-mole debugging. Every fix that doesn't address
root cause makes the next bug harder to find. Find the root cause, then fix it.

---

## Phase 1: Root Cause Investigation

Gather context before forming any hypothesis.

0. **Check prior memory — has this bug been seen before?**
   ```
   search_memories(query="<error-type> <component> bugs investigations", scope="project")
   ```
   If results include a prior investigation, READ that report first. There is no
   point debugging a solved problem. If the prior fix is still in place, this is
   a recurrence — treat it as an architectural smell, not a fresh bug.

1. **Collect symptoms:** Read error messages, stack traces, and reproduction
   steps. If the user hasn't provided enough context, ask ONE question at a time.

2. **Trace the code:** Trace the code path from the symptom back to potential
   causes. Use Grep to find references, Read to understand logic.

3. **Check recent changes:**
   ```bash
   git log --oneline -20 -- <affected-files>
   ```
   Was this working before? What changed? A regression means the root cause
   is in the diff.

4. **Reproduce:** Can you trigger the bug deterministically? If not, gather
   more evidence before proceeding.

5. **Check for prior investigations in opencode-mem:** Search for prior bug investigation
   reports via `search_memories(query="<error-type> <component> bugs", scope="project")`,
   or via `graphify query`. Recurring bugs in the same module are an
   architectural smell, not a coincidence.

**Output:** A specific, testable hypothesis about what is wrong and why.

---

## Phase 2: Pattern Analysis

Check if this bug matches a known pattern:

| Pattern | Signature | Where to look |
|---------|-----------|---------------|
| Race condition | Intermittent, timing-dependent | Concurrent access to shared state |
| Null/nil propagation | TypeError, NoMethodError | Missing guards on optional values |
| State corruption | Inconsistent data, partial updates | Transactions, callbacks, hooks |
| Integration failure | Timeout, unexpected response | External API calls, service boundaries |
| Configuration drift | Works locally, fails elsewhere | Env vars, feature flags, DB state |
| Stale cache | Shows old data, clears on restart | Redis, CDN, browser cache, Turbo |
| Off-by-one | Boundary errors, index errors | Loop conditions, array access |
| Resource leak | Gradual degradation, OOM | File handles, connections, memory |

If the bug doesn't match a known pattern, search for the error type:
- Strip hostnames, IPs, file paths, and sensitive data from the error
- Search: `{component} {generic error type} {language/framework}`

---

## Phase 3: Hypothesis Testing

Before writing ANY fix, verify your hypothesis.

1. **Confirm:** Add a temporary assertion, log, or debug output at the suspected
   root cause. Run the reproduction. Does the evidence match?

2. **If wrong:** Return to Phase 1. Gather more evidence. Do not guess.

3. **3-strike rule:** If 3 hypotheses fail, STOP and escalate:
   > 3 hypotheses tested, none match. This may be an architectural issue
   > rather than a simple bug.
   >
   > Options:
   > - Continue with a new hypothesis
   > - Escalate to human review
   > - Instrument the area and catch it next time

### Red flags
- "Quick fix for now" — there is no "for now." Fix it right or escalate.
- Proposing a fix before tracing data flow — you are guessing.
- Each fix reveals a new problem — wrong layer, not wrong code.

---

## Phase 4: Implementation

Once root cause is confirmed:

1. **Fix the root cause, not the symptom.** The smallest change that eliminates
   the actual problem.

2. **Minimal diff:** Fewest files touched, fewest lines changed. Resist the
   urge to refactor adjacent code.

3. **Write a regression test** that:
   - **Fails** without the fix (proves the test is meaningful)
   - **Passes** with the fix (proves the fix works)

4. **Run the full test suite.** No regressions allowed.

5. **If the fix touches >5 files:** Flag the blast radius and ask whether to
   proceed, split, or rethink.

---

## Phase 5: Verification & Report

**Fresh verification:** Reproduce the original bug scenario and confirm it is
fixed. This is not optional.

Output a structured debug report:

```
═══ DEBUG REPORT ═══════════════════════
Symptom:         What the user observed
Root cause:      What was actually wrong
Fix:             Changed file:line references
Evidence:        Test output showing fix works
Regression test: file:line of the new test
Related:         Prior bugs, architectural notes
Status:          DONE | DONE_WITH_CONCERNS | BLOCKED
═════════════════════════════════════════
```

### Store findings via opencode-mem

Save the investigation report so future sessions can find it:

```
add_memory(content="[Full debug report content]", tags=["bug", "<component>"], scope="project")
```

Access via: `search_memories(query="investigation <component>", scope="project")` or
`graphify query "investigation <component>"`

---

## Important Rules

- **3+ failed fix attempts → STOP.** Wrong architecture, not failed hypothesis.
- **Never apply a fix you cannot verify.** If you can't reproduce and confirm,
  do not ship it.
- **Never say "this should fix it."** Verify and prove it. Run the tests.
- **If fix touches >5 files → ask** about blast radius before proceeding.
- **Document everything.** Store findings via `add_memory(tags=["bug"], scope="project")`. The next person debugging this code will thank you.
