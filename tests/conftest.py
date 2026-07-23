"""Shared fixtures for KodeHold test suite."""

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest


# ---------------------------------------------------------------------------
# Bash / Python discovery
# ---------------------------------------------------------------------------

def _find_bash() -> str:
    """Return the path to a usable bash executable."""
    bash = shutil.which("bash")
    if bash:
        return bash
    candidates = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files\Git\usr\bin\bash.exe",
    ]
    for c in candidates:
        if Path(c).exists():
            return c
    msg = "bash not found — install Git for Windows or ensure bash is on PATH"
    raise RuntimeError(msg)


BASH = _find_bash()
PYTHON = sys.executable


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

@pytest.fixture
def project_root() -> Path:
    """Absolute path to the KodeHold project root (two levels up from tests/)."""
    return Path(__file__).resolve().parent.parent


# ---------------------------------------------------------------------------
# Helpers — used directly by test functions (not fixtures)
# ---------------------------------------------------------------------------

def run_script(script_path: str | Path, *args: str, cwd: str | Path | None = None,
               env: dict | None = None, input_text: str | None = None) -> subprocess.CompletedProcess:
    """Run *script_path* with *args* in *cwd* and return CompletedProcess.

    If *input_text* is provided, it is sent to stdin (useful for interactive prompts).
    Auto-detects whether to use bash or python based on file extension.
    """
    full_env = os.environ.copy()
    if env:
        full_env.update(env)
    # If input_text is given, remove OPENCODE_NONINTERACTIVE so prompts aren't auto-skipped
    if input_text is not None and "OPENCODE_NONINTERACTIVE" in full_env:
        del full_env["OPENCODE_NONINTERACTIVE"]

    path_str = str(script_path)
    if path_str.endswith(".py"):
        cmd = [PYTHON, path_str, *args]
    else:
        cmd = [BASH, path_str, *args]

    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        cwd=str(cwd) if cwd else None,
        env=full_env,
        input=input_text,
    )


def setup_gate_project(dest: Path, project_root: Path) -> None:
    """Create a minimal project structure for gate transition tests in *dest*."""
    for d in ["scripts/lib", "docs/design", "docs/adr", "tests"]:
        (dest / d).mkdir(parents=True, exist_ok=True)

    shutil.copy2(project_root / "scripts/gate.py", dest / "scripts/gate.py")
    shutil.copy2(project_root / "scripts/lib/output.py", dest / "scripts/lib/output.py")
    # Make __init__.py available so the package imports work
    init_py = dest / "scripts/__init__.py"
    if not init_py.is_file():
        init_py.write_text("# KodeHold Scripts Package\n")
    lib_init = dest / "scripts/lib/__init__.py"
    if not lib_init.is_file():
        lib_init.write_text("# KodeHold Scripts Library Package\n")

    # Default passing test runner
    (dest / "tests/run.sh").write_text(
        "#!/usr/bin/env bash\necho \"All tests pass\"\nexit 0\n"
    )
    (dest / "tests/run.sh").chmod(0o755)

    # Design doc with all required sections
    (dest / "docs/design/README.md").write_text(
        "# Design\nStatus: Active\nLast Updated: 2026-06-28\n\n"
        "## Purpose & Scope\n## Requirements\n## Architecture Overview\n"
        "## Component Design\n## Data Model\n## API Design\n"
        "## Implementation Plan\n## Testing Strategy\n## ADR Index\n"
        "## Open Questions\n## Changelog\n"
    )

    # ADR index + sample ADR
    (dest / "docs/adr/README.md").write_text("# ADR Index\n")
    (dest / "docs/adr/ADR-0001-sample.md").write_text("# ADR-0001: Sample\n")


def setup_ship_project(dest: Path, project_root: Path) -> None:
    """Create a minimal project structure for ship.py tests in *dest*."""
    for d in ["tests", "scripts/lib"]:
        (dest / d).mkdir(parents=True, exist_ok=True)

    shutil.copy2(project_root / "scripts/ship.py", dest / "scripts/ship.py")
    shutil.copy2(project_root / "scripts/lib/output.py", dest / "scripts/lib/output.py")

    # Default version
    (dest / "VERSION.md").write_text("| 0.2.0 |\n")

    # Default changelog
    (dest / "CHANGES.md").write_text(
        "# Changelog\n\n## 0.2.0 — 2026-06-28\n### Fixed\n- Initial fixes\n"
    )

    # Default TODO
    (dest / "TODO.md").write_text("# TODOs\n")

    # Default passing test runner
    (dest / "tests/run.sh").write_text(
        "#!/usr/bin/env bash\necho \"All tests pass\"\nexit 0\n"
    )
    (dest / "tests/run.sh").chmod(0o755)


def setup_workspace_root(dest: Path, project_root: Path) -> None:
    """Create a minimal workspace root (scripts/ + workspaces/)."""
    for d in ["scripts/lib", "workspaces"]:
        (dest / d).mkdir(parents=True, exist_ok=True)

    shutil.copy2(project_root / "scripts/workspace.py", dest / "scripts/workspace.py")
    shutil.copy2(project_root / "scripts/lib/output.py", dest / "scripts/lib/output.py")
    shutil.copy2(project_root / "scripts/gate.py", dest / "scripts/gate.py")
