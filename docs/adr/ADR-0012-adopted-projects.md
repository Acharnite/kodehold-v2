---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0012: Adopted Projects — Existing Codebases in KodeHold

## Status

Accepted

## Context

KodeHold manages projects through a structured lifecycle (INIT → ACTIVE → REVIEW → CLOSED → REOPEN). The `workspace.sh init` command creates a fresh project from scratch — design doc, ADR index, source tree, and state file. This works well for greenfield projects but excludes the vast majority of real-world scenarios:

- An existing open-source library that needs maintenance
- A legacy codebase being modernised
- A project that was started outside KodeHold but is now being managed by it
- A third-party dependency that needs to be tracked and patched

The key forces are:
- Existing projects already have code, tests, git history, and build systems — KodeHold must not disrupt these
- The project must remain at its original location — moving it into `workspaces/` is not acceptable
- KodeHold artifacts (design doc, ADRs, ICM) must be co-located with the project without polluting it
- The design process must be retroactive: describe what exists, not what will be built
- The lifecycle gates must be relaxed for adopted projects — they already pass code existence checks by definition
- Future feature work on adopted projects should follow the normal ACTIVE → REVIEW → CLOSED flow

## Decision

We introduce `workspace.sh adopt <name> <path>` — a command that registers an existing project under KodeHold management without moving or copying it.

### Mechanism: Symlink + Sidecar Artifacts

```
workspaces/<name> → /path/to/existing/project  (symlink)
/path/to/existing/project/
├── .kodehold-state       # Lifecycle state with ADOPTED=true flag
├── docs/
│   ├── design/
│   │   └── README.md     # Retroactive design doc
│   └── adr/
│       ├── README.md     # ADR index (initially empty)
│       └── ADR-*.md      # Written retroactively
```

The symlink approach ensures:
- The project stays at its original path — no moves, no copies
- `workspaces/<name>` resolves transparently to the real project
- KodeHold artifacts live alongside the project, not inside the KodeHold repo
- The catalog records both `path` (symlink) and `real_path` (target)

### Catalog Entry

Adopted projects are registered in `workspaces/.catalog` with an `origin: "adopted"` field and `real_path` pointing to the actual project directory. This allows the list command to detect missing symlinks and differentiate adopted from native projects.

### Relaxed Gates

The INIT→ACTIVE gate checks `ADOPTED=true` in `.kodehold-state` and applies relaxed rules:
- **Implementation Plan** section is optional (the project is already built)
- **ADRs** are not required initially (written retroactively as architecture is understood)
- All other design doc sections are still required — they must be filled in retroactively

### Project Scan

On adoption, the script auto-detects:
- Language (package.json → JS/TS, Cargo.toml → Rust, pyproject.toml → Python, go.mod → Go)
- Build system (npm, cargo, pip, go, make)
- Test framework (jest, vitest, pytest, go test, cargo test)
- File count and git commit count
This information is pre-filled into the design doc's Architecture Overview section.

### Retroactive Design

All future feature work on adopted projects follows the standard lifecycle:
- ACTIVE: Engineers implement, Testers test, Reviewers review
- REVIEW: Team Meeting sign-off
- CLOSED: Full context stored in ICM
- REOPEN: Standard reopen protocol

The only difference is the INIT phase — instead of designing forward, Architects describe backward.

## Consequences

- Positive: Existing projects can be managed without restructuring their directory layout
- Positive: Symlinks are transparent — all tools (git, editors, build systems) work unchanged
- Positive: Retroactive design doc forces understanding of the codebase before changes
- Positive: Relaxed gates match reality — code already exists, ADRs can be written as time allows
- Negative: Symlinks can break if the target is moved or the symlink is not recreated (mitigated by catalog tracking)
- Negative: Retroactive design is inherently less precise than forward design — gaps in understanding may surface later
- Negative: The .kodehold-state file is added to the project root, which may feel like "pollution" (mitigated by .gitignore)
- Neutral: Adopted projects start in INIT but can transition to ACTIVE immediately after design doc is filled in
