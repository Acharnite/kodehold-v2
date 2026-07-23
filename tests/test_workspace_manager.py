"""Workspace manager tests — scripts/workspace.py."""

import os
import shutil
from pathlib import Path

import pytest

from .conftest import run_script, setup_workspace_root


# ===================================================================
# Test 1: init creates workspace structure
# ===================================================================

def test_init_creates_workspace_structure(project_root: Path,
                                          tmp_path: Path) -> None:
    """workspace.py init must create the expected directory structure."""
    setup_workspace_root(tmp_path, project_root)

    result = run_script(
        tmp_path / "scripts/workspace.py", "init", "my-project",
        cwd=tmp_path, input_text="\n",
    )
    ws = tmp_path / "workspaces" / "my-project"

    assert result.returncode == 0
    assert ws.is_dir(), "workspace directory not created"
    assert (ws / ".kodehold-state").is_file(), ".kodehold-state not created"
    assert (ws / "docs/design/README.md").is_file(), "design doc not created"
    assert (ws / "docs/adr/README.md").is_file(), "ADR index not created"
    assert (ws / "src").is_dir(), "src/ not created"
    assert (ws / "tests").is_dir(), "tests/ not created"
    assert (ws / ".gitignore").is_file(), ".gitignore not created"
    assert (ws / ".git").is_dir(), ".git/ not initialized"


# ===================================================================
# Test 2: init rejects invalid slug
# ===================================================================

def test_init_rejects_invalid_slug(project_root: Path, tmp_path: Path) -> None:
    """workspace.py init must reject names not matching slug pattern."""
    setup_workspace_root(tmp_path, project_root)

    result = run_script(
        tmp_path / "scripts/workspace.py", "init", "My Project",
        cwd=tmp_path, input_text="\n",
    )
    assert result.returncode != 0
    assert "Invalid workspace name" in result.stdout


# ===================================================================
# Test 3: init rejects duplicate
# ===================================================================

def test_init_rejects_duplicate(project_root: Path, tmp_path: Path) -> None:
    """workspace.py init must reject duplicate workspace names."""
    setup_workspace_root(tmp_path, project_root)

    # First init succeeds
    run_script(
        tmp_path / "scripts/workspace.py", "init", "test-project",
        cwd=tmp_path, input_text="\n",
    )
    # Second init should fail
    result = run_script(
        tmp_path / "scripts/workspace.py", "init", "test-project",
        cwd=tmp_path, input_text="\n",
    )
    assert result.returncode != 0
    assert "already exists" in result.stdout


# ===================================================================
# Test 4: adopt creates symlink with ADOPTED=true
# ===================================================================

@pytest.mark.skipif(os.name == "nt", reason="Symlinks require admin on Windows")
def test_adopt_creates_symlink(project_root: Path, tmp_path: Path) -> None:
    """workspace.py adopt must create a symlink with ADOPTED=true."""
    setup_workspace_root(tmp_path, project_root)

    # Create a real project to adopt
    target = tmp_path / "adopt-source"
    target.mkdir()
    (target / "package.json").write_text('{"name":"test-project"}')

    result = run_script(
        tmp_path / "scripts/workspace.py", "adopt", "my-adopted", str(target),
        cwd=tmp_path, input_text="\n",
    )
    ws = tmp_path / "workspaces" / "my-adopted"

    assert result.returncode == 0
    assert ws.is_symlink(), "adopt: symlink not created"
    assert (ws / ".kodehold-state").is_file(), "adopt: .kodehold-state not created"
    state_content = (ws / ".kodehold-state").read_text()
    assert "ADOPTED=true" in state_content, "ADOPTED=true missing from state"
    assert "JavaScript" in result.stdout or "TypeScript" in result.stdout, \
        "Language detection failed"


# ===================================================================
# Test 5: adopt validates target exists
# ===================================================================

def test_adopt_validates_target_exists(project_root: Path,
                                        tmp_path: Path) -> None:
    """workspace.py adopt must reject non-existent target paths."""
    setup_workspace_root(tmp_path, project_root)

    result = run_script(
        tmp_path / "scripts/workspace.py", "adopt", "nonexistent",
        "/path/does/not/exist", cwd=tmp_path,
    )
    assert result.returncode != 0
    assert "Target path does not exist" in result.stdout


# ===================================================================
# Test 6: state shows STATE=INIT
# ===================================================================

def test_state_shows_init(project_root: Path, tmp_path: Path) -> None:
    """workspace.py state must show STATE=INIT after init."""
    setup_workspace_root(tmp_path, project_root)

    run_script(
        tmp_path / "scripts/workspace.py", "init", "state-project",
        cwd=tmp_path, input_text="\n",
    )
    result = run_script(
        tmp_path / "scripts/workspace.py", "state", "state-project",
        cwd=tmp_path,
    )
    assert "STATE=INIT" in result.stdout


# ===================================================================
# Test 7: deploy-ready checks CLOSED
# ===================================================================

def test_deploy_ready_checks_closed(project_root: Path, tmp_path: Path) -> None:
    """workspace.py deploy-ready must exit 1 for non-CLOSED state."""
    setup_workspace_root(tmp_path, project_root)

    run_script(
        tmp_path / "scripts/workspace.py", "init", "deploy-project",
        cwd=tmp_path, input_text="\n",
    )
    result = run_script(
        tmp_path / "scripts/workspace.py", "deploy-ready", "deploy-project",
        cwd=tmp_path,
    )
    assert result.returncode != 0
    assert "must reach CLOSED before deploy" in result.stdout


# ===================================================================
# Test 8: ensure-git backfills after .git removal
# ===================================================================

def test_ensure_git_backfills(project_root: Path, tmp_path: Path) -> None:
    """workspace.py ensure-git must recreate .git if missing."""
    setup_workspace_root(tmp_path, project_root)

    run_script(
        tmp_path / "scripts/workspace.py", "init", "ensure-test",
        cwd=tmp_path, input_text="\n",
    )

    # Remove .git to simulate backfill scenario
    shutil.rmtree(tmp_path / "workspaces/ensure-test/.git")

    result = run_script(
        tmp_path / "scripts/workspace.py", "ensure-git", "ensure-test",
        cwd=tmp_path, input_text="\n",
    )
    assert result.returncode == 0
    assert (tmp_path / "workspaces/ensure-test/.git").is_dir(), \
        ".git not recreated by ensure-git"
    assert "Git repository initialized" in result.stdout
