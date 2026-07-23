# Shipping Gate

## Phase 0: Team Meeting (manual)

All teams approve or block (Architects, Engineers, Testers, Reviewers, Scribes, FLS — Second Opinion is excluded as it is a cross-model validator, not a decision-making team). See ADR-0011. Must complete before Phase 1.

## Phase 1: Pre-ship Verification (automated)

Run: `python3 scripts/ship.py`

This verifies: VERSION.md exists + parses, CHANGES.md entry exists, TODO.md exists, tests pass, git status clean, branch check.

## Phase 2: Manual Shipping Actions (Director executes AFTER ship.py passes)

| # | Action | Delegated to |
|---|--------|-------------|
| 1 | Bump VERSION.md (MAJOR/MINOR/PATCH) | Scribes |
| 2 | Update CHANGES.md with version + date + changes | Scribes |
| 3 | Update TODO.md — mark completed items [x] | Scribes |
| 4 | Store release note: `add_memory(content=<release-note>, tags=['release'])` | Director |
| 5 | Delegate structured commit: `<type>(<scope>): <desc>` | Scribes |
| 6 | Push: `git push` | Director |
| 7 | Tag: `git tag v<ver> && git push origin v<ver>` | Director |

**CRITICAL:** ship.py is a verification gate only. It does NOT execute shipping actions.
Do NOT stop after ship.py passes — you must complete Phase 2 manually.

**Blocked if:** any team blocks in Phase 0, ship.py fails in Phase 1, or any Phase 2 step fails.
