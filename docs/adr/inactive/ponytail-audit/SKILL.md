---
name: ponytail-audit
description: |
  Whole-repo over-engineering audit. Like ponytail-review (diff-level), but
  scans the entire codebase instead of a diff: a ranked list of what to delete,
  simplify, or replace with stdlib/native equivalents. Produces a one-shot
  report with net line and dependency reduction estimates.

  Scans for: deps stdlib/platform already ships, single-implementation interfaces,
  factories with one product, wrappers that only delegate, files exporting one
  thing, dead flags/config, hand-rolled stdlib, stale ponytail: comments.

  Output: one file-level finding per line, ranked by impact.
  End with: `net: -<N> lines, -<M> deps possible.`
  
  Does NOT apply fixes. Does NOT replace normal review.
---

# Ponytail Audit — Whole-Repo Over-Engineering Scan

## Philosophical Foundation

This skill operationalizes **The Ladder (ADR-0049)** at the project level. While
`ponytail-review` catches over-engineering in a *diff* (lines added/changed),
`ponytail-audit` finds over-engineering that is *already merged* — accumulated
complexity, dead weight, and unnecessary abstractions that crept in over time.

The same five tags apply, at the **file** level rather than the line level:

| Tag      | Ladder Rung        | What It Flags                             |
|----------|--------------------|-------------------------------------------|
| `delete:` | Rung 1 (YAGNI)     | File/module that doesn't need to exist    |
| `stdlib:` | Rung 2 (stdlib)    | Hand-rolled what stdlib already ships     |
| `native:` | Rung 3 (platform)  | Dependency for something the platform does|
| `yagni:`  | Rung 1 (YAGNI)     | Unused abstraction, speculative generality|
| `shrink:` | Rungs 5-6 (min)    | File contains logic that could be shorter |

---

## When to Load This Skill

Load via `skill` tool when:

- The user asks: "audit this project", "find bloat", "what can we delete",
  "ponytail-audit", "how over-engineered is this codebase"
- **Reviewers** are doing a pre-review audit before a major gate transition
  (e.g., REVIEW→CLOSED) to check for accumulated complexity
- **Scribes** are tracking project debt and want a baseline measurement
- **Directors** are evaluating a project's health during REOPEN impact analysis
- The project has not been audited before and you want a complexity baseline

**Do NOT load** for:

- Diff review during ACTIVE phase (use `ponytail-review` skill instead)
- Correctness, security, or performance review (use normal checklists)
- Design-phase discussions where no code exists

### How It Complements ponytail-review

| Dimension | ponytail-review | ponytail-audit |
|-----------|----------------|----------------|
| Scope | Diff (lines changed) | Whole tree (all files) |
| Granularity | Line-level (`L<line>:`) | File-level (`[path]`) |
| When used | During code review (ACTIVE) | Project health / pre-audit / debt tracking |
| How findings found | Manual scan of diff | Systematic grep/search commands |
| Score | `net: -<N> lines` | `net: -<N> lines, -<M> deps` |
| File-based storage | No | Optional (store via `add_memory(tags=["metrics"], scope="project")`) |
| Frequency | Every PR/review | Per-project, or on-demand |

---

## Protocol

### Step 0: Determine project language and structure

Identify the project's language(s) and build system. This determines which
hunt commands to run:

- **Python**: look for `requirements.txt`, `pyproject.toml`, `setup.py`, `Pipfile`
- **Node/JS/TS**: look for `package.json`, `tsconfig.json`
- **Go**: look for `go.mod`
- **Rust**: look for `Cargo.toml`
- **Ruby**: look for `Gemfile`
- **Multi-language**: run relevant hunts for each language

Set the source root. Default: project root. If `src/` or `lib/` exists, use that.

### Step 1: Exclude generated directories

These are **always excluded** from the audit scan:

```
.git, node_modules, __pycache__, .venv, .tox, dist, build,
.eggs, *.egg-info, vendor, third_party, .next, .nuxt,
public/build, coverage, .mypy_cache, .pytest_cache, .ruff_cache,
target, .sass-cache, .gradle, buck-out, .dart_tool
```

If a `.ponytailignore` file exists at the project root, read it and add its
patterns (one per line, glob format) to the exclusion list.

If the project has no non-excluded source files, stop: `No source files to audit.`

### Step 2: Run hunt commands

Execute the relevant grep-based hunts. Each hunt targets a specific over-engineering
pattern. Run all that apply to the project's language(s).

Findings are **provisional** — review each before reporting.

#### Hunt 1 — Single-implementation interfaces

Look for interfaces/abstract classes that have exactly one concrete implementation.

**Python:**
```
# Find ABC/Protocol subclasses
grep -rn "^class.*ABC" src/ --include="*.py"
grep -rn "^class.*Protocol" src/ --include="*.py"
grep -rn "@abstractmethod" src/ --include="*.py"
# Then check how many concrete subclasses exist for each
```

**TypeScript/JavaScript:**
```
# Find interfaces/abstract classes
grep -rn "^interface " src/ --include="*.ts"
grep -rn "^abstract class" src/ --include="*.ts"
# Then check implementors
```

**Go:**
```
# Find interfaces with one implementation
grep -rn "^type.*interface" . --include="*.go"
```

**Tag:** `yagni:` — if exactly one implementation and it was not explicitly
requested in the design doc.

#### Hunt 2 — Factories with one product

Find factory functions or classes that return/create only one type of object.

```
# Python: functions named create_*/build_*/make_*/factory
grep -rn "def create_\|def build_\|def make_\|def factory\|class.*Factory" src/ --include="*.py"
# TS/JS: similar
grep -rn "function create\|function build\|class.*Factory" src/ --include="*.ts"
```

**Tag:** `yagni:` — if the factory always produces the same type and has
exactly one call site. Inline it.

#### Hunt 3 — Wrappers that only delegate

Find files/functions whose sole purpose is to call something else unchanged.

```
# Python: one-liner delegation
grep -rn "return self\._\|return self\.__\|return super()" src/ --include="*.py"
# Decorators/wrappers that just call through
grep -rn "def wrapper.*args.*kwargs" src/ --include="*.py"
```

**Tag:** `delete:` — if the wrapper adds zero logic. If it adds logging/metrics
that were requested in the design doc, skip it.

#### Hunt 4 — Files exporting one thing

Find files that define/export a single public symbol.

```
# Python: count public symbols per file
grep -rn "^def \|^class " src/ --include="*.py" | ...
```

**Tag:** `shrink:` — merge into the file that uses it, or `delete:` if the
single thing is also unused.

#### Hunt 5 — Dead flags and config

Find configuration flags, feature flags, or environment variables that are
always set to one value or never read.

```
# Python: flags that are always True/False
grep -rn "ENABLE_\|FEATURE_\|USE_\|FLAG_" src/ --include="*.py"
# Config keys read from env but never varied
grep -rn "os\.getenv\|os\.environ\.get" src/ --include="*.py"
```

**Tag:** `delete:` — if the flag is always the same value and has been for >2
releases. **Tag:** `shrink:` — if the config could be replaced by a constant.

#### Hunt 6 — Hand-rolled stdlib

Find common stdlib replacements that were reimplemented.

**Python targets:**
```
# Custom caching — stdlib: functools.lru_cache / functools.cache
grep -rn "def.*cache\|class.*Cache" src/ --include="*.py"
# Custom dataclass — stdlib: dataclasses.dataclass
grep -rn "__init__\|__repr__\|__eq__" src/ --include="*.py"
# Custom path handling — stdlib: pathlib.Path
grep -rn "os\.path\.\|os\.walk\|shutil\." src/ --include="*.py"
# Custom date parsing — stdlib: datetime / zoneinfo
grep -rn "dateutil\|pytz\|dateparser" src/ --include="*.py"
# Custom logging — stdlib: logging
grep -rn "print(" src/ --include="*.py"
# Custom CLI argument parsing — stdlib: argparse
grep -rn "sys\.argv" src/ --include="*.py"
```

**JS/TS targets:**
```
# Custom URL parsing — stdlib: URL / URLSearchParams
grep -rn "new URL\|\\.split.*?" src/ --include="*.ts"
# Custom array operations — stdlib: Array methods
grep -rn "\.forEach\|\.map\|\.filter" src/ --include="*.ts"
```

**Tag:** `stdlib:` — name the stdlib function that replaces the hand-rolled code.

#### Hunt 7 — Deps that stdlib already covers

Scan the dependency manifest(s) for packages that have stdlib alternatives.

**Python:**
```
# Check against this list:
#   requests → urllib.request (stdlib)
#   dateutil → zoneinfo + datetime (stdlib, Python 3.9+)
#   pytz → zoneinfo (stdlib, Python 3.9+)
#   dataclasses → dataclasses (stdlib, Python 3.7+)
#   typing_extensions → typing (stdlib — check Python version first)
#   pydantic → dataclasses (if not using Pydantic features)
#   attrs → dataclasses
#   colorama → ANSI escape codes (stdlib on most terminals)
#   mock → unittest.mock (stdlib, Python 3.3+)
```

**JS/TS:**
```
# Check against this list:
#   lodash → Array.map, Object.entries, etc (stdlib)
#   moment → Intl.DateTimeFormat (stdlib)
#   uuid → crypto.randomUUID() (stdlib, Node 19+)
#   axios → fetch (stdlib, Node 18+)
#   qs → URLSearchParams (stdlib)
#   date-fns → Intl.DateTimeFormat (stdlib, for basic formatting)
```

**Tag:** `native:` (if replaced by platform feature) or `stdlib:` (if replaced
by stdlib). For each, estimate the dependency reduction.

#### Hunt 8 — Stale ponytail: comments

Find ponytail: comments and verify the shortcut they document is still valid.

```
grep -rn "ponytail:" src/ --include="*.py" --include="*.ts" --include="*.js"
```

**Tag:** `delete:` — if the ponytail: ceiling has been reached.
**Tag:** `yagni:` — if the ponytail: upgrade path is now trivially available.

#### Hunt 9 — Unnecessary dependencies

Scan import statements for dependencies that are barely used.

```
# Python: find imports used once or in trivial ways
grep -rn "import " src/ --include="*.py" | grep -v "^#"
```

Focus on large dependencies (frameworks, ORMs, ML libraries) imported for
single small operations.

**Tag:** `native:` or `stdlib:` — depending on what replaces it.

### Step 3: Rank findings by impact

Sort findings by estimated lines saved, largest first. Use this rubric:

| Impact | Lines Saved | Description |
|--------|-------------|-------------|
| **Critical** | 100+ lines | Entire module or multi-file abstraction |
| **High** | 30–99 lines | Class or substantial function |
| **Medium** | 10–29 lines | Function or sizeable block |
| **Low** | 1–9 lines | Single expression or small refactor |

Within same-impact tiers, order by dependency reduction first.

### Step 4: Report findings in format

One line per finding, ranked highest impact first with file path:

```
<tag> <what>. <replacement>. [<path>]
```

### Step 5: End with net score

```
net: -<N> lines, -<M> deps possible.
```

If nothing to cut:

```
Lean already. Ship.
```

---

## Tags Reference

| Tag | When to Use | Replacement | Example Finding |
|-----|------------|-------------|-----------------|
| `delete:` | Dead code, unused flexibility, speculative feature, stale ponytail: shortcut | Nothing — remove it | `delete: unused retry wrapper. Nothing. [src/client/retry.py]` |
| `stdlib:` | Hand-rolled implementation of something in stdlib | Name the stdlib function | `stdlib: custom path join utility. os.path.join / pathlib.Path. [src/utils/paths.py]` |
| `native:` | Dependency or code doing what the platform does natively | Name the platform feature | `native: moment.js for one format. Intl.DateTimeFormat. [src/utils/format.ts]` |
| `yagni:` | Unused abstraction (interface with one impl, factory with one product) | Inline it | `yagni: IDataProvider with one impl. Inline. [src/data/provider.py]` |
| `shrink:` | Same logic, more lines than necessary | Show the shorter form | `shrink: 15-line manual reduce loop. sum(), 1 line. [src/calc/total.py]` |

---

## Scoring

Every ponytail-audit ends with a two-part net score:

```
net: -<N> lines, -<M> deps possible.
```

### Estimating lines

For each finding, estimate conservatively:

- **delete:** full file line count + imports (if the whole file goes)
- **stdlib:** lines replaced by the stdlib function + imports removed
- **native:** lines replaced + dependency removal overhead
- **yagni:** lines of the abstraction + its import and the inline cost
- **shrink:** the conservative difference between old and new line count

If two findings overlap, count the net saving only once. Round down.

### Estimating deps

Count each *unique* external dependency that could be fully removed:

- A dependency used only in the deleted code: **−1 dep**
- A dependency that could be replaced by stdlib: **−1 dep**
- A dependency whose only remaining use is test/dev: **−1 dep** (move to dev deps)
- A dependency still used elsewhere: **−0 dep**

Do not count transitive dependencies.

---

## Boundaries

### IN scope (flag these)

- Unnecessary abstractions not in the design doc
- Hand-rolled implementations of standard library features
- External dependencies replaceable by stdlib or platform features
- Speculative generality (config for fixed values, feature flags always on/off)
- Boilerplate that could be expressed in fewer lines
- Dead code, commented-out code, scaffolding, unused exports
- Files/classes that exist only to delegate to something else
- Wrapper layers that add zero value
- Stale `ponytail:` comments
- Dependencies imported but barely used

### OUT of scope (route to normal review)

- Correctness bugs, security vulnerabilities, performance bottlenecks
- Test coverage — a single smoke test is the ponytail minimum
- Code style, formatting, naming conventions
- Missing error handling (that is the "Not lazy about" check)
- Code explicitly requested in the design doc

### What this skill does NOT do

- Does **NOT** apply fixes — only lists findings
- Does **NOT** replace normal review — it is a complementary audit pass
- Does **NOT** second-guess the design doc
- Does **NOT** run automatically — must be loaded via the `skill` tool
- Does **NOT** recurse into excluded directories

---

## Persistent Storage (Optional)

The calling agent may store the audit findings via `add_memory(tags=["metrics"], scope="project")` for debt tracking.
This is **optional** — the caller decides whether to persist.

### Store as memory

```
add_memory(content="# Ponytail Audit: <project>\n\nnet: -<N> lines, -<M> deps possible.\n\nFindings:\nyagni: AbstractRepository... [src/repo/interfaces.py]\nstdlib: custom LRU cache... [src/cache/local.py]\n...", tags=["metrics", "ponytail-audit"], scope="project")
```

### Recall previous audit

```
graphify query "ponytail-audit <project>"
```

### Persist findings

Store the report via `add_memory(tags=["metrics"], scope="project")` so it can be recalled later.

---

## Important Notes

- **No false findings.** If a hunt returns a false positive, skip it. The audit
  is a *judgment call*, not a mechanistic report.
- **`ponytail:` comments are your ally.** A comment with a clear ceiling and
  upgrade path is **not** a finding. Only flag if the ceiling has been reached.
- **Correctness always wins.** If the minimal alternative breaks correctness,
  flag in normal review, not here.
- **Boring over clever.** Slightly more verbose but readable code is not a
  `shrink:` finding — it's The Ladder's tie-breaker in action.
- **One-shot only.** Generates a report and stops. Applies nothing unless the
  caller chooses to persist findings.
- **Rank by impact, not count.** A single 200-line file removal is worth more
  than five 3-line savings.

---

Deliverables: Confirm that the full SKILL.md was created successfully at .opencode/skills/ponytail-audit/SKILL.md.
