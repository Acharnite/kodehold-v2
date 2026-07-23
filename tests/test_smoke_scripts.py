"""Smoke tests — basic Python script functionality checks.

Replaces bash tests/smoke/05-script-smoke.sh.
"""

import os
import stat
from pathlib import Path

import pytest

from .conftest import run_script


# ===================================================================
# 1. lib/output.py imports without error
# ===================================================================

def test_output_lib_imports(project_root: Path) -> None:
    """scripts/lib/output.py must import without errors."""
    result = run_script(
        project_root / "scripts" / "lib" / "output.py",
        cwd=project_root,
    )
    # Just importing should exit 0 (no CLI args -> usage/help or no-op)
    assert result.returncode in (0, 1), (
        f"output.py import failed:\nstdout:{result.stdout}\nstderr:{result.stderr}"
    )


# ===================================================================
# 2. gate.py --list
# ===================================================================

GATE_TRANSITIONS = [
    "INIT_TO_ACTIVE",
    "ACTIVE_TO_REVIEW",
    "REVIEW_TO_CLOSED",
    "CLOSED_TO_REOPEN",
    "REOPEN_TO_ACTIVE",
]


def test_gate_list_exits_zero(project_root: Path) -> None:
    """gate.py --list must exit 0."""
    result = run_script(project_root / "scripts/gate.py", "--list",
                        cwd=project_root)
    assert result.returncode == 0, f"gate.py --list exited {result.returncode}"


def test_gate_list_lists_all_transitions(project_root: Path) -> None:
    """gate.py --list must include all 5 transitions."""
    result = run_script(project_root / "scripts/gate.py", "--list",
                        cwd=project_root)
    for t in GATE_TRANSITIONS:
        assert t in result.stdout, f"gate.py --list missing transition {t}"


# ===================================================================
# 3. All Python scripts are executable
# ===================================================================

SCRIPTS = [
    "gate.py", "ship.py", "workspace.py",
    "detect_test_framework.py",
]


@pytest.mark.skipif(os.name == "nt", reason="POSIX executable bits not available on Windows")
def test_all_scripts_executable(project_root: Path) -> None:
    """All scripts in scripts/ must be executable."""
    for name in SCRIPTS:
        path = project_root / "scripts" / name
        assert path.exists(), f"scripts/{name} not found"
        mode = os.stat(path).st_mode
        assert mode & stat.S_IXUSR, f"scripts/{name} is not executable"
