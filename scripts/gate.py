#!/usr/bin/env python3
"""KodeHold Lifecycle Gate — run automated quality checks before state transitions.

Usage:
    python3 scripts/gate.py --transition INIT_TO_ACTIVE
    python3 scripts/gate.py --transition ACTIVE_TO_REVIEW --validate-only
    python3 scripts/gate.py --project-path workspaces/my-project --status
    python3 scripts/gate.py --list
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

# Ensure project root is in sys.path so we can import scripts.lib
_PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(_PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(_PROJECT_ROOT))

from scripts.lib.output import (  # noqa: E402
    pass_msg,
    fail_msg,
    warn,
    info,
    json_add,
    json_emit,
    set_json_mode,
    is_json_mode,
    is_self_modification,
    reset_checks,
    emit_reviewer_output,
    YELLOW,
    NC,
)

# ── Constants ─────────────────────────────────────────────────────────────

STATE_FILE = ".kodehold-state"
DESIGN_DOC = "docs/design/README.md"
ADR_DIR = "docs/adr"
TEST_RUNNER = "tests/run.sh"

# Required design doc sections (the Implementation Plan check is handled separately)
DESIGN_SECTIONS = [
    "Purpose & Scope",
    "Requirements",
    "Architecture Overview",
    "Component Design",
    "Data Model",
    "API Design",
    "Testing Strategy",
    "ADR Index",
    "Open Questions",
    "Changelog",
]

# ── Module-level state ────────────────────────────────────────────────────

gate_failed: int = 0
cleanup_markers: list[str] = []
check_results: dict[str, str] = {}
_self_mod_handled: bool = False


# ── Helper functions ──────────────────────────────────────────────────────


def _handle_self_mod(args: argparse.Namespace) -> None:
    """Handle self-modification — skip gates, print banner, set up state for standard output.

    Does NOT exit — main() handles all output routing (JSON + reviewer)
    via the standard paths. This sets check_results so the standard
    paths have data.
    """
    global gate_failed, _self_mod_handled, check_results
    _self_mod_handled = True
    gate_failed = 0
    check_results = {"self_modification": "PASS"}
    json_add("self_modification", "PASS",
              "KodeHold self-modification detected — gate skipped")

    # In JSON mode, skip the banner — the standard JSON path handles output
    if is_json_mode():
        return

    print()
    print(f"  {YELLOW}━━━ KodeHold self-modification detected — skipping gate checks ━━━{NC}")
    print()


def check(cmd: str, *args: str) -> None:
    """Run a command; mark gate as failed if it returns non-zero."""
    global gate_failed
    result = subprocess.run([cmd, *args], capture_output=True)
    if result.returncode != 0:
        fail_msg(f"{cmd} {' '.join(args)} failed")
        gate_failed = 1


def assert_file(path: str) -> bool:
    """Assert a file exists; mark gate as failed if not."""
    global gate_failed
    if os.path.isfile(path):
        pass_msg(f"{path} exists")
        return True
    else:
        fail_msg(f"{path} not found")
        gate_failed = 1
        return False


def assert_dir(path: str) -> bool:
    """Assert a directory exists; mark gate as failed if not."""
    global gate_failed
    if os.path.isdir(path):
        pass_msg(f"{path} exists")
        return True
    else:
        fail_msg(f"{path} not found")
        gate_failed = 1
        return False


def queue_cleanup_marker(path: str) -> None:
    """Queue a marker file for cleanup on gate pass."""
    cleanup_markers.append(path)


def record_check(name: str, result: str) -> None:
    """Record a named check result."""
    check_results[name] = result
    json_add(name, result)


def cleanup_markers_on_pass() -> None:
    """Remove all queued marker files."""
    if not cleanup_markers:
        return
    for marker in cleanup_markers:
        if os.path.exists(marker):
            os.remove(marker)
    pass_msg(f"Lifecycle markers cleaned up: {' '.join(cleanup_markers)}")


def is_noninteractive() -> bool:
    """Check if running in non-interactive mode."""
    val = os.environ.get("OPENCODE_NONINTERACTIVE", "").lower()
    return val in ("true", "1", "yes")


def _run_tests() -> None:
    """Run the test suite, falling back to pytest if tests/run.sh doesn't exist."""
    global gate_failed
    tests_passed = False

    if os.path.isfile(TEST_RUNNER):
        if is_json_mode():
            result = subprocess.run(
                ["bash", TEST_RUNNER], capture_output=True
            )
            if result.returncode == 0:
                pass_msg(f"All tests pass ({TEST_RUNNER})")
                tests_passed = True
            else:
                fail_msg("Test suite has failures — fix before transition")
                gate_failed = 1
        else:
            result = subprocess.run(
                ["bash", TEST_RUNNER], capture_output=False
            )
            if result.returncode == 0:
                pass_msg(f"All tests pass ({TEST_RUNNER})")
                tests_passed = True
            else:
                fail_msg("Test suite has failures — fix before transition")
                gate_failed = 1
    elif _has_pytest_tests():
        pytest_cmd = _find_pytest_cmd()
        pytest_env = os.environ.copy()
        if os.path.isdir("src"):
            pytest_env["PYTHONPATH"] = "src"
        if is_json_mode():
            result = subprocess.run(
                [*pytest_cmd, "tests/", "-q"],
                capture_output=True,
                env=pytest_env,
            )
            if result.returncode == 0:
                pass_msg("All pytest tests pass")
                tests_passed = True
            else:
                fail_msg("Pytest suite has failures — fix before transition")
                gate_failed = 1
        else:
            result = subprocess.run(
                [*pytest_cmd, "tests/", "-q"],
                capture_output=False,
                env=pytest_env,
            )
            if result.returncode == 0:
                pass_msg("All pytest tests pass")
                tests_passed = True
            else:
                fail_msg("Pytest suite has failures — fix before transition")
                gate_failed = 1
    else:
        fail_msg("No test suite found (tests/run.sh or tests/test_*.py)")
        gate_failed = 1

    if tests_passed:
        record_check("tests_passing", "PASS")
    else:
        record_check("tests_passing", "FAIL")


def _has_pytest_tests() -> bool:
    """Check if there are pytest test files in the tests directory."""
    tests_dir = Path("tests")
    if not tests_dir.is_dir():
        return False
    return any(tests_dir.glob("test_*.py"))


def _find_pytest_cmd() -> list[str]:
    """Find the pytest command to use."""
    venv_pytest = Path(".venv/bin/pytest")
    if venv_pytest.is_file():
        return [str(venv_pytest)]
    return [sys.executable, "-m", "pytest"]


# ── Design doc section check ──────────────────────────────────────────────


def _check_design_section(doc_path: str, section: str, adopted: bool = False) -> None:
    """Check that a design doc section exists. Fail if missing (adopted projects may skip Implementation Plan)."""
    global gate_failed
    if section == "Implementation Plan" and adopted:
        # Optional for adopted projects
        if _grep_doc(doc_path, section):
            pass_msg(f"Design doc section: {section} (optional for adopted)")
        else:
            warn("No Implementation Plan — adopted project already exists")
        return

    if _grep_doc(doc_path, section):
        pass_msg(f"Design doc section: {section}")
    else:
        fail_msg(f"Design doc missing section: {section}")
        gate_failed = 1


def _grep_doc(doc_path: str, pattern: str) -> bool:
    """Check if a pattern exists in the design doc."""
    if not os.path.isfile(doc_path):
        return False
    try:
        with open(doc_path) as f:
            return pattern in f.read()
    except OSError:
        return False


# ── Transition functions ──────────────────────────────────────────────────


def _load_state() -> dict[str, str]:
    """Load state file key-value pairs."""
    state: dict[str, str] = {}
    if os.path.isfile(STATE_FILE):
        try:
            with open(STATE_FILE) as f:
                for line in f:
                    line = line.strip()
                    if "=" in line and not line.startswith("#"):
                        key, _, val = line.partition("=")
                        state[key.strip()] = val.strip()
        except OSError:
            pass
    return state


def init_to_active() -> None:
    """Validate INIT → ACTIVE transition."""
    global gate_failed
    print() if not is_json_mode() else None
    if not is_json_mode():
        print("━━━ Gate: INIT → ACTIVE ━━━\n")

    state = _load_state()
    adopted = state.get("ADOPTED", "").lower() == "true"

    if adopted:
        info("Adopted project detected — relaxing INIT→ACTIVE checks")

    # Design doc exists
    assert_file(DESIGN_DOC)

    # Check design doc sections
    for section in DESIGN_SECTIONS:
        _check_design_section(DESIGN_DOC, section, adopted)
    _check_design_section(DESIGN_DOC, "Implementation Plan", adopted)

    # ADR directory
    assert_dir(ADR_DIR)
    adr_count = len(list(Path(ADR_DIR).glob("ADR-*.md")))
    if adr_count >= 1:
        pass_msg(f"{adr_count} ADR(s) found")
    elif adopted:
        warn("No ADRs yet — write retroactive ADRs for key architectural decisions")
    else:
        fail_msg(f"No ADRs found in {ADR_DIR}")
        gate_failed = 1

    # ADR index
    assert_file(f"{ADR_DIR}/README.md")

    # Design doc status
    if _grep_doc(DESIGN_DOC, "Status.*Active"):
        pass_msg("Design doc status is Active")
    else:
        warn("Design doc status not set to Active")

    # Design reviewed marker
    if os.path.isfile(".design_reviewed"):
        pass_msg("Design reviewed and approved by Reviewers (sequence enforced)")
        queue_cleanup_marker(".design_reviewed")
        record_check("design_reviewed", "PASS")
    else:
        fail_msg(
            "Design not reviewed — Reviewers must approve design doc before INIT→ACTIVE"
        )
        gate_failed = 1
        record_check("design_reviewed", "FAIL")

    # Second opinion marker
    if os.path.isfile(".second_opinion_done"):
        pass_msg("Second opinion completed (cross-model validation)")
        queue_cleanup_marker(".second_opinion_done")
        record_check("second_opinion_done", "PASS")
    else:
        fail_msg(
            "Second opinion not completed — Reviewers must complete cross-model validation before INIT→ACTIVE"
        )
        gate_failed = 1
        record_check("second_opinion_done", "FAIL")

    # User review stop
    if gate_failed == 0 and not is_noninteractive():
        print()
        print("  ─── Design Review ───")
        print(f"  Design doc:  {DESIGN_DOC}")
        print("  ADRs:")
        for adr in sorted(Path(ADR_DIR).glob("ADR-*.md")):
            try:
                with open(adr) as f:
                    title = f.readline().strip().lstrip("# ")
            except OSError:
                title = ""
            print(f"    • {adr.name} — {title}")
        print()
        confirm = input("  Proceed with INIT → ACTIVE? [Y/n] ")
        if confirm.lower() in ("n", "no"):
            fail_msg("User cancelled — design needs changes")
            gate_failed = 1
        else:
            pass_msg("User approved — proceeding with transition")


def active_to_review() -> None:
    """Validate ACTIVE → REVIEW transition."""
    global gate_failed
    print() if not is_json_mode() else None
    if not is_json_mode():
        print("━━━ Gate: ACTIVE → REVIEW ━━━\n")

    # TODO.md check
    if os.path.isfile("TODO.md"):
        pass_msg("TODO.md exists")
    else:
        warn("No TODO.md — manual completeness check needed")

    # Tests pass
    _run_tests()

    # Testers completed marker
    if os.path.isfile(".testers_done"):
        pass_msg("Testers completed before review (sequence enforced)")
        queue_cleanup_marker(".testers_done")
        record_check("testers_done", "PASS")
    else:
        fail_msg(
            "Testers did not complete before review — must run Testers before Reviewers"
        )
        gate_failed = 1
        record_check("testers_done", "FAIL")

    # Code reviewed marker
    if os.path.isfile(".code_reviewed"):
        pass_msg("Code reviewed and approved by Reviewers")
        queue_cleanup_marker(".code_reviewed")
        record_check("code_reviewed", "PASS")
    else:
        fail_msg(
            "Code not reviewed — Reviewers must approve implementation before ACTIVE→REVIEW"
        )
        gate_failed = 1
        record_check("code_reviewed", "FAIL")

    # Check for recent review commits
    try:
        log = subprocess.run(
            ["git", "log", "--oneline", "-20"],
            capture_output=True,
            text=True,
        )
        if re.search(r"review|reviewed|approve", log.stdout, re.IGNORECASE):
            pass_msg("Recent review commits found")
        else:
            warn("No review commits in recent history — manual check needed")
    except OSError:
        warn("Git not available — skipping review commit check")


def review_to_closed() -> None:
    """Validate REVIEW → CLOSED transition."""
    global gate_failed
    print() if not is_json_mode() else None
    if not is_json_mode():
        print("━━━ Gate: REVIEW → CLOSED ━━━\n")

    # Tests green
    _run_tests()

    # Design doc exists
    has_design = assert_file(DESIGN_DOC)
    record_check("design_doc_exists", "PASS" if has_design else "FAIL")

    # Git is clean
    try:
        result = subprocess.run(
            ["git", "diff", "--quiet"], capture_output=True
        )
        if result.returncode == 0:
            pass_msg("Working tree clean")
        else:
            warn("Uncommitted changes exist — should commit before CLOSED")
    except OSError:
        warn("Git not available — skipping clean check")

    # Queue lifecycle markers for cleanup
    for marker in [
        ".design_reviewed",
        ".testers_done",
        ".impact_analysis_done",
        ".code_reviewed",
        ".second_opinion_done",
        ".team_meeting_done",
    ]:
        queue_cleanup_marker(marker)

    # User review stop
    if gate_failed == 0 and not is_noninteractive():
        confirm = input("  Proceed with REVIEW → CLOSED? (y/N) ")
        if confirm.lower() in ("y", "yes"):
            pass_msg("User approved — proceeding with transition")
        else:
            fail_msg("User cancelled — REVIEW → CLOSED aborted")
            gate_failed = 1


def closed_to_reopen() -> None:
    """Validate CLOSED → REOPEN transition."""
    global gate_failed
    print() if not is_json_mode() else None
    if not is_json_mode():
        print("━━━ Gate: CLOSED → REOPEN ━━━\n")

    # Design doc exists and has update timestamp
    has_design = assert_file(DESIGN_DOC)
    record_check("design_doc_exists", "PASS" if has_design else "FAIL")
    if _grep_doc(DESIGN_DOC, "Last Updated:"):
        pass_msg("Design doc has update timestamp")

    # Impact analysis notes
    decisions_dir = Path("docs/decisions")
    if decisions_dir.is_dir() and list(decisions_dir.glob("*.md")):
        pass_msg("Impact analysis notes found")
    else:
        warn("No impact analysis in docs/decisions/ — manual check needed")

    # Impact analysis completed marker
    if os.path.isfile(".impact_analysis_done"):
        pass_msg("Impact analysis completed by Architects (sequence enforced)")
        queue_cleanup_marker(".impact_analysis_done")
        record_check("impact_analysis_done", "PASS")
    else:
        fail_msg(
            "Impact analysis not completed — Architects must assess scope before CLOSED→REOPEN"
        )
        gate_failed = 1
        record_check("impact_analysis_done", "FAIL")

    # User confirmation prompt
    if gate_failed == 0 and not is_noninteractive():
        print()
        confirm = input("  Proceed with CLOSED → REOPEN? [Y/n] ")
        if confirm.lower() in ("n", "no"):
            fail_msg("User cancelled — impact analysis needs more work")
            gate_failed = 1
        else:
            pass_msg("User approved — proceeding with transition")


def reopen_to_active() -> None:
    """Validate REOPEN → ACTIVE transition."""
    global gate_failed
    print() if not is_json_mode() else None
    if not is_json_mode():
        print("━━━ Gate: REOPEN → ACTIVE ━━━\n")

    # Design doc exists
    has_design = assert_file(DESIGN_DOC)
    record_check("design_doc_exists", "PASS" if has_design else "FAIL")
    if _grep_doc(DESIGN_DOC, "Status.*Active"):
        pass_msg("Design doc status is Active")
    else:
        warn("Design doc status not set to Active")

    # ADR directory exists
    has_adr_dir = assert_dir(ADR_DIR)
    record_check("adr_dir_exists", "PASS" if has_adr_dir else "FAIL")

    # Second opinion marker
    if os.path.isfile(".second_opinion_done"):
        pass_msg("Second opinion completed for updated design")
        queue_cleanup_marker(".second_opinion_done")
        record_check("second_opinion_done", "PASS")
    else:
        fail_msg(
            "Second opinion not completed — Reviewers must complete cross-model validation before REOPEN→ACTIVE"
        )
        gate_failed = 1
        record_check("second_opinion_done", "FAIL")


# ── CLI ───────────────────────────────────────────────────────────────────


def build_parser() -> argparse.ArgumentParser:
    """Build the argument parser."""
    parser = argparse.ArgumentParser(
        description="KodeHold Lifecycle Gate — run quality checks for state transitions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Transitions:
  INIT_TO_ACTIVE      Design doc complete, ADRs written
  ACTIVE_TO_REVIEW    Features implemented, tests pass, code reviewed
  REVIEW_TO_CLOSED    Final sign-off, tests green, memories stored
  CLOSED_TO_REOPEN    Impact analysis, design doc updated
  REOPEN_TO_ACTIVE    Design doc approved, new ADRs

Examples:
  python3 scripts/gate.py --transition INIT_TO_ACTIVE
  python3 scripts/gate.py --transition ACTIVE_TO_REVIEW --validate-only
  python3 scripts/gate.py --project-path workspaces/my-project --status
  python3 scripts/gate.py --list
        """,
    )
    parser.add_argument(
        "--transition",
        choices=[
            "INIT_TO_ACTIVE",
            "ACTIVE_TO_REVIEW",
            "REVIEW_TO_CLOSED",
            "CLOSED_TO_REOPEN",
            "REOPEN_TO_ACTIVE",
        ],
        help="The state transition to validate",
    )
    parser.add_argument(
        "--project-path",
        type=str,
        default="",
        help="Path to project directory (default: current dir)",
    )
    # Boolean flags — collapsed via loop per ADR-0049
    for flag, help_text, kwargs in [
        ("--validate-only", "Run checks without executing transition (for Reviewers)", {}),
        ("--reviewer-mode", "Output structured results for Reviewers (PASS/BLOCKED per check)", {}),
        ("--json", "Output results as JSON", {"dest": "json_mode"}),
        ("--list", "List all gates and their checks", {}),
        ("--status", "Show current lifecycle state", {}),
        ("--yes", "Skip interactive prompts (for CI/automation)", {}),
    ]:
        parser.add_argument(flag, action="store_true", help=help_text, **kwargs)
    return parser


def main() -> None:
    """Main entry point."""
    global gate_failed, cleanup_markers, check_results

    parser = build_parser()
    args = parser.parse_args()

    # Handle --list early
    if args.list:
        print("Available transitions:")
        for t in [
            "INIT_TO_ACTIVE",
            "ACTIVE_TO_REVIEW",
            "REVIEW_TO_CLOSED",
            "CLOSED_TO_REOPEN",
            "REOPEN_TO_ACTIVE",
        ]:
            print(f"  {t}")
        sys.exit(0)

    # JSON mode
    if args.json_mode:
        set_json_mode(True)

    # Non-interactive mode
    if args.yes:
        os.environ["OPENCODE_NONINTERACTIVE"] = "true"

    # Change to project path if specified
    if args.project_path:
        proj_path = Path(args.project_path).resolve()
        if not proj_path.is_dir():
            print(
                f"Error: Project path not found: {proj_path}",
                file=sys.stderr,
            )
            sys.exit(1)
        os.chdir(str(proj_path))

    # Handle --status
    if args.status:
        if os.path.isfile(STATE_FILE):
            with open(STATE_FILE) as f:
                print(f.read(), end="")
        else:
            print("No state file found — project not initialized")
        sys.exit(0)

    # --transition is required for gate transitions (but not for --status or --list)
    if not args.transition:
        parser.print_usage()
        print("Error: --transition is required (or use --status / --list)")
        sys.exit(1)

    # Reset module state for each run
    gate_failed = 0
    cleanup_markers = []
    check_results = {}
    reset_checks()

    # ── Self-modification check ───────────────────────────────────────────
    # If KodeHold is modifying itself, skip all gate checks to avoid circular
    # self-gating. Detection is based on env var, marker file, or git diff.
    if is_self_modification(project_path=args.project_path):
        _handle_self_mod(args)

    # Dispatch to transition function (skip if self-mod handled)
    if not _self_mod_handled:
        transition_map = {
            "INIT_TO_ACTIVE": init_to_active,
            "ACTIVE_TO_REVIEW": active_to_review,
            "REVIEW_TO_CLOSED": review_to_closed,
            "CLOSED_TO_REOPEN": closed_to_reopen,
            "REOPEN_TO_ACTIVE": reopen_to_active,
        }

        transition_fn = transition_map.get(args.transition)
        if not transition_fn:
            print(f"Unknown transition: {args.transition}", file=sys.stderr)
            sys.exit(1)

        transition_fn()

    # Print result header
    print() if not is_json_mode() else None
    if gate_failed == 0:
        if args.validate_only:
            msg = "━━━ VALIDATION PASSED — transition is allowed ━━━"
        else:
            msg = "━━━ GATE PASSED ━━━"
        if not is_json_mode():
            print(f"  {msg}")
    else:
        if args.validate_only:
            msg = "━━━ VALIDATION BLOCKED — transition not allowed ━━━"
        else:
            msg = "━━━ GATE BLOCKED — fix failures above ━━━"
        if not is_json_mode():
            print(f"  {msg}")
    print() if not is_json_mode() else None

    # Determine markers for this transition (used by JSON/reviewer output)
    markers_for_transition = {
        "INIT_TO_ACTIVE": ".design_reviewed .second_opinion_done",
        "ACTIVE_TO_REVIEW": ".code_reviewed .testers_done",
        "REVIEW_TO_CLOSED": "",
        "CLOSED_TO_REOPEN": "",
        "REOPEN_TO_ACTIVE": ".second_opinion_done",
    }
    markers_required = markers_for_transition.get(args.transition, "")

    # JSON mode output
    if is_json_mode():
        result_status = "PASS" if gate_failed == 0 else "BLOCKED"
        json_emit("gate.sh", result_status, transition=args.transition)
        sys.exit(gate_failed)

    # Reviewer mode output
    if args.reviewer_mode:
        emit_reviewer_output(
            "PASS" if gate_failed == 0 else "BLOCKED",
            args.transition, args.validate_only,
            check_results, markers_required, cleanup_markers,
        )

    # In validate-only mode, do NOT clean markers or modify state
    if not args.validate_only and gate_failed == 0:
        cleanup_markers_on_pass()

        # Trigger memoir distillation for Scribes (ADR-0009 phase 4)
        if args.transition == "REVIEW_TO_CLOSED":
            Path(".distill_needed").touch()
            pass_msg("Memoir distillation marker created (.distill_needed)")

    sys.exit(gate_failed)


if __name__ == "__main__":
    main()
