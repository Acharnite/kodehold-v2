# ADR-0047: Universal Test Execution Standard

## Status

Accepted

## Context

KodeHold agents currently lack a shared standard for executing tests, leading to
wasted tokens, inconsistent behaviour, and silent failures:

1. **Engineers** have zero test execution in their workflow (engineers.md).
2. **FLS** has vague instructions at line 99 (`Verify: run relevant tests using
   KodeHold root `.venv/bin/pytest``) — no flag guidance, no mode distinction.
3. **Testers** has detailed but incomplete pytest instructions (testers.md lines
   68-73), including forbidding `rtk pytest`, but no quick/full/smoke distinction.
4. **Reviewers** never run tests independently during gate validation.

In practice, agents cobble together ad-hoc flags (`-x`, `-v`, `--tb=short`,
`-q`, `--no-header`) or try invalid commands (`rtk pytest`), which wastes
tokens and produces inconsistent output.

This ADR defines a universal test execution standard that all six KodeHold
subagent teams will reference, covering:

- Three formally defined test modes (quick, full, smoke) with precise flag sets
- Virtual environment discovery chain
- Working directory and PYTHONPATH handling
- Symlink resolution for adopted projects
- Non-Python framework detection
- Agent-file integration changes

## Decision

### 1. Test Mode Definitions

Every test invocation MUST use one of three formally defined modes. The mode
determines the flag set, not the scope of tests — scope is always explicit
(path or directory argument).

| Mode | Purpose | Flags |
|------|---------|-------|
| **quick** | Targeted verification of a single change | `-x --no-header -q --tb=line` |
| **full**  | Complete suite validation before merge | `-v --tb=short` |
| **smoke** | Sanity check that the suite boots | `-x --no-header -q` |

**Quick mode** — `-x` (fail-fast, stop at first failure), `--no-header`
(suppress pytest header), `-q` (compact output), `--tb=line` (single-line
traceback — compact enough for rapid dev feedback). Use when verifying a
specific change or debugging a single test.

**Full mode** — `-v` (verbose test names), `--tb=short` (moderately detailed
traceback without the full `long` verbosity). Use before committing or when
running the complete suite.

**Smoke mode** — Same flags as quick but without `--tb=line`. Use for a quick
sanity check that the test suite can load and start running (e.g., checking for
import errors, configuration issues).

#### Flag Reference

| Flag | Effect | Included In |
|------|--------|-------------|
| `-x` | Stop on first failure | quick, smoke |
| `--no-header` | Suppress pytest header banner | quick, smoke |
| `-q` | Decrease verbosity (compact output) | quick, smoke |
| `--tb=line` | Single-line traceback per frame | quick |
| `-v` | Increase verbosity (test names) | full |
| `--tb=short` | Moderate traceback (no local vars) | full |

#### General Rules

- Always pass an explicit **path or directory** — never run `pytest` without
  arguments (which defaults to `tests/` but is ambiguous in certain setups).
- Never use `rtk pytest` — `rtk` does not support `pytest` as a subcommand.
  Use `.venv/bin/pytest <mode-flags> <path>` directly.
- The `<path>` may resolve through a symlink (see Section 4).

### 2. Virtual Environment Discovery

When invoking pytest, resolve the `pytest` binary via this ordered chain.
Stop at the first match.

| Priority | Check | Command |
|----------|-------|---------|
| 1 | Project `.venv` (workspace) | `workspaces/<name>/.venv/bin/pytest` |
| 2 | KodeHold root `.venv` | `<kodehold-root>/.venv/bin/pytest` |
| 3 | System `pytest` | `pytest` (from `$PATH`) |

**Rationale:** Priority 1 allows workspace projects to pin their own pytest
version. Priority 2 is the always-available fallback (KodeHold root `.venv` has
pytest pre-installed). Priority 3 is a last resort for environments where
neither project nor KodeHold root has a `.venv`.

> **Note:** For adopted projects (ADR-0012), the workspace path is a symlink.
> See Section 4 for symlink resolution rules. Priority 1's path resolution
> respects the realpath rules defined there.

### 3. Working Directory & PYTHONPATH

**Working directory:** Always `cd` to the **project root** (resolved via
`realpath` — see Section 4) before executing any test command. This ensures
relative imports and config discovery work correctly.

**PYTHONPATH:** Set automatically based on project layout:

| Layout | PYTHONPATH |
|--------|------------|
| `src/` exists | `<project-root>/src` |
| `lib/` exists | `<project-root>/lib` |
| neither | `<project-root>` (i.e., `.`) |

> The project root is the `realpath`-resolved canonical path (see Section 4),
> not the symlink path. This avoids import resolution issues.

### 4. Symlink Handling for Adopted Projects

When the workspace path is a symlink (adopted projects per ADR-0012), follow
these rules:

1. **Resolve paths** — use `realpath` to resolve the workspace path to its
   canonical absolute path before constructing PYTHONPATH, working directory,
   and test discovery paths:
   ```bash
   PROJECT_ROOT="$(realpath "workspaces/<name>")"
   ```
2. **pytest confdir** — if pytest complains about config discovery, pass
   `--rootdir "$(realpath "workspaces/<name>")"` explicitly.
3. **Test discovery paths** — when passing a test directory, resolve it through
   `realpath` to avoid "module not found" errors from duplicate collection.
   ```bash
   .venv/bin/pytest "$(realpath "workspaces/<name>/tests")"
   ```
4. **Marker files** — `.testers_done` and similar marker files are created at
   the workspace path (`workspaces/<name>/`), which resolves through the
   symlink to the real project root.

Where `<name>` is the workspace project slug from `workspaces/<name>/` per
ADR-0012 and ADR-0036.

### 5. Non-Python Framework Detection

For projects that do not use pytest, agents run a lightweight detection script
before falling back to the mode-based standard. The detection script checks
for framework-specific indicator files:

| Priority | Indicator | Command | Framework |
|----------|-----------|---------|-----------|
| 1 | `Cargo.toml` at project root | `cargo test` | Rust/Cargo |
| 2 | `package.json` with `"scripts": {"test": "..."}` | `npm test` | Node.js/npm |
| 3 | `package.json` (has test script via yarn) | `yarn test` | Node.js/yarn |
| 4 | `Gemfile` containing `rspec` | `bundle exec rspec` | Ruby/RSpec |
| 5 | `mix.exs` | `mix test` | Elixir |
| 6 | `build.gradle` or `build.gradle.kts` | `gradle test` | Gradle |
| 7 | `pom.xml` | `mvn test` | Maven |
| 8 | `build.sbt` | `sbt test` | SBT/Scala |
| 9 | `deno.json` or `deno.jsonc` | `deno test` | Deno |
| 10 | `go.mod` | `go test ./...` | Go |
| 11 | `Makefile` (containing a `test:` target) | `make test` | Make |

**Detection order notes:**

- Frameworks are checked in priority order (1–11). The first match wins.
- **Makefile detection is a generic fallback** and is deliberately placed last
  because many projects use Makefiles as wrappers around framework-specific
  test tools. A Python project with a `Makefile` that wraps `pytest` would
  otherwise be misdetected as `make` rather than `pytest`.
- **Makefile → pytest passthrough:** If `make test` is the matched target AND
  the `Makefile` contains a reference to `pytest` (detected via
  `grep -q pytest Makefile`), resolve to the `pytest` framework instead of
  `make`. The agent should then use the pytest mode system from Section 1
  rather than running `make test`.
- This script is embedded as a skill or inline agent instruction (see
  Sections 6–7). It is **not** a separate file — it lives in agent definitions
  or the skills system.

For Python projects specifically, detection should also check:
- `pyproject.toml` for `[tool.pytest.ini_options]`
- `setup.cfg` for `[tool:pytest]`
- `pytest.ini` at project root

Any of these indicate a pytest project regardless of the presence of `tests/`
with `.py` files.

#### Detection Script (pseudocode)

```
for each (indicator, command) in ordered_list:
    if indicator matches:
        if indicator == "Makefile" and grep("pytest", "Makefile"):
            resolve_to_pytest()
        else:
            run(command)
            break
```

### 6. Agent Integration

Each team agent file must be updated to reference this standard:

#### Testers (testers.md)

**Replace lines 68-73** (the current pytest invocation block) with:

```
4. Run existing test suite to verify no regressions
    - Follow ADR-0047 for test execution mode selection:
      - **quick** mode (`-x --no-header -q --tb=line`) for targeted change verification
      - **full** mode (`-v --tb=short`) for complete suite validation
      - **smoke** mode (`-x --no-header -q`) for sanity checks
    - Resolve the venv using the ADR-0047 discovery chain:
      1. `workspaces/<name>/.venv/bin/pytest`
      2. `<kodehold-root>/.venv/bin/pytest`
      3. System `pytest`
    - Set PYTHONPATH per ADR-0047: `src/` → `src`, `lib/` → `lib`, else `.`
    - For adopted projects, resolve paths via `realpath` (ADR-0047 Section 4)
    - Never use `rtk pytest` — rtk does not support pytest as a subcommand
```

#### Engineers (engineers.md)

Add after the existing step 5 ("Run RTK-compact commands to minimize token
consumption"):

```
5b. **Run quick mode tests** on the specific test file(s) covering your change
    before submitting for review. Follow ADR-0047 for invocation:
    `.venv/bin/pytest -x --no-header -q --tb=line <test-path>`
    This catches regressions early and reduces Reviewers' iteration cycles.
```

#### FLS (fls.md)

**Replace line 99** (vague test instruction) with:

```
f. Verify using ADR-0047: run relevant tests in **quick** mode
   (`.venv/bin/pytest -x --no-header -q --tb=line <test-path>`).
   Follow the venv discovery chain in ADR-0047 Section 2.
```

#### Reviewers (reviewers.md)

Add to the Review Checklist (after the existing test item):

```
- [ ] Tests pass in **full** mode per ADR-0047 — run the complete suite with
      `.venv/bin/pytest -v --tb=short <test-dir>` before approving gates
```

### 7. Skills & Enforcement

This standard is enforced through two mechanisms:

#### 7.1 Tool Permission Denial

**Deny `rtk` for subagents if possible, or add a pre-execution check.** The
root cause of agents trying `rtk pytest` is that `rtk` is in the agent's
toolset but doesn't support `pytest` as a subcommand. The strongest enforcement
is to remove the `pytest` misdirection at the permission level:

1. If the agent platform supports denying specific RTK subcommands, block
   `rtk pytest`.
2. Otherwise, add a pre-execution check in each agent's workflow step that
   validates the command before running it: "Before running any test command,
   verify you are invoking `.venv/bin/pytest` directly, not via `rtk`."

#### 7.2 Skills System Integration

If a reusable skill is warranted (e.g., for the framework detection script in
Section 5), create a `universal-test-execution` skill at
`.opencode/skills/universal-test-execution/SKILL.md` containing:

- The framework detection script (Section 5)
- The venv discovery chain (Section 2)
- The three mode definitions (Section 1)

This skill would be listed in the Skills System table (design doc Section 7.4)
and loaded on-demand by agents that need to run tests.

### 8. Alternatives Considered

#### Alternative 1 — Keep status quo (no standard)

Let each agent continue to improvise test commands. **Rejected** because real
agents waste tokens on incorrect invocations (`rtk pytest`) and inconsistent
flag sets. The investigation found 4 different flag combinations in use.

#### Alternative 2 — Single `pytest` wrapper script

Create a single `scripts/test` script that handles venv resolution, flag
selection, and framework detection internally. Agents just run
`bash scripts/test <mode> <path>`. **Rejected** because it adds a script
maintenance burden and hides the invocation details from agents, making
debugging harder. Agents should understand what flags they are passing.

#### Alternative 3 — Auto-detect mode from git diff

Use `git diff --name-only` to automatically determine whether to run quick,
full, or smoke mode based on the scope of changes. **Rejected** because it
introduces non-deterministic behaviour — the same test run could produce
different modes depending on git state. Agents should explicitly choose
the mode.

#### Alternative 4 — All tests in smoke mode

Default all test invocations to smoke mode (`-x --no-header -q`). **Rejected**
because smoke mode lacks traceback detail, which is essential for debugging.
Quick mode (`--tb=line`) is better for targeted debugging, and full mode
(`-v --tb=short`) is needed for pre-merge validation.

#### Alternative 5 — Use `make test` as universal abstraction

Define a `Makefile` with `make test-quick`, `make test-full`, `make test-smoke`
in every project. **Rejected** because it requires every project to have a
Makefile and adds a per-project maintenance burden. The detection script
(Section 5) already handles `make test` as a fallback.

#### Alternative 6 — Makefile-based abstraction

Each project defines `make test-quick`, `make test-full`, `make test-smoke`
targets. Agents just run `make test-<mode>`. This would be fully
project-agnostic. **Rejected** because it adds per-project Makefile burden and
does not eliminate the need for detection logic — the agent still needs to
know which make targets exist and whether `make` is even the right tool.

#### Alternative 7 — Pytest profiles in pyproject.toml

Register the three modes as pytest profiles using `[tool.pytest.ini_options]`.
Then agents run `pytest --override-ini="addopts=-x -q"` etc. **Rejected**
because not all projects use `pyproject.toml`, and it only works for Python
projects. The standard must be universal across all frameworks.

## Consequences

### Positive

1. **Consistent test output** across all six agent teams — same flags, same
   behaviour, same token cost per mode.
2. **Reduced token waste** — no more `rtk pytest` failures, no more ad-hoc
   flag experimentation. Estimated 200-400 tokens saved per test invocation.
3. **Faster debugging** — quick mode with `--tb=line` gives developers
   immediately useful traceback without the noise of `--tb=long`.
4. **Clear migration path** for agent files — each team's changes are
   precisely scoped (testers.md lines 68-73, engineers.md new step 5b,
   fls.md line 99, reviewers.md checklist addition).
5. **Framework-agnostic** — the detection script in Section 5 covers 11
   frameworks, with an extensible priority-ordered table.

### Negative

1. **Initial adoption friction** — all four team agent files must be updated
   simultaneously. Outdated agents will continue to use ad-hoc invocations.
2. **Detection script maintenance** — the framework detection table will need
   updates as new languages/tools are adopted. This is a living document.
3. **PYTHONPATH heuristics** — the `src/` → `lib/` → `.` fallback may not fit
   all project layouts. Projects with unusual source directory names (e.g.,
   `app/`, `core/`) will need an explicit override.

### Follow-ups

1. Update testers.md, engineers.md, fls.md, and reviewers.md with the changes
   specified in Section 6.
2. If the detection script proves useful, promote it to a formal skill
   (`.opencode/skills/universal-test-execution/SKILL.md`) per Section 7.2.
3. Add an explicit override mechanism for non-standard PYTHONPATH layouts
   (e.g., a `.kodehold-test-config` file).
4. Create a pre-execution check in agent startup templates to warn if
   `rtk pytest` is attempted.
