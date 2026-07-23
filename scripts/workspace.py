#!/usr/bin/env python3
"""KodeHold Workspace Manager — create, list, and manage project workspaces.

Usage:
    # Workspace management
    python3 scripts/workspace.py init <name>
    python3 scripts/workspace.py adopt <name> <path>
    python3 scripts/workspace.py list
    python3 scripts/workspace.py state <name>
    python3 scripts/workspace.py gate <name> <transition>
    python3 scripts/workspace.py deploy-ready <name>
    python3 scripts/workspace.py ensure-git <name>

    # Loop management (ADR-0060)
    python3 scripts/workspace.py loop <name> list
    python3 scripts/workspace.py loop <name> enable <pattern>
    python3 scripts/workspace.py loop <name> disable <pattern>
    python3 scripts/workspace.py loop <name> run <pattern>

    # Cron management
    python3 scripts/workspace.py cron install
    python3 scripts/workspace.py cron remove
    python3 scripts/workspace.py cron list

    # Monitoring
    python3 scripts/workspace.py audit <name>
    python3 scripts/workspace.py cost <name> <pattern>
    python3 scripts/workspace.py sync <name>

Supported patterns:
    daily-triage, pr-babysitter, ci-sweeper, dependency-sweeper,
    changelog-drafter, post-merge-cleanup, issue-triage
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import date
from pathlib import Path

# Ensure project root is in sys.path
_PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(_PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(_PROJECT_ROOT))

from scripts.lib.output import (  # noqa: E402
    pass_msg,
    fail_msg,
    warn,
    info,
)

WORKSPACE_ROOT = "workspaces"
CATALOG = f"{WORKSPACE_ROOT}/.catalog"
GATE_SCRIPT = "scripts/gate.py"


# ── Loop Engineering Integration ──────────────────────────────────────────


def run_loop_init(ws_dir: str, pattern: str = "daily-triage") -> bool:
    """Run loop-init in workspace to scaffold loop-engineering files.
    
    Returns True on success, False on failure.
    """
    try:
        result = subprocess.run(
            [
                "npx",
                "@cobusgreyling/loop-init",
                ".",
                "--pattern", pattern,
                "--tool", "opencode",
            ],
            cwd=ws_dir,
            capture_output=True,
            text=True,
            timeout=60,
        )
        if result.returncode == 0:
            info(f"Loop-engineering files scaffolded (pattern: {pattern})")
            return True
        else:
            warn(f"loop-init failed: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        warn("loop-init timed out (60s) — skipping loop setup")
        return False
    except FileNotFoundError:
        warn("npx not found — skipping loop setup (install Node.js)")
        return False
    except OSError as e:
        warn(f"loop-init error: {e}")
        return False


def ws_loop_list(name: str) -> None:
    """List active loops for a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    # Check for loop-engineering files
    loop_md = ws_dir / "LOOP.md"
    state_md = ws_dir / "STATE.md"
    
    if not loop_md.is_file():
        info(f"No LOOP.md found in '{name}' — run 'workspace.py init' or 'workspace.py adopt' to scaffold loop files")
        return
    
    print(f"\n━━━ Loops for {name} ━━━\n")
    
    # Parse LOOP.md for active loops
    content = loop_md.read_text()
    in_table = False
    for line in content.splitlines():
        if "| Pattern" in line and "Cadence" in line:
            in_table = True
            print(f"  {line}")
            continue
        if in_table and line.startswith("|"):
            print(f"  {line}")
        elif in_table and not line.startswith("|"):
            in_table = False
    
    if state_md.is_file():
        state_content = state_md.read_text()
        last_run = "never"
        for line in state_content.splitlines():
            if line.startswith("Last run:"):
                last_run = line.split(":", 1)[1].strip()
                break
        print(f"\n  Last run: {last_run}")
    
    print()


def ws_loop_enable(name: str, pattern: str) -> None:
    """Enable a loop pattern for a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    loop_md = ws_dir / "LOOP.md"
    if not loop_md.is_file():
        fail_msg(f"No LOOP.md found in '{name}' — run 'workspace.py init' or 'workspace.py adopt' first")
        sys.exit(1)

    # Valid patterns from loop-engineering
    valid_patterns = [
        "daily-triage",
        "pr-babysitter",
        "ci-sweeper",
        "dependency-sweeper",
        "changelog-drafter",
        "post-merge-cleanup",
        "issue-triage",
    ]

    if pattern not in valid_patterns:
        fail_msg(f"Invalid pattern '{pattern}' — valid patterns: {', '.join(valid_patterns)}")
        sys.exit(1)

    content = loop_md.read_text()
    
    # Check if pattern already enabled
    if f"| {pattern.replace('-', ' ').title()}" in content or f"| {pattern}" in content:
        info(f"Pattern '{pattern}' already enabled for '{name}'")
        return

    # Add pattern to the table
    # Find the table and add the pattern
    lines = content.splitlines()
    new_lines = []
    table_found = False
    pattern_added = False
    
    for line in lines:
        new_lines.append(line)
        if "| Pattern" in line and "Cadence" in line:
            table_found = True
        elif table_found and line.startswith("|") and not pattern_added:
            # Check if this is the last row in the table (empty or placeholder)
            if "_(none configured)_" in line:
                # Replace the placeholder row
                new_lines[-1] = f"| {pattern.replace('-', ' ').title()} | 1d | L1 report-only | `opencode run \"Run {pattern}\" --agent {pattern}` via cron/systemd |"
                pattern_added = True

    if not pattern_added:
        # Find the end of the table and add there
        for i, line in enumerate(lines):
            if "| Pattern" in line and "Cadence" in line:
                # Find the next empty line after the table
                for j in range(i + 1, len(lines)):
                    if not lines[j].startswith("|"):
                        # Insert before this line
                        new_lines.insert(j, f"| {pattern.replace('-', ' ').title()} | 1d | L1 report-only | `opencode run \"Run {pattern}\" --agent {pattern}` via cron/systemd |")
                        pattern_added = True
                        break
                break

    if pattern_added:
        loop_md.write_text("\n".join(new_lines))
        pass_msg(f"Enabled pattern '{pattern}' for '{name}'")
    else:
        warn(f"Could not add pattern '{pattern}' to LOOP.md — manual edit required")


def ws_loop_disable(name: str, pattern: str) -> None:
    """Disable a loop pattern for a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    loop_md = ws_dir / "LOOP.md"
    if not loop_md.is_file():
        fail_msg(f"No LOOP.md found in '{name}'")
        sys.exit(1)

    content = loop_md.read_text()
    lines = content.splitlines()
    new_lines = []
    removed = False
    
    pattern_title = pattern.replace('-', ' ').title()
    
    for line in lines:
        # Skip the line containing the pattern
        if f"| {pattern_title}" in line or f"| {pattern}" in line:
            removed = True
            continue
        new_lines.append(line)

    if removed:
        # Check if table is now empty, add placeholder if needed
        in_table = False
        has_rows = False
        for line in new_lines:
            if "| Pattern" in line and "Cadence" in line:
                in_table = True
            elif in_table and line.startswith("|"):
                has_rows = True
            elif in_table and not line.startswith("|"):
                break
        
        if in_table and not has_rows:
            # Add placeholder row
            for i, line in enumerate(new_lines):
                if "| Cadence" in line and "Status" in line:
                    new_lines.insert(i + 1, "| _(none configured)_ | — | L1 report-only | — |")
                    break
        
        loop_md.write_text("\n".join(new_lines))
        pass_msg(f"Disabled pattern '{pattern}' for '{name}'")
    else:
        info(f"Pattern '{pattern}' not found in '{name}'")


def ws_loop_run(name: str, pattern: str) -> None:
    """Run a loop pattern manually for a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    loop_md = ws_dir / "LOOP.md"
    if not loop_md.is_file():
        fail_msg(f"No LOOP.md found in '{name}' — run 'workspace.py init' or 'workspace.py adopt' first")
        sys.exit(1)

    # Valid patterns
    valid_patterns = [
        "daily-triage",
        "pr-babysitter",
        "ci-sweeper",
        "dependency-sweeper",
        "changelog-drafter",
        "post-merge-cleanup",
        "issue-triage",
    ]

    if pattern not in valid_patterns:
        fail_msg(f"Invalid pattern '{pattern}' — valid patterns: {', '.join(valid_patterns)}")
        sys.exit(1)

    # Map pattern to agent name
    agent_map = {
        "daily-triage": "loop-triage",
        "pr-babysitter": "pr-babysitter",
        "ci-sweeper": "ci-triage",
        "dependency-sweeper": "dependency-triage",
        "changelog-drafter": "changelog-scan",
        "post-merge-cleanup": "post-merge-scan",
        "issue-triage": "issue-triage",
    }

    agent = agent_map.get(pattern, "loop-triage")

    # Build the opencode run command
    prompts = {
        "daily-triage": "Run loop-triage. Read STATE.md first. Update High Priority and Watch List. No auto-fix in week one.",
        "pr-babysitter": "Run pr-babysitter. Read pr-babysitter-state.md. Watch open PRs. No code changes.",
        "ci-sweeper": "Run ci-triage. Read ci-sweeper-state.md. Classify CI failures. No auto-fix.",
        "dependency-sweeper": "Run dependency-triage. Read dependency-sweeper-state.md. Scan for outdated deps. No auto-fix.",
        "changelog-drafter": "Run changelog-scan. Read changelog-drafter-state.md. Scan merges. Draft release notes.",
        "post-merge-cleanup": "Run post-merge-scan. Read post-merge-state.md. Scan recent merges. Propose cleanup.",
        "issue-triage": "Run issue-triage. Read issue-triage-state.md. Scan open issues. Propose labels.",
    }

    prompt = prompts.get(pattern, f"Run {pattern}")

    info(f"Running {pattern} for workspace '{name}'...")
    info(f"Directory: {ws_dir}")
    info(f"Agent: {agent}")
    info(f"Prompt: {prompt}")

    # Run opencode
    try:
        result = subprocess.run(
            [
                "opencode",
                "run",
                prompt,
                "--agent", agent,
            ],
            cwd=str(ws_dir),
            capture_output=False,
            timeout=300,
        )
        if result.returncode == 0:
            pass_msg(f"Loop '{pattern}' completed for '{name}'")
        else:
            fail_msg(f"Loop '{pattern}' exited with code {result.returncode}")
            sys.exit(1)
    except subprocess.TimeoutExpired:
        fail_msg(f"Loop '{pattern}' timed out (300s)")
        sys.exit(1)
    except FileNotFoundError:
        fail_msg("opencode not found — install opencode CLI")
        sys.exit(1)
    except OSError as e:
        fail_msg(f"Error running loop: {e}")
        sys.exit(1)


# ── Cron Management ──────────────────────────────────────────────────────


def _get_crontab_path() -> Path:
    """Get path to kodehold crontab file."""
    return _PROJECT_ROOT / "config" / "crontab.kodehold"


def _read_crontab() -> str:
    """Read the kodehold crontab file."""
    crontab_path = _get_crontab_path()
    if crontab_path.is_file():
        return crontab_path.read_text()
    return ""


def _write_crontab(content: str) -> None:
    """Write the kodehold crontab file."""
    crontab_path = _get_crontab_path()
    crontab_path.parent.mkdir(parents=True, exist_ok=True)
    crontab_path.write_text(content)


def _generate_cron_entry(ws_name: str, pattern: str, schedule: str) -> str:
    """Generate a crontab entry for a workspace loop."""
    ws_dir = (_PROJECT_ROOT / WORKSPACE_ROOT / ws_name).resolve()
    
    # Map pattern to agent and prompt
    agent_map = {
        "daily-triage": ("loop-triage", "Run loop-triage. Read STATE.md first. Update High Priority and Watch List. No auto-fix in week one."),
        "pr-babysitter": ("pr-babysitter", "Run pr-babysitter. Read pr-babysitter-state.md. Watch open PRs. No code changes."),
        "ci-sweeper": ("ci-triage", "Run ci-triage. Read ci-sweeper-state.md. Classify CI failures. No auto-fix."),
        "dependency-sweeper": ("dependency-triage", "Run dependency-triage. Read dependency-sweeper-state.md. Scan for outdated deps. No auto-fix."),
        "changelog-drafter": ("changelog-scan", "Run changelog-scan. Read changelog-drafter-state.md. Scan merges. Draft release notes."),
        "post-merge-cleanup": ("post-merge-scan", "Run post-merge-scan. Read post-merge-state.md. Scan recent merges. Propose cleanup."),
        "issue-triage": ("issue-triage", "Run issue-triage. Read issue-triage-state.md. Scan open issues. Propose labels."),
    }
    
    agent, prompt = agent_map.get(pattern, ("loop-triage", f"Run {pattern}"))
    
    # Path to discord-notify.py
    discord_script = (_PROJECT_ROOT / "scripts" / "discord-notify.py").resolve()
    
    # Build the command with Discord notification and findings extraction
    cmd = f"""# {pattern} for {ws_name} — {schedule}
{schedule} cd {ws_dir} && START_TIME=$(date +%s) && opencode run "{prompt}" --agent {agent} 2>&1 | tee /tmp/loop-{ws_name}.log && END_TIME=$(date +%s) && DURATION=$((END_TIME - START_TIME)) && python3 {discord_script} {ws_name} {pattern} "Loop completed" $DURATION /tmp/loop-{ws_name}.log"""
    
    return cmd


def ws_cron_install() -> None:
    """Install crontab entries for all workspaces with enabled loops."""
    catalog = ensure_catalog()
    
    if not catalog:
        info("No workspaces found")
        return
    
    cron_entries = []
    cron_entries.append("# ── KodeHold Loop Engineering — Auto-generated crontab ──")
    cron_entries.append("# Generated by: workspace.py cron install")
    cron_entries.append("# Manual edits will be overwritten on next install")
    cron_entries.append("")
    
    installed_count = 0
    
    for ws_name, meta in sorted(catalog.items()):
        ws_dir = Path(WORKSPACE_ROOT) / ws_name
        loop_md = ws_dir / "LOOP.md"
        
        if not loop_md.is_file():
            continue
        
        # Parse LOOP.md for enabled patterns
        content = loop_md.read_text()
        patterns = []
        
        for line in content.splitlines():
            if line.startswith("|") and "L1" in line:
                # Extract pattern name from the table row
                parts = [p.strip() for p in line.split("|") if p.strip()]
                if parts:
                    pattern_name = parts[0].lower().replace(" ", "-")
                    # Skip placeholder rows
                    if "none" not in pattern_name and "configured" not in pattern_name:
                        patterns.append(pattern_name)
        
        if not patterns:
            continue
        
        cron_entries.append(f"# ── {ws_name} ──")
        
        for i, pattern in enumerate(patterns):
            # Stagger schedules to avoid conflicts
            if pattern == "daily-triage":
                schedule = f"{8 + (installed_count % 4)} * * * *"
            elif pattern == "pr-babysitter":
                schedule = f"0 8,12,16 * * 1-5"
            elif pattern == "issue-triage":
                schedule = f"0 */2 * * *"
            else:
                schedule = f"0 {8 + (installed_count % 4)} * * *"
            
            cron_entries.append(_generate_cron_entry(ws_name, pattern, schedule))
            installed_count += 1
        
        cron_entries.append("")
    
    if installed_count == 0:
        info("No loops enabled in any workspace")
        return
    
    # Write crontab
    _write_crontab("\n".join(cron_entries))
    pass_msg(f"Installed {installed_count} crontab entries for {len(catalog)} workspaces")
    info(f"Crontab file: {_get_crontab_path()}")
    info("To activate: crontab < {_get_crontab_path()}")


def ws_cron_remove() -> None:
    """Remove kodehold crontab entries."""
    crontab_path = _get_crontab_path()
    if crontab_path.is_file():
        crontab_path.unlink()
        pass_msg("Removed kodehold crontab file")
    else:
        info("No kodehold crontab file found")


def ws_cron_list() -> None:
    """Show current crontab entries."""
    crontab_path = _get_crontab_path()
    if crontab_path.is_file():
        print(f"\n━━━ KodeHold Crontab ━━━\n")
        print(crontab_path.read_text())
        print(f"\nFile: {crontab_path}")
    else:
        info("No kodehold crontab file found")
        info("Run 'workspace.py cron install' to generate")


# ── Monitoring Commands ──────────────────────────────────────────────────


def ws_audit(name: str) -> None:
    """Run loop-audit in a workspace to check loop readiness."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    loop_md = ws_dir / "LOOP.md"
    if not loop_md.is_file():
        fail_msg(f"No LOOP.md found in '{name}' — run 'workspace.py init' or 'workspace.py adopt' first")
        sys.exit(1)

    info(f"Running loop-audit for workspace '{name}'...")
    info(f"Directory: {ws_dir}")

    try:
        result = subprocess.run(
            [
                "npx",
                "@cobusgreyling/loop-audit",
                ".",
                "--suggest",
            ],
            cwd=str(ws_dir),
            capture_output=False,
            timeout=120,
        )
        if result.returncode == 0:
            pass_msg(f"Loop-audit completed for '{name}'")
        else:
            warn(f"Loop-audit exited with code {result.returncode}")
    except subprocess.TimeoutExpired:
        fail_msg("loop-audit timed out (120s)")
        sys.exit(1)
    except FileNotFoundError:
        fail_msg("npx not found — install Node.js")
        sys.exit(1)
    except OSError as e:
        fail_msg(f"Error running loop-audit: {e}")
        sys.exit(1)


def ws_cost(name: str, pattern: str) -> None:
    """Estimate token cost for a loop pattern in a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    # Valid patterns
    valid_patterns = [
        "daily-triage",
        "pr-babysitter",
        "ci-sweeper",
        "dependency-sweeper",
        "changelog-drafter",
        "post-merge-cleanup",
        "issue-triage",
    ]

    if pattern not in valid_patterns:
        fail_msg(f"Invalid pattern '{pattern}' — valid patterns: {', '.join(valid_patterns)}")
        sys.exit(1)

    info(f"Estimating token cost for pattern '{pattern}' in workspace '{name}'...")

    try:
        result = subprocess.run(
            [
                "npx",
                "@cobusgreyling/loop-cost",
                "--pattern", pattern,
                "--level", "L1",
            ],
            cwd=str(ws_dir),
            capture_output=False,
            timeout=60,
        )
        if result.returncode == 0:
            pass_msg(f"Cost estimation completed for '{pattern}'")
        else:
            warn(f"Loop-cost exited with code {result.returncode}")
    except subprocess.TimeoutExpired:
        fail_msg("loop-cost timed out (60s)")
        sys.exit(1)
    except FileNotFoundError:
        fail_msg("npx not found — install Node.js")
        sys.exit(1)
    except OSError as e:
        fail_msg(f"Error running loop-cost: {e}")
        sys.exit(1)


def ws_sync(name: str) -> None:
    """Check drift between STATE.md and LOOP.md in a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    loop_md = ws_dir / "LOOP.md"
    state_md = ws_dir / "STATE.md"
    
    if not loop_md.is_file():
        fail_msg(f"No LOOP.md found in '{name}' — run 'workspace.py init' or 'workspace.py adopt' first")
        sys.exit(1)

    info(f"Running loop-sync for workspace '{name}'...")
    info(f"Directory: {ws_dir}")

    try:
        result = subprocess.run(
            [
                "npx",
                "@cobusgreyling/loop-sync",
                ".",
            ],
            cwd=str(ws_dir),
            capture_output=False,
            timeout=60,
        )
        if result.returncode == 0:
            pass_msg(f"Loop-sync completed for '{name}'")
        else:
            warn(f"Loop-sync exited with code {result.returncode}")
    except subprocess.TimeoutExpired:
        fail_msg("loop-sync timed out (60s)")
        sys.exit(1)
    except FileNotFoundError:
        fail_msg("npx not found — install Node.js")
        sys.exit(1)
    except OSError as e:
        fail_msg(f"Error running loop-sync: {e}")
        sys.exit(1)


def validate_slug(slug: str) -> bool:
    """Validate a slug per ADR-0036 format: /^[a-z][a-z0-9-]{0,49}$/"""
    return bool(re.match(r"^[a-z][a-z0-9-]{0,49}$", slug))


def ensure_catalog() -> dict:
    """Ensure the catalog file exists and return its contents."""
    cat_path = Path(CATALOG)
    if not cat_path.is_file():
        cat_path.parent.mkdir(parents=True, exist_ok=True)
        cat_path.write_text("{}")
        return {}
    try:
        return json.loads(cat_path.read_text())
    except (json.JSONDecodeError, OSError):
        return {}


def catalog_write(data: dict) -> None:
    """Write catalog data."""
    Path(CATALOG).write_text(json.dumps(data, indent=2))


def prompt_remote(ws_dir: str, label: str = "Workspace") -> None:
    """Prompt user to add a git remote."""
    remote_url = input(f"  i Add a git remote for {label}? (URL or empty to skip): ").strip()
    if remote_url:
        result = subprocess.run(
            ["git", "-C", ws_dir, "remote", "add", "origin", remote_url],
            capture_output=True,
        )
        if result.returncode == 0:
            pass_msg(f"Remote 'origin' added: {remote_url}")
        else:
            warn("Failed to add remote — check the URL")
    else:
        info("Skipped remote setup")


def git_init(ws_dir: str, commit_msg: str = "Initial commit") -> bool:
    """Initialize a git repository in ws_dir and make initial commit."""
    try:
        if subprocess.run(["git", "init", ws_dir], capture_output=True).returncode != 0:
            return False
        subprocess.run(
            ["git", "-C", ws_dir, "add", "-A"], capture_output=True
        )
        subprocess.run(
            ["git", "-C", ws_dir, "commit", "-m", commit_msg],
            capture_output=True,
        )
        return True
    except OSError:
        return False


def detect_project_lang(ws_dir: str) -> tuple[str, str, str, str]:
    """Detect language, test framework, and build system for a project."""
    lang = "Unknown"
    test_framework = ""
    build_system = ""
    commit_count = 0

    # Count commits
    if (Path(ws_dir) / ".git").is_dir():
        try:
            result = subprocess.run(
                ["git", "-C", ws_dir, "log", "--oneline"],
                capture_output=True,
                text=True,
            )
            commit_count = len(result.stdout.strip().splitlines()) if result.stdout.strip() else 0
        except OSError:
            pass

    pkg_json = Path(ws_dir) / "package.json"
    cargo_toml = Path(ws_dir) / "Cargo.toml"
    pyproject = Path(ws_dir) / "pyproject.toml"
    go_mod = Path(ws_dir) / "go.mod"
    makefile = Path(ws_dir) / "Makefile"

    if pkg_json.is_file():
        lang = "JavaScript/TypeScript"
        text = pkg_json.read_text()
        for fw in ("jest", "vitest", "mocha", "playwright"):
            if fw in text:
                test_framework = fw
                break
        m = re.search(r'"build":\s*"[^"]+"', text)
        if m:
            build_system = m.group(0)
        return lang, test_framework, build_system, commit_count

    if cargo_toml.is_file():
        return "Rust", "cargo test (built-in)", "cargo", commit_count

    if pyproject.is_file():
        lang = "Python"
        text = pyproject.read_text()
        if "pytest" in text:
            test_framework = "pytest"
        elif "unittest" in text:
            test_framework = "unittest"
        else:
            test_framework = "pytest"
        build_system = "pip/setuptools"
        return lang, test_framework, build_system, commit_count

    if go_mod.is_file():
        return "Go", "go test (built-in)", "go build", commit_count

    if makefile.is_file():
        return "Unknown (has Makefile)", "", "make", commit_count

    return lang, test_framework, build_system, commit_count


# ── Workspace commands ────────────────────────────────────────────────────


def ws_init(name: str) -> None:
    """Create a new project workspace from scratch."""
    ws_dir = Path(WORKSPACE_ROOT) / name

    if not validate_slug(name):
        fail_msg(
            f"Invalid workspace name '{name}' — must match slug pattern: ^[a-z][a-z0-9-]{{0,49}}$"
        )
        sys.exit(1)

    if ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' already exists at {ws_dir}")
        sys.exit(1)

    info(f"Creating workspace: {name}")

    # Create directory structure
    for sub in ("docs/design", "docs/adr", "docs/decisions", "src", "tests"):
        (ws_dir / sub).mkdir(parents=True, exist_ok=True)

    today = date.today().isoformat()

    # Design doc template
    design_doc = f"""# {name} — Design Document
**Version:** 0.1
**Status:** Draft
**Design Authority:** Architects
**Last Reviewed:** {today}

## 1. Purpose & Scope
## 2. Requirements
## 3. Architecture Overview
## 4. Component Design
## 5. Data Model
## 6. API Design
## 7. Implementation Plan
## 8. Testing Strategy
## 9. ADR Index
## 10. Open Questions
## 11. Changelog
"""
    (ws_dir / "docs/design" / "README.md").write_text(design_doc)

    # ADR index
    adr_index = f"""# ADR Index — {name}

| ADR | Title | Status |
|-----|-------|--------|
"""
    (ws_dir / "docs/adr" / "README.md").write_text(adr_index)

    # State file
    state = f"""# KodeHold Lifecycle State
# Valid states: INIT, ACTIVE, REVIEW, CLOSED, REOPEN
STATE=INIT
LAST_UPDATED={today}
DESIGN_DOC_APPROVED=false
ADRS_COMPLETE=false
TESTS_PASSING=false
CODE_REVIEWED=false
"""
    (ws_dir / ".kodehold-state").write_text(state)

    # .gitignore
    (ws_dir / ".gitignore").write_text("*.pyc\n__pycache__/\n.venv/\n")

    # Git init
    if git_init(str(ws_dir), "Initial commit"):
        pass_msg(f"Git repository initialized at {ws_dir}")
    else:
        warn("git not found — workspace will not have version control")
        prompt_remote(str(ws_dir), name)
        return

    prompt_remote(str(ws_dir), name)

    # Scaffold loop-engineering files
    run_loop_init(str(ws_dir))

    # Register in catalog
    catalog = ensure_catalog()
    catalog[name] = {"created": today, "project": name}
    catalog_write(catalog)

    pass_msg(f"Workspace '{name}' created at {ws_dir}")


def ws_adopt(name: str, target_path: str) -> None:
    """Adopt an existing project into a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name

    if not validate_slug(name):
        fail_msg(
            f"Invalid workspace name '{name}' — must match slug pattern: ^[a-z][a-z0-9-]{{0,49}}$"
        )
        sys.exit(1)

    if ws_dir.exists():
        fail_msg(f"Workspace '{name}' already exists at {ws_dir}")
        sys.exit(1)

    target = Path(target_path)
    if not target.is_dir():
        fail_msg(f"Target path does not exist: {target}")
        sys.exit(1)

    target_resolved = target.resolve()

    info(f"Adopting existing project: {target_resolved} → {ws_dir}")

    # Create symlink
    ws_dir.parent.mkdir(parents=True, exist_ok=True)
    ws_dir.symlink_to(target_resolved)

    # Create KodeHold artifacts inside the adopted project
    (ws_dir / "docs/design").mkdir(parents=True, exist_ok=True)
    (ws_dir / "docs/adr").mkdir(parents=True, exist_ok=True)

    today = date.today().isoformat()

    # State file with ADOPTED flag
    state = f"""# KodeHold Lifecycle State — Adopted Project
# Valid states: INIT, ACTIVE, REVIEW, CLOSED, REOPEN
STATE=INIT
ADOPTED=true
LAST_UPDATED={today}
DESIGN_DOC_APPROVED=false
ADRS_COMPLETE=false
TESTS_PASSING=false
CODE_REVIEWED=false
"""
    (ws_dir / ".kodehold-state").write_text(state)

    # Detect project info
    lang, test_framework, build_system, commit_count = detect_project_lang(str(ws_dir))

    # Design doc template
    design_doc = f"""# {name} — Design Document
**Version:** 0.1
**Status:** Draft
**Design Authority:** Architects
**Last Reviewed:** {today}
**Origin:** Adopted ({target_resolved})

> Project adopted by KodeHold on {today}. It was not originally created with KodeHold.
> This design document is a retroactive description of the existing codebase.

## 1. Purpose & Scope
_Describe what this project does — derived from existing code._

## 2. Requirements
_Reverse-engineered from existing functionality._

## 3. Architecture Overview
_Describe the existing architecture._

- Language: {lang}
- Build system: {build_system}
- Test framework: {test_framework or "None detected"}
- {commit_count} git commits

## 4. Component Design
_Catalogue existing components and modules._

## 5. Data Model
_Document existing data structures and schemas._

## 6. API Design
_Document existing API endpoints and interfaces._

## 7. Implementation Plan
_No forward plan — this project is already implemented. Use for feature additions._

## 8. Testing Strategy
_Describe existing test approach._

- Test framework: {test_framework or "Not detected"}
- _Add test discovery results here_

## 9. ADR Index
_Record architectural decisions retroactively as ADRs._

## 10. Open Questions
_What needs to be understood about the codebase._

## 11. Changelog
- {today}: Adopted by KodeHold — design doc created retroactively
"""
    (ws_dir / "docs/design" / "README.md").write_text(design_doc)

    # ADR index
    adr_index = f"""# ADR Index — {name} (Adopted Project)

| ADR | Title | Status |
|-----|-------|--------|
"""
    (ws_dir / "docs/adr" / "README.md").write_text(adr_index)

    info(f"Project scan complete: {lang}, {commit_count} commits")

    # Ensure git
    if not (ws_dir / ".git").is_dir():
        if git_init(str(ws_dir), "Initial commit — adopted by KodeHold"):
            pass_msg(f"Git repository initialized at {ws_dir}")
            prompt_remote(str(ws_dir), name)
        else:
            warn("git init failed — adopted project will not have version control")
    else:
        # Still prompt for remote if none exists
        result = subprocess.run(
            ["git", "-C", str(ws_dir), "remote", "-v"],
            capture_output=True,
            text=True,
        )
        if not result.stdout.strip():
            prompt_remote(str(ws_dir), name)

    # Scaffold loop-engineering files
    run_loop_init(str(ws_dir))

    # Register in catalog
    catalog = ensure_catalog()
    catalog[name] = {
        "created": today,
        "project": name,
        "origin": "adopted",
        "real_path": str(target_resolved),
    }
    catalog_write(catalog)

    pass_msg(f"Adopted '{name}' from {target_resolved}")
    info("Next steps:")
    info("  1. Read design doc at {}docs/design/README.md".format(str(ws_dir)))
    info("  2. Fill in Purpose, Architecture, Components sections")
    info("  3. Run: python3 scripts/workspace.py gate {} INIT_TO_ACTIVE".format(name))
    info("  Note: Adopted projects get relaxed gates — design doc fill-in is the priority")


def ws_list() -> None:
    """List all workspaces."""
    catalog = ensure_catalog()
    count = len(catalog)

    print()
    print(f"━━━ Workspaces ({count}) ━━━")
    print()

    if count == 0:
        info("No workspaces yet. Create one with: python3 scripts/workspace.py init <name>")
        print()
        return

    print(f"  {'NAME':<20} {'STATE':<12} {'UPDATED':<14} PATH")
    print(f"  {'─' * 75}")

    for name, meta in sorted(catalog.items()):
        created = meta.get("created", "N/A")
        ws_dir = Path(WORKSPACE_ROOT) / name

        # Read state from .kodehold-state
        state = "N/A"
        state_file = ws_dir / ".kodehold-state"
        if state_file.is_file():
            for line in state_file.read_text().splitlines():
                if line.startswith("STATE="):
                    state = line.split("=", 1)[1]
                    break

        if ws_dir.is_dir():
            print(f"  {name:<20} {state:<12} {created:<14} {ws_dir}")
        else:
            print(f"  {name:<20} {state:<12} {created:<14} MISSING")

    print()


def ws_state(name: str) -> None:
    """Show lifecycle state of a workspace."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    state_file = ws_dir / ".kodehold-state"
    if state_file.is_file():
        print(state_file.read_text(), end="")
    else:
        info("No .kodehold-state found — workspace not initialized with lifecycle")


def ws_transition(name: str, transition: str) -> None:
    """Run a gate transition on a workspace."""
    ws_dir = (Path(WORKSPACE_ROOT) / name).resolve()

    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    # Run gate via subprocess
    gate = Path(GATE_SCRIPT).resolve()
    if gate.is_file():
        result = subprocess.run(
            [sys.executable, str(gate), "--project-path", str(ws_dir), "--transition", transition],
        )
        if result.returncode == 0:
            pass_msg(f"Gate {transition} passed for '{name}'")
        else:
            fail_msg(f"Gate {transition} BLOCKED for '{name}' — fix before transition")
            sys.exit(1)

    # Update state file
    next_state_map = {
        "INIT_TO_ACTIVE": "ACTIVE",
        "ACTIVE_TO_REVIEW": "REVIEW",
        "REVIEW_TO_CLOSED": "CLOSED",
        "CLOSED_TO_REOPEN": "REOPEN",
        "REOPEN_TO_ACTIVE": "ACTIVE",
    }
    next_state = next_state_map.get(transition)
    if not next_state:
        fail_msg(f"Unknown transition: {transition}")
        sys.exit(1)

    state_file = ws_dir / ".kodehold-state"
    if state_file.is_file():
        text = state_file.read_text()
        text = re.sub(r"^STATE=.*", f"STATE={next_state}", text, flags=re.MULTILINE)
        text = re.sub(
            r"^LAST_UPDATED=.*",
            f"LAST_UPDATED={date.today().isoformat()}",
            text,
            flags=re.MULTILINE,
        )
        state_file.write_text(text)

    pass_msg(f"Transitioned '{name}' to {next_state}")


def ws_deploy_ready(name: str) -> None:
    """Check if workspace is ready to deploy."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    state_file = ws_dir / ".kodehold-state"
    state = ""
    if state_file.is_file():
        for line in state_file.read_text().splitlines():
            if line.startswith("STATE="):
                state = line.split("=", 1)[1]
                break

    if state == "CLOSED":
        pass_msg(f"'{name}' is CLOSED — ready for deploy")
    else:
        info(f"'{name}' is {state} — must reach CLOSED before deploy")
        info("  Current path: INIT → ACTIVE → REVIEW → CLOSED")
        sys.exit(1)


def ws_ensure_git(name: str) -> None:
    """Initialize git repo for an existing workspace (backfill)."""
    ws_dir = Path(WORKSPACE_ROOT) / name
    if not ws_dir.is_dir():
        fail_msg(f"Workspace '{name}' not found")
        sys.exit(1)

    if (ws_dir / ".git").is_dir():
        pass_msg(f"'{name}' already has a git repository")
        return

    if git_init(str(ws_dir), "Initial commit — backfilled by KodeHold"):
        pass_msg(f"Git repository initialized for '{name}'")
        prompt_remote(str(ws_dir), name)
    else:
        warn("git not available — cannot initialize '{name}'")
        sys.exit(1)


# ── CLI ───────────────────────────────────────────────────────────────────


def _build_parser() -> argparse.ArgumentParser:
    """Build the argument parser with subcommands."""
    parser = argparse.ArgumentParser(
        prog="workspace.py",
        description="KodeHold Workspace Manager — create, list, and manage project workspaces",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Transitions:
  INIT_TO_ACTIVE, ACTIVE_TO_REVIEW, REVIEW_TO_CLOSED,
  CLOSED_TO_REOPEN, REOPEN_TO_ACTIVE""",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    init_p = sub.add_parser("init", help="Create a new project workspace from scratch")
    init_p.add_argument("name", help="Workspace slug (e.g. my-project)")

    adopt_p = sub.add_parser("adopt", help="Adopt an existing project (symlink + KodeHold bootstrap)")
    adopt_p.add_argument("name", help="Workspace slug")
    adopt_p.add_argument("path", help="Path to existing project directory")

    sub.add_parser("list", help="List all workspaces")

    state_p = sub.add_parser("state", help="Show lifecycle state of a workspace")
    state_p.add_argument("name", help="Workspace name")

    gate_p = sub.add_parser("gate", help="Run a gate transition on a workspace")
    gate_p.add_argument("name", help="Workspace name")
    gate_p.add_argument("transition", help="Transition (e.g. INIT_TO_ACTIVE)")

    deploy_p = sub.add_parser("deploy-ready", help="Check if workspace is ready to deploy")
    deploy_p.add_argument("name", help="Workspace name")

    git_p = sub.add_parser("ensure-git", help="Initialize git repo for an existing workspace (backfill)")
    git_p.add_argument("name", help="Workspace name")

    # Loop subcommand group
    loop_p = sub.add_parser("loop", help="Manage loops in a workspace")
    loop_p.add_argument("name", help="Workspace name")
    loop_sub = loop_p.add_subparsers(dest="loop_command", required=True)
    loop_sub.add_parser("list", help="List active loops in a workspace")
    
    enable_p = loop_sub.add_parser("enable", help="Enable a loop pattern")
    enable_p.add_argument("pattern", help="Pattern to enable (e.g. daily-triage)")
    
    disable_p = loop_sub.add_parser("disable", help="Disable a loop pattern")
    disable_p.add_argument("pattern", help="Pattern to disable (e.g. daily-triage)")
    
    run_p = loop_sub.add_parser("run", help="Run a loop pattern manually")
    run_p.add_argument("pattern", help="Pattern to run (e.g. daily-triage)")

    # Cron subcommand group
    cron_p = sub.add_parser("cron", help="Manage crontab entries for workspace loops")
    cron_sub = cron_p.add_subparsers(dest="cron_command", required=True)
    cron_sub.add_parser("install", help="Install crontab entries for all workspaces")
    cron_sub.add_parser("remove", help="Remove kodehold crontab entries")
    cron_sub.add_parser("list", help="Show current crontab entries")

    # Monitoring subcommand group
    audit_p = sub.add_parser("audit", help="Run loop-audit in a workspace")
    audit_p.add_argument("name", help="Workspace name")

    cost_p = sub.add_parser("cost", help="Estimate token cost for a loop pattern")
    cost_p.add_argument("name", help="Workspace name")
    cost_p.add_argument("pattern", help="Pattern to estimate (e.g. daily-triage)")

    sync_p = sub.add_parser("sync", help="Check drift between STATE.md and LOOP.md")
    sync_p.add_argument("name", help="Workspace name")

    return parser


def _dispatch_loop(args: argparse.Namespace) -> None:
    """Dispatch loop subcommands."""
    if args.loop_command == "list":
        ws_loop_list(args.name)
    elif args.loop_command == "enable":
        ws_loop_enable(args.name, args.pattern)
    elif args.loop_command == "disable":
        ws_loop_disable(args.name, args.pattern)
    elif args.loop_command == "run":
        ws_loop_run(args.name, args.pattern)
    else:
        fail_msg(f"Unknown loop command: {args.loop_command}")
        sys.exit(1)


def _dispatch_cron(args: argparse.Namespace) -> None:
    """Dispatch cron subcommands."""
    if args.cron_command == "install":
        ws_cron_install()
    elif args.cron_command == "remove":
        ws_cron_remove()
    elif args.cron_command == "list":
        ws_cron_list()
    else:
        fail_msg(f"Unknown cron command: {args.cron_command}")
        sys.exit(1)


def main() -> None:
    """Main entry point."""
    parser = _build_parser()
    args = parser.parse_args()

    # Change to project root
    os.chdir(str(_PROJECT_ROOT))

    dispatch = {
        "init": lambda: ws_init(args.name),
        "adopt": lambda: ws_adopt(args.name, args.path),
        "list": lambda: ws_list(),
        "state": lambda: ws_state(args.name),
        "gate": lambda: ws_transition(args.name, args.transition),
        "deploy-ready": lambda: ws_deploy_ready(args.name),
        "ensure-git": lambda: ws_ensure_git(args.name),
        "loop": lambda: _dispatch_loop(args),
        "cron": lambda: _dispatch_cron(args),
        "audit": lambda: ws_audit(args.name),
        "cost": lambda: ws_cost(args.name, args.pattern),
        "sync": lambda: ws_sync(args.name),
    }
    dispatch[args.command]()


if __name__ == "__main__":
    main()
