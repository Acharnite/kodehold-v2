#!/usr/bin/env python3
"""KodeHold Shipping Gate — automated steps 1-7 of 8-step shipping process.

Step 0 (Team Meeting) must be completed manually before running this script.

Usage:
    python3 scripts/ship.py                          # Run shipping gate checks
    python3 scripts/ship.py --generate-changes       # Auto-generate CHANGES.md entry
    python3 scripts/ship.py --json                   # JSON output mode
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Optional

# Ensure project root is in sys.path
_PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(_PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(_PROJECT_ROOT))

from scripts.lib.output import (  # noqa: E402
    pass_msg,
    warn,
    json_add,
    json_emit,
    set_json_mode,
    is_json_mode,
    is_self_modification,
    reset_checks,
)

SHIP_FAILED = 0


def fail_hard(msg: str) -> None:
    """Print error and exit (for pre-check failures)."""
    global SHIP_FAILED
    if is_json_mode():
        SHIP_FAILED = 1
        json_add("ship_check", "FAIL", msg)
    else:
        print(f"  ✗ {msg}")
        sys.exit(1)


# ── Version helpers ───────────────────────────────────────────────────────


def parse_version() -> Optional[str]:
    """Parse version from VERSION.md."""
    path = Path("VERSION.md")
    if not path.is_file():
        return None
    try:
        text = path.read_text()
        m = re.search(r"(?m)^\|\s*(\d+\.\d+\.\d+)\s*\|", text)
        if m:
            return m.group(1).strip()
    except OSError:
        pass
    return None


# ── generate-changes ──────────────────────────────────────────────────────


def generate_changes() -> None:
    """Auto-generate a CHANGES.md entry from git log since last tag."""
    ver = parse_version()
    if not ver:
        fail_hard("Could not parse version from VERSION.md")

    today_str = date.today().isoformat()

    # Check if entry already exists
    changes_path = Path("CHANGES.md")
    if changes_path.is_file():
        if re.search(rf"^## {re.escape(ver)} ", changes_path.read_text(), re.MULTILINE):
            warn(f"CHANGES.md already has an entry for v{ver} — skipping generation")
            return

    # Get last tag
    last_tag = ""
    try:
        result = subprocess.run(
            ["git", "describe", "--tags", "--abbrev=0", "HEAD"],
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            last_tag = result.stdout.strip()
    except OSError:
        pass

    # Get commits since last tag
    raw_log = ""
    try:
        if last_tag:
            result = subprocess.run(
                ["git", "log", "--oneline", f"{last_tag}..HEAD"],
                capture_output=True,
                text=True,
            )
        else:
            result = subprocess.run(
                ["git", "log", "--oneline"],
                capture_output=True,
                text=True,
            )
        if result.returncode == 0:
            raw_log = result.stdout.strip()
    except OSError:
        pass

    if not raw_log:
        warn("No commits found since last tag — nothing to generate")
        return

    # Classify commits
    added: list[str] = []
    changed: list[str] = []
    fixed: list[str] = []
    docs: list[str] = []
    ci: list[str] = []
    other: list[str] = []

    for line in raw_log.splitlines():
        # Strip hash prefix
        subject = line.split(" ", 1)[1] if " " in line else line
        # Strip conventional commit scope prefix
        desc = re.sub(r"^[a-z]+(\([^)]*\))?[ :]\s*", "", subject)
        # Capitalize first letter
        desc = desc[0].upper() + desc[1:] if desc else desc

        if re.match(r"^(feat|add)", subject, re.IGNORECASE):
            added.append(f"- {desc}")
        elif re.match(r"^(fix|bug)", subject, re.IGNORECASE):
            fixed.append(f"- {desc}")
        elif re.match(r"^docs", subject, re.IGNORECASE):
            docs.append(f"- {desc}")
        elif re.match(r"^refactor", subject, re.IGNORECASE):
            changed.append(f"- {desc}")
        elif re.match(r"^(ci|chore)", subject, re.IGNORECASE):
            ci.append(f"- {desc}")
        elif re.match(r"^test", subject, re.IGNORECASE):
            continue  # skip internal test commits
        else:
            other.append(f"- {desc}")

    # Build entry
    entry_parts = [f"## {ver} — {today_str}"]
    if added:
        entry_parts.append("\n### Added\n" + "\n".join(added))
    if changed:
        entry_parts.append("\n### Changed\n" + "\n".join(changed))
    if fixed:
        entry_parts.append("\n### Fixed\n" + "\n".join(fixed))
    if docs:
        entry_parts.append("\n### Docs\n" + "\n".join(docs))
    if ci:
        entry_parts.append("\n### CI\n" + "\n".join(ci))
    if other:
        entry_parts.append("\n### Other\n" + "\n".join(other))

    entry = "".join(entry_parts)

    # Insert into CHANGES.md
    if changes_path.is_file():
        content = changes_path.read_text()
        # Find first "## " line after "# Changelog" header
        m = re.search(r"^## ", content, re.MULTILINE)
        if m:
            insert_pos = m.start()
            new_content = (
                content[:insert_pos] + entry + "\n\n" + content[insert_pos:]
            )
        else:
            new_content = content + "\n" + entry + "\n"
    else:
        new_content = "# Changelog\n\n" + entry + "\n"

    changes_path.write_text(new_content)
    pass_msg(f"Generated CHANGES.md entry for v{ver} ({today_str})")
    print()
    print("Generated entry:")
    print("────────────────")
    print(entry)
    print("────────────────")
    print()
    print("Review and edit CHANGES.md before shipping.")


# ── Shipping checks ───────────────────────────────────────────────────────


def run_ship_checks() -> None:
    """Run all 6 shipping gate checks."""
    global SHIP_FAILED

    print() if not is_json_mode() else None
    if not is_json_mode():
        now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
        print("=" * 42)
        print("  KodeHold Shipping Gate")
        print(f"  {now}")
        print("=" * 42)
        print()

    # 1. Version check
    current_ver = parse_version()
    ver_path = Path("VERSION.md")
    if not ver_path.is_file():
        fail_hard("VERSION.md not found")
    if current_ver:
        json_add("version_file", "PASS", current_ver)
        pass_msg(f"Current version: {current_ver}")
    else:
        json_add("version_file", "FAIL")
        fail_hard("Could not parse version from VERSION.md")
    print() if not is_json_mode() else None

    if SHIP_FAILED:
        return  # Can't continue without version

    # 2. CHANGES.md check
    changes_path = Path("CHANGES.md")
    if not changes_path.is_file():
        fail_hard("CHANGES.md not found")
    changes_text = changes_path.read_text()
    if re.search(rf"^## {re.escape(current_ver)} ", changes_text, re.MULTILINE):
        json_add("changelog", "PASS")
        pass_msg(f"CHANGES.md entry found for v{current_ver}")
    else:
        json_add("changelog", "FAIL", f"No entry for v{current_ver}")
        fail_hard(f"No entry for v{current_ver} in CHANGES.md — add one before shipping")
    print() if not is_json_mode() else None

    # 3. TODO.md check
    if Path("TODO.md").is_file():
        json_add("todo_file", "PASS")
        pass_msg("TODO.md exists")
    else:
        json_add("todo_file", "FAIL")
        fail_hard("TODO.md not found")
    print() if not is_json_mode() else None

    # 4. Test suite
    test_runner = Path("tests/run.sh")
    if test_runner.is_file():
        if is_json_mode():
            result = subprocess.run(
                ["bash", str(test_runner)], capture_output=True
            )
            if result.returncode == 0:
                json_add("tests", "PASS")
                pass_msg("All tests pass")
            else:
                json_add("tests", "FAIL")
                fail_hard("Test suite failed — fix before shipping")
        else:
            result = subprocess.run(["bash", str(test_runner)])
            if result.returncode == 0:
                pass_msg("All tests pass")
            else:
                fail_hard("Test suite failed — fix before shipping")
    else:
        json_add("tests", "FAIL", "tests/run.sh not found")
        fail_hard("tests/run.sh not found")
    print() if not is_json_mode() else None

    # 5. Git status
    try:
        staged = subprocess.run(
            ["git", "diff", "--stat", "--cached"], capture_output=True, text=True
        )
        unstaged = subprocess.run(
            ["git", "diff", "--stat"], capture_output=True, text=True
        )
        untracked = subprocess.run(
            ["git", "ls-files", "--others", "--exclude-standard"],
            capture_output=True,
            text=True,
        )

        if staged.stdout.strip():
            json_add("git_status", "PASS", "staged")
            pass_msg("Changes staged for commit")
        elif unstaged.stdout.strip():
            json_add("git_status", "WARN", "unstaged changes")
            warn("Unstaged changes exist — stage them before committing")
        elif untracked.stdout.strip():
            json_add("git_status", "WARN", "untracked files")
            warn("Untracked files exist — check git status")
        else:
            json_add("git_status", "PASS", "clean")
    except OSError:
        warn("Git not available — skipping git status check")
    print() if not is_json_mode() else None

    # 6. PR/branch check
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            text=True,
        )
        branch = result.stdout.strip()
        if branch == "main":
            json_add("branch", "PASS", "main")
            pass_msg("On main branch — direct push")
        else:
            json_add("branch", "WARN", branch)
            warn(f"On branch '{branch}' — remember to create PR: gh pr create")
    except OSError:
        warn("Git not available — skipping branch check")
    print() if not is_json_mode() else None


# ── CLI ───────────────────────────────────────────────────────────────────


def main() -> None:
    """Main entry point."""
    global SHIP_FAILED

    parser = argparse.ArgumentParser(
        description="KodeHold Shipping Gate — pre-ship checks",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--generate-changes",
        action="store_true",
        help="Auto-generate CHANGES.md entry from git log",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="json_mode",
        help="Output results as JSON",
    )
    args = parser.parse_args()

    if args.json_mode:
        set_json_mode(True)

    reset_checks()

    # ── Self-modification check ───────────────────────────────────────────
    # If KodeHold is modifying itself, skip shipping gate to avoid circular
    # self-gating. Detection is based on env var, marker file, or git diff.
    if is_self_modification() and not args.generate_changes:
        if is_json_mode():
            json_add("self_modification", "PASS", "KodeHold self-modification detected — shipping gate skipped")
            json_emit("ship.py", "PASS")
            sys.exit(0)
        print()
        print("  \033[1;33m━━━ KodeHold self-modification detected — skipping shipping gate ━━━\033[0m")
        print()
        sys.exit(0)

    if args.generate_changes:
        print()
        print("=" * 42)
        print("  Generating CHANGES.md entry")
        print("=" * 42)
        print()
        generate_changes()
        sys.exit(0)

    # Run full shipping gate
    run_ship_checks()

    current_ver = parse_version() or ""

    if is_json_mode():
        result_status = "FAIL" if SHIP_FAILED else "PASS"
        json_emit("ship.sh", result_status, version=current_ver)
        sys.exit(1 if SHIP_FAILED else 0)

    if SHIP_FAILED:
        sys.exit(1)

    print("=" * 42)
    print("  Pre-ship Checks Passed (6/6)")
    print("=" * 42)
    print()
    print("  Director: you must now manually execute:")
    print("    1. Bump VERSION.md (MAJOR/MINOR/PATCH)")
    print("    2. Update CHANGES.md with version + date + changes")
    print("    3. Store release: Store release note via add_memory(tags=['release'], scope='project')")
    print("    4. Delegate structured commit to Scribes")
    print("    5. Push: git push")
    print("    6. Tag: git tag v<ver> && git push origin v<ver>")
    print()
    print("  See director.md § Shipping Gate for full protocol.")
    print()
    sys.exit(0)


if __name__ == "__main__":
    main()
