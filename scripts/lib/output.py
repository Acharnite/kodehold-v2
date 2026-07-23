"""Shared output formatting for KodeHold scripts.

Provides colored terminal output, JSON mode, and check tracking.
All functions respect JSON_MODE to suppress human-readable output.
"""

from __future__ import annotations

import json
import os
import subprocess
from typing import Optional

# ANSI color constants
RED = "\033[0;31m"
GREEN = "\033[0;32m"
YELLOW = "\033[1;33m"
CYAN = "\033[0;36m"
NC = "\033[0m"

# Module-level state
_JSON_MODE = False
_JSON_CHECKS: list[dict[str, str]] = []


# ── Self-modification detection ──────────────────────────────────────────
# KodeHold self-modification system paths — if changes are detected to any
# of these files, the gate system assumes KodeHold is modifying itself and
# skips quality checks (avoiding circular self-gating).
KODEHOLD_SYSTEM_PATHS: list[str] = [
    "scripts/gate.py",
    "scripts/ship.py",
    "scripts/workspace.py",
    "scripts/lib/output.py",
    ".opencode/agents/",
    "opencode.json",
    "AGENTS.md",
]


def is_self_modification(project_path: str = "") -> bool:
    """Check whether the gate is running on KodeHold itself.

    Detection order: env var → marker file → git diff on system paths.

    Args:
        project_path: If set, the gate is running on a workspace project
                      (not KodeHold itself), so auto-detection is skipped.

    Returns:
        True if self-modification is detected, False otherwise.
    """
    # 1. Explicit environment variable
    if os.environ.get("KODEHOLD_SELF_MODE") == "1":
        return True

    # 2. Marker file in project root
    if os.path.isfile(".kodehold-self-mode"):
        return True

    # 3. Auto-detection: only in the KodeHold root (gate scripts exist) AND
    #    no project_path was given (meaning we're not checking a workspace)
    if os.path.isfile("scripts/gate.py") and not project_path:
        try:
            result = subprocess.run(
                ["git", "diff", "--name-only", "HEAD"],
                capture_output=True, text=True,
            )
            if result.returncode != 0:
                result = subprocess.run(
                    ["git", "diff", "--name-only"],
                    capture_output=True, text=True,
                )
            changed = result.stdout.strip().splitlines()
            for pattern in KODEHOLD_SYSTEM_PATHS:
                for f in changed:
                    if f.startswith(pattern.rstrip("/")):
                        return True
        except OSError:
            pass

    return False


def set_json_mode(enabled: bool = True) -> None:
    """Enable or disable JSON output mode."""
    global _JSON_MODE
    _JSON_MODE = enabled


def is_json_mode() -> bool:
    """Check if JSON output mode is active."""
    return _JSON_MODE


def pass_msg(msg: str) -> None:
    """Print a green checkmark message (silenced in JSON mode)."""
    if not _JSON_MODE:
        print(f"  {GREEN}✓{NC} {msg}")


def fail_msg(msg: str) -> None:
    """Print a red X message (silenced in JSON mode)."""
    if not _JSON_MODE:
        print(f"  {RED}✗{NC} {msg}")


def warn(msg: str) -> None:
    """Print a yellow warning message (silenced in JSON mode)."""
    if not _JSON_MODE:
        print(f"  {YELLOW}⚠{NC} {msg}")


def info(msg: str) -> None:
    """Print a cyan info message (silenced in JSON mode)."""
    if not _JSON_MODE:
        print(f"  {CYAN}i{NC} {msg}")


def json_add(name: str, status: str, detail: Optional[str] = None) -> None:
    """Record a named check result for JSON output."""
    entry: dict[str, str] = {"name": name, "result": status}
    if detail:
        entry["detail"] = detail
    _JSON_CHECKS.append(entry)


def json_emit(
    script: str,
    result: str,
    version: Optional[str] = None,
    transition: Optional[str] = None,
) -> str:
    """Emit accumulated checks as a JSON object and print it.

    Returns the JSON string for testing/inspection.
    """
    payload: dict = {
        "script": script,
        "result": result,
        "checks": list(_JSON_CHECKS),
    }
    if version:
        payload["version"] = version
    if transition:
        payload["transition"] = transition
    text = json.dumps(payload, indent=2)
    print(text)
    return text


def reset_checks() -> None:
    """Clear accumulated check results (for testing or re-runs)."""
    _JSON_CHECKS.clear()


def emit_reviewer_output(
    result: str,
    transition: str,
    validate_only: bool,
    checks: dict[str, str],
    markers_required: str,
    markers_cleanup: list[str],
) -> None:
    """Emit structured reviewer mode output (key=value pairs)."""
    print("─── Reviewer Mode Output ───")
    print(f"GATE_RESULT:{result}")
    print(f"TRANSITION:{transition}")
    print(f"VALIDATE_ONLY:{str(validate_only).lower()}")
    checks_line = ",".join(f"{k}:{v}" for k, v in checks.items())
    print(f"CHECKS:{checks_line}")
    print(f"MARKERS_REQUIRED:{markers_required}")
    if markers_cleanup:
        print(f"MARKERS_CLEANUP:{' '.join(markers_cleanup)}")
    print("────────────────────────────")
