"""Gate transition tests — all 5 lifecycle transitions.

Replaces bash tests/integration/06-gate-transitions.sh.
"""

from pathlib import Path


from .conftest import run_script, setup_gate_project


# ===================================================================
# ACTIVE_TO_REVIEW
# ===================================================================

def test_active_to_review_enforces_testers_done(project_root: Path,
                                                 tmp_path: Path) -> None:
    """ACTIVE_TO_REVIEW must fail without .testers_done."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".code_reviewed").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "ACTIVE_TO_REVIEW",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode != 0
    assert "Testers did not complete" in result.stdout


def test_active_to_review_enforces_code_reviewed(project_root: Path,
                                                  tmp_path: Path) -> None:
    """ACTIVE_TO_REVIEW must fail without .code_reviewed."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".testers_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "ACTIVE_TO_REVIEW",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode != 0
    assert "Code not reviewed" in result.stdout


def test_active_to_review_passes_and_cleans(project_root: Path,
                                             tmp_path: Path) -> None:
    """ACTIVE_TO_REVIEW passes with all markers and cleans up."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".testers_done").touch()
    (tmp_path / ".code_reviewed").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "ACTIVE_TO_REVIEW",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode == 0
    assert not (tmp_path / ".testers_done").exists()
    assert not (tmp_path / ".code_reviewed").exists()


# ===================================================================
# REVIEW_TO_CLOSED
# ===================================================================

def test_review_to_closed_test_failure(project_root: Path,
                                        tmp_path: Path) -> None:
    """REVIEW_TO_CLOSED must fail when tests fail."""
    setup_gate_project(tmp_path, project_root)
    # Override test runner to fail
    (tmp_path / "tests/run.sh").write_text(
        "#!/usr/bin/env bash\necho \"Tests failing\"\nexit 1\n"
    )
    (tmp_path / "tests/run.sh").chmod(0o755)

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "REVIEW_TO_CLOSED",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode != 0
    assert "Test suite has failures" in result.stdout


def test_review_to_closed_passes_and_cleans(project_root: Path,
                                             tmp_path: Path) -> None:
    """REVIEW_TO_CLOSED passes with passing tests, then cleans 6 markers + creates .distill_needed."""
    setup_gate_project(tmp_path, project_root)

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "REVIEW_TO_CLOSED",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode == 0

    # All 6 markers should be cleaned
    for marker in [".design_reviewed", ".testers_done", ".impact_analysis_done",
                   ".code_reviewed", ".second_opinion_done", ".team_meeting_done"]:
        assert not (tmp_path / marker).exists(), \
            f"{marker} not cleaned after REVIEW_TO_CLOSED"

    # .distill_needed should be created
    assert (tmp_path / ".distill_needed").exists(), \
        ".distill_needed not created after REVIEW_TO_CLOSED"


# ===================================================================
# CLOSED_TO_REOPEN
# ===================================================================

def test_closed_to_reopen_enforces_impact_analysis(project_root: Path,
                                                    tmp_path: Path) -> None:
    """CLOSED_TO_REOPEN must fail without .impact_analysis_done."""
    setup_gate_project(tmp_path, project_root)
    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "CLOSED_TO_REOPEN",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode != 0
    assert "Impact analysis not completed" in result.stdout


def test_closed_to_reopen_passes_and_cleans(project_root: Path,
                                             tmp_path: Path) -> None:
    """CLOSED_TO_REOPEN passes with .impact_analysis_done and cleans up."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".impact_analysis_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "CLOSED_TO_REOPEN",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode == 0
    assert not (tmp_path / ".impact_analysis_done").exists()


# ===================================================================
# REOPEN_TO_ACTIVE
# ===================================================================

def test_reopen_to_active_enforces_second_opinion(project_root: Path,
                                                   tmp_path: Path) -> None:
    """REOPEN_TO_ACTIVE must fail without .second_opinion_done."""
    setup_gate_project(tmp_path, project_root)
    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "REOPEN_TO_ACTIVE",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode != 0
    assert "Second opinion not completed" in result.stdout


def test_reopen_to_active_passes_and_cleans(project_root: Path,
                                             tmp_path: Path) -> None:
    """REOPEN_TO_ACTIVE passes with .second_opinion_done and cleans up."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".second_opinion_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "REOPEN_TO_ACTIVE",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode == 0
    assert not (tmp_path / ".second_opinion_done").exists()
