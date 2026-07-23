"""Gate marker enforcement tests — INIT_TO_ACTIVE and CLOSED_TO_REOPEN.

Replaces bash tests/integration/05-gate-marker-enforcement.sh.
"""

from pathlib import Path


from .conftest import run_script, setup_gate_project


# ===================================================================
# INIT_TO_ACTIVE — .design_reviewed enforcement
# ===================================================================

def test_init_to_active_enforces_design_reviewed(project_root: Path,
                                                  tmp_path: Path) -> None:
    """INIT_TO_ACTIVE must fail without .design_reviewed marker."""
    setup_gate_project(tmp_path, project_root)
    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "INIT_TO_ACTIVE",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode != 0
    assert "Design not reviewed" in result.stdout


def test_init_to_active_cancellation_retains_markers(project_root: Path,
                                                      tmp_path: Path) -> None:
    """When user cancels ('n'), .design_reviewed and .second_opinion_done survive."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".design_reviewed").touch()
    (tmp_path / ".second_opinion_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "INIT_TO_ACTIVE",
        cwd=tmp_path,
        input_text="n\n",
    )
    assert result.returncode != 0
    assert (tmp_path / ".design_reviewed").exists(), ".design_reviewed was removed"
    assert (tmp_path / ".second_opinion_done").exists(), ".second_opinion_done was removed"


def test_init_to_active_noninteractive_bypass(project_root: Path,
                                               tmp_path: Path) -> None:
    """OPENCODE_NONINTERACTIVE must bypass the confirmation prompt."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".design_reviewed").touch()
    (tmp_path / ".second_opinion_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "INIT_TO_ACTIVE",
        cwd=tmp_path,
        env={"OPENCODE_NONINTERACTIVE": "true"},
    )
    assert result.returncode == 0
    assert "Proceed with INIT" not in result.stdout, "Prompt should be skipped"
    # Markers removed after successful pass
    assert not (tmp_path / ".design_reviewed").exists(), ".design_reviewed not cleaned"
    assert not (tmp_path / ".second_opinion_done").exists(), ".second_opinion_done not cleaned"


def test_init_to_active_yes_flag(project_root: Path, tmp_path: Path) -> None:
    """--yes must bypass the confirmation prompt."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".design_reviewed").touch()
    (tmp_path / ".second_opinion_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--yes", "--transition", "INIT_TO_ACTIVE",
        cwd=tmp_path,
    )
    assert result.returncode == 0
    assert "Proceed with INIT" not in result.stdout, "Prompt should be skipped with --yes"
    assert not (tmp_path / ".design_reviewed").exists(), ".design_reviewed not cleaned"
    assert not (tmp_path / ".second_opinion_done").exists(), ".second_opinion_done not cleaned"


# ===================================================================
# CLOSED_TO_REOPEN — .impact_analysis_done enforcement
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


def test_closed_to_reopen_cancellation_retains_markers(project_root: Path,
                                                       tmp_path: Path) -> None:
    """CLOSED_TO_REOPEN must keep .impact_analysis_done when user cancels."""
    setup_gate_project(tmp_path, project_root)
    (tmp_path / ".impact_analysis_done").touch()

    result = run_script(
        tmp_path / "scripts/gate.py",
        "--transition", "CLOSED_TO_REOPEN",
        cwd=tmp_path,
        input_text="n\n",
    )
    assert result.returncode != 0
    assert (tmp_path / ".impact_analysis_done").exists()
