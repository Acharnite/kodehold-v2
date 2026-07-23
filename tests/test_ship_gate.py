"""Shipping gate tests — scripts/ship.py.

Replaces bash tests/integration/07-ship-gate.sh.
"""

import subprocess
from pathlib import Path


from .conftest import run_script, setup_ship_project


# ===================================================================
# Test 1: Missing VERSION.md
# ===================================================================

def test_ship_missing_version(project_root: Path, tmp_path: Path) -> None:
    """ship.py must fail when VERSION.md is missing."""
    setup_ship_project(tmp_path, project_root)
    (tmp_path / "VERSION.md").unlink()

    result = run_script(tmp_path / "scripts/ship.py", cwd=tmp_path)
    assert result.returncode != 0
    assert "VERSION.md not found" in result.stdout


# ===================================================================
# Test 2: Missing CHANGES.md
# ===================================================================

def test_ship_missing_changelog(project_root: Path, tmp_path: Path) -> None:
    """ship.py must fail when CHANGES.md is missing."""
    setup_ship_project(tmp_path, project_root)
    (tmp_path / "CHANGES.md").unlink()

    result = run_script(tmp_path / "scripts/ship.py", cwd=tmp_path)
    assert result.returncode != 0
    assert "CHANGES.md not found" in result.stdout


# ===================================================================
# Test 3: Missing TODO.md
# ===================================================================

def test_ship_missing_todo(project_root: Path, tmp_path: Path) -> None:
    """ship.py must fail when TODO.md is missing."""
    setup_ship_project(tmp_path, project_root)
    (tmp_path / "TODO.md").unlink()

    result = run_script(tmp_path / "scripts/ship.py", cwd=tmp_path)
    assert result.returncode != 0
    assert "TODO.md not found" in result.stdout


# ===================================================================
# Test 4: Version mismatch
# ===================================================================

def test_ship_version_mismatch(project_root: Path, tmp_path: Path) -> None:
    """ship.py must fail when VERSION.md and CHANGES.md versions differ."""
    setup_ship_project(tmp_path, project_root)
    # Override VERSION.md with a version not in CHANGES.md
    (tmp_path / "VERSION.md").write_text("| 1.0.0 |\n")

    result = run_script(tmp_path / "scripts/ship.py", cwd=tmp_path)
    assert result.returncode != 0
    assert "No entry for v1.0.0" in result.stdout


# ===================================================================
# Test 5: All checks pass
# ===================================================================

def test_ship_all_checks_pass(project_root: Path, tmp_path: Path) -> None:
    """ship.py must pass when all conditions are met (with git init)."""
    setup_ship_project(tmp_path, project_root)

    # Initialize git repo so git checks pass
    subprocess.run(
        ["git", "init", "--initial-branch=main"],
        cwd=str(tmp_path), capture_output=True,
    )
    subprocess.run(
        ["git", "-C", str(tmp_path), "config", "user.email", "test@test.com"],
        capture_output=True,
    )
    subprocess.run(
        ["git", "-C", str(tmp_path), "config", "user.name", "Test"],
        capture_output=True,
    )
    subprocess.run(
        ["git", "-C", str(tmp_path), "add", "-A"],
        capture_output=True,
    )
    subprocess.run(
        ["git", "-C", str(tmp_path), "commit", "-m", "initial"],
        capture_output=True,
    )

    result = run_script(tmp_path / "scripts/ship.py", cwd=tmp_path)
    assert result.returncode == 0
    assert "Pre-ship Checks Passed" in result.stdout


# ===================================================================
# Test 6: Tests fail
# ===================================================================

def test_ship_tests_fail(project_root: Path, tmp_path: Path) -> None:
    """ship.py must fail when tests/run.sh exits non-zero."""
    setup_ship_project(tmp_path, project_root)
    # Override test runner to fail
    (tmp_path / "tests/run.sh").write_text(
        "#!/usr/bin/env bash\necho \"Tests failing\"\nexit 1\n"
    )
    (tmp_path / "tests/run.sh").chmod(0o755)

    result = run_script(tmp_path / "scripts/ship.py", cwd=tmp_path)
    assert result.returncode != 0
    assert "Test suite failed" in result.stdout
