---
name: ponytail-review
description: |
  Over-engineering analysis for code review. Companion to The Ladder (ADR-0049).
  Tags diffs with delete:, stdlib:, native:, yagni:, shrink: tags to flag
  unnecessary complexity. One line per finding: location, what to cut, what
  replaces it.
  NOT a replacement for normal review — correctness, security, and performance
  are out of scope.
  Load when you need to audit a diff for over-engineering, or when the
  "The Ladder compliance (ADR-0049)" checklist item finds potential issues.
---

# Ponytail Review — Over-Engineering Analysis

## Philosophical Foundation

This skill operationalizes **The Ladder (ADR-0049)** for code review. The Ladder
asks engineers to ascend 6 rungs before writing code:

1. Does this need to exist? (YAGNI)
2. Does the standard library already do this?
3. Does a native platform feature cover it?
4. Does an already-installed dependency solve it?
5. Can this be one line?
6. Only then — write the minimum code that works.

This skill asks the corresponding question in reverse: **did the code stop at
the highest rung that held, or did it fall through to unnecessary complexity?**

The five tags map directly to Ladder rungs:

| Tag      | Ladder Rung        | What It Flags                              |
|----------|--------------------|--------------------------------------------|
| `delete:` | Rung 1 (YAGNI)     | Code that doesn't need to exist at all     |
| `stdlib:` | Rung 2 (stdlib)    | Hand-rolled what stdlib already ships      |
| `native:` | Rung 3 (platform)  | Dependency for something the platform does |
| `yagni:`  | Rung 1 (YAGNI)     | Speculative abstraction, unrequested       |
| `shrink:` | Rungs 5-6 (min)    | Same logic, fewer lines possible           |

---

## When to Load This Skill

Load via `skill` tool when:

- You are in a code review and the **"The Ladder compliance (ADR-0049)"** 
  checklist item in `reviewers.md` identifies potential over-engineering
- You see code that feels "heavy" — unnecessary abstractions, unneeded 
  dependencies, boilerplate
- The user explicitly asks: "review for over-engineering", "what can we 
  delete", "is this over-engineered", "simplify review"
- The diff is unusually large and you suspect complexity creep

**Do NOT load** for:
- First-pass correctness/security/performance review (use normal checklist)
- Greenfield design discussions where no code exists yet

---

## Protocol

### Step 1: Complete normal review first

Run through the full Reviewers checklist in `reviewers.md` **before** loading
this skill. The ponytail-review is a **second pass** that only hunts over-
engineering. Correctness, security, and performance must be verified first.

### Step 2: Scan diff with Ladder rungs in mind

For each changed file, ask in order:

1. **Does any of this code not need to exist at all?** → `delete:` tag
2. **Is the engineer reinventing stdlib?** → `stdlib:` tag  
3. **Could a platform feature replace a dependency?** → `native:` tag
4. **Is there a speculative abstraction with one caller?** → `yagni:` tag
5. **Can the same logic be written in fewer lines?** → `shrink:` tag

### Step 3: Report findings in format

One line per finding. Use the diff's line numbers:

```
L<line>: <tag> <what>. <replacement>.
```

For multi-file diffs, prefix the file:

```
<file>:L<line>: <tag> <what>. <replacement>.
```

### Step 4: End with net score

```
net: -<N> lines possible.
```

If nothing to cut: `Lean already. Ship.`

---

## Tags Reference

| Tag | When to Use | Replacement | Example |
|-----|------------|-------------|---------|
| `delete:` | Dead code, unused flexibility, speculative feature, commented-out code, scaffolding "for later" | Nothing — remove it | `L52-71: delete: retry wrapper around an idempotent local call. Nothing replaces it.` |
| `stdlib:` | Hand-rolled implementation of something in the standard library | Name the stdlib function | `L12-38: stdlib: 27-line validator class. "@" in email, 1 line, real validation is the confirmation mail.` |
| `native:` | Dependency or code doing what the platform already does natively | Name the platform feature | `L4: native: moment.js imported for one format call. Intl.DateTimeFormat, 0 deps.` |
| `yagni:` | Abstraction with one implementation, config nobody sets, interface with one implementor, factory with one product | Inline it | `repo.py:L88: yagni: AbstractRepository with one implementation. Inline it until a second one exists.` |
| `shrink:` | Same logic, more lines than necessary | Show the shorter form | `L30-44: shrink: manual loop builds dict. dict(zip(keys, values)), 1 line.` |

---

## Examples

### ❌ Bad review feedback (vague, not actionable)

> "This EmailValidator class might be more complex than necessary, have you
> considered whether all these validation rules are needed at this stage?"

### ✅ Good ponytail-review feedback (specific, tagged, scorable)

> `L12-38: stdlib: 27-line validator class. "@" in email, 1 line, real validation is the confirmation mail.`
>
> `L4: native: moment.js imported for one format call. Intl.DateTimeFormat, 0 deps.`
>
> `repo.py:L88: yagni: AbstractRepository with one implementation. Inline it until a second one exists.`
>
> `L52-71: delete: retry wrapper around an idempotent local call. Nothing replaces it.`
>
> `L30-44: shrink: manual loop builds dict. dict(zip(keys, values)), 1 line.`
>
> `net: -112 lines possible.`

---

## Scoring

End every ponytail-review with the only metric that matters:

```
net: -<N> lines possible.
```

This is the net reduction if all findings were applied. It is an estimate —
some findings may overlap or interact. Round conservatively.

If there is nothing to cut, say `Lean already. Ship.` and stop. Do not force
findings.

---

## Boundaries

### IN scope (flag these)

- Unnecessary abstractions (interfaces, factories, base classes not in design doc)
- Reinvented standard library functions
- Unneeded external dependencies
- Speculative generality (config for a value that never changes, feature flags for unimplemented features)
- Boilerplate that could be one line
- Code that doesn't need to exist at all
- `ponytail:` comments that are missing or stale (the engineer claimed a shortcut but didn't document ceiling or upgrade path)

### OUT of scope (route to normal review)

- Correctness bugs
- Security vulnerabilities
- Performance bottlenecks (unless trivially obvious, e.g., O(n²) in a hot loop)
- Test coverage — a single smoke test or `assert`-based self-check is the ponytail minimum, never flag it for deletion
- Code style, formatting, naming conventions
- Missing error handling (that is the "Not lazy about" check — see reviewers.md)
- Missing documentation

### What this skill does NOT do

- Does NOT apply fixes — only lists findings
- Does NOT replace the normal review checklist — it is a complementary pass
- Does NOT second-guess the design doc — if an abstraction was explicitly
  requested in the design doc, it is not yagni

---

## Integration with Reviewers Checklist

This skill is referenced by the existing checklist item in `reviewers.md`:

```
- [ ] **The Ladder compliance (ADR-0049)** — verify implementation ascends the ladder:
  - Could this have been done with stdlib? If yes, why was a dependency introduced?
  - Are there abstractions not explicitly requested in the design doc?
  - Are there `ponytail:` comments documenting intentional shortcuts with ceilings and upgrade paths?
  - Does every new dependency have clear justification vs. stdlib alternatives?
  - Edge-case-correctness verified — if stdlib offered two same-sized approaches, was the more correct one chosen?
```

When this checklist item identifies **concrete findings**, load this skill to
perform the systematic tagging and scoring. The checklist is the **what**;
this skill is the **how**.

Conversely, if this skill finds nothing to cut (`Lean already. Ship.`), the
checklist item passes automatically.

---

## Important Notes

- **Do not force findings.** If the diff is already lean, say so and move on.
- **`ponytail:` comments are your ally.** ADR-0049 requires engineers to mark
  intentional shortcuts. If a `ponytail:` comment exists with a clear ceiling
  and upgrade path, that is **not** a finding — it's compliance.
- **Correctness always wins.** If a minimal solution is wrong, flag it in the
  normal review, not here. This skill assumes correctness is already verified.
- **Boring over clever.** When an engineer chose a slightly more verbose
  solution that is easier to read, that is not a `shrink:` finding — it is
  The Ladder's tie-breaking rule in action.
