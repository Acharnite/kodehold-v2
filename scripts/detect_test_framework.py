#!/usr/bin/env python3
"""Detect test framework for a project.

Usage:
    python3 scripts/detect_test_framework.py [<project-root>]
    python3 scripts/detect_test_framework.py /path/to/project

Output: <framework>:<command> on stdout
Exit: 0 if detected, 1 if none found
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def detect_framework(root: str = ".") -> tuple[str, str] | None:
    """Detect test framework in the given project root.

    Returns (framework_name, command) tuple or None if not detected.
    """
    root_path = Path(root).resolve()

    # Priority 1: Cargo.toml
    if (root_path / "Cargo.toml").is_file():
        return "cargo", "cargo test"

    # Priority 2: package.json
    pkg_json = root_path / "package.json"
    if pkg_json.is_file():
        text = pkg_json.read_text()
        if '"jest"' in text:
            return "jest", "npx jest"
        if '"vitest"' in text:
            return "vitest", "npx vitest run"
        # Check for test script in scripts section
        m = re.search(r'"scripts"\s*:\s*\{[^}]*"test"\s*:', text)
        if m:
            return "npm", "npm test"

    # Priority 3: Gemfile with rspec
    gemfile = root_path / "Gemfile"
    if gemfile.is_file() and "rspec" in gemfile.read_text():
        return "rspec", "bundle exec rspec"

    # Priority 4: mix.exs (Elixir)
    if (root_path / "mix.exs").is_file():
        return "elixir", "mix test"

    # Priority 5: Gradle
    if (root_path / "build.gradle").is_file() or (root_path / "build.gradle.kts").is_file():
        return "gradle", "gradle test"

    # Priority 6: Maven
    if (root_path / "pom.xml").is_file():
        return "maven", "mvn test"

    # Priority 7: SBT (Scala)
    if (root_path / "build.sbt").is_file():
        return "sbt", "sbt test"

    # Priority 8: Deno
    if (root_path / "deno.json").is_file() or (root_path / "deno.jsonc").is_file():
        return "deno", "deno test"

    # Priority 9: Go
    if (root_path / "go.mod").is_file():
        return "go", "go test ./..."

    # Priority 10: Makefile with test target
    makefile = root_path / "Makefile"
    if makefile.is_file():
        make_text = makefile.read_text()
        if re.search(r"^test:", make_text, re.MULTILINE):
            if "pytest" in make_text:
                return "pytest", "make test  # detected pytest in Makefile"
            return "make", "make test"

    # Priority 11: Python checks
    pyproject = root_path / "pyproject.toml"
    if pyproject.is_file():
        text = pyproject.read_text()
        if "[tool.pytest.ini_options]" in text:
            return "pytest", "pytest"

    setup_cfg = root_path / "setup.cfg"
    if setup_cfg.is_file() and "[tool:pytest]" in setup_cfg.read_text():
        return "pytest", "pytest"

    if (root_path / "pytest.ini").is_file():
        return "pytest", "pytest"

    # Check for Python test files
    tests_dir = root_path / "tests"
    if tests_dir.is_dir() and list(tests_dir.glob("*.py")):
        return "pytest", "pytest"

    return None


def main() -> None:
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    result = detect_framework(root)

    if result:
        framework, command = result
        print(f"{framework}:{command}")
        sys.exit(0)
    else:
        print("No test framework detected", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
