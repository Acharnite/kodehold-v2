# KodeHold v2.0 — User Guide

> **Version:** 2.0.0 · **Last Updated:** 2026-07-24
>
> KodeHold (/ˈkəʊd həʊld/) is a **specification compiler and code generation
> toolchain** — a single Rust binary for deterministic, auditable software
> production from DSL specifications.

---

## Table of Contents

1. [Philosophy & Principles](#1-philosophy--principles)
2. [Installation](#2-installation)
3. [Quick Start](#3-quick-start)
4. [Project Layout](#4-project-layout)
5. [Configuration (`kodehold.yaml`)](#5-configuration-kodeholdyaml)
6. [CLI Commands](#6-cli-commands)
   - [`init` — Create a project](#61-init--create-a-project)
   - [`verify` — Run project checks](#62-verify--run-project-checks)
   - [`ship` — Version, tag, and release](#63-ship--version-tag-and-release)
   - [`state` — Show workspace state](#64-state--show-workspace-state)
   - [`pipeline` — Run the compilation pipeline](#65-pipeline--run-the-compilation-pipeline)
   - [`spec` — Compile a `.khold` spec file](#66-spec--compile-a-khold-spec-file)
   - [`pattern` — Manage the pattern database](#67-pattern--manage-the-pattern-database)
   - [`listen` — Start the event daemon](#68-listen--start-the-event-daemon)
   - [`plugin` — Manage hooks and plugins](#69-plugin--manage-hooks-and-plugins)
   - [`loop` — The continuous maintenance daemon](#610-loop--the-continuous-maintenance-daemon)
   - [`migrate` — Upgrading from v1.0](#611-migrate--upgrading-from-v10)
7. [The Loop Daemon](#7-the-loop-daemon)
   - [Feature Lifecycle](#71-feature-lifecycle)
   - [Worktree Isolation](#72-worktree-isolation)
   - [Creating a Feature Request](#73-creating-a-feature-request)
   - [Starting the Loop](#74-starting-the-loop)
   - [Maintenance & Pruning](#75-maintenance--pruning)
8. [The `.khold` DSL](#8-the-khold-dsl)
9. [Pipeline Phases](#9-pipeline-phases)
10. [Git-Native State Machine](#10-git-native-state-machine)
11. [Event System](#11-event-system)
12. [Plugin System](#12-plugin-system)
13. [Pattern Database](#13-pattern-database)
14. [JSON Output Mode](#14-json-output-mode)
15. [Migration from v1.0](#15-migration-from-v10)
16. [Development & Contributing](#16-development--contributing)

---

## 1. Philosophy & Principles

KodeHold is built on five core principles:

| Principle | Meaning |
|-----------|---------|
| **Specification-first** | Describe *what* you want in the `.khold` DSL, not *how* to build it |
| **Deterministic by default** | Equivalent input → equivalent output. AI is a fallback, not the primary path |
| **Git as source of truth** | All state lives in git tags. No marker files, no side channels |
| **Events over polling** | Webhooks and file watchers, not cron ticks |
| **Zero runtime dependencies** | One static Rust binary. No Python, no Node.js, no `pip install` |

### What v2.0 IS

- A **specification compiler** that generates code deterministically
- A **constraint solver** that matches problems to known solutions
- An **event-driven system** that reacts to git events, not cron ticks
- A **loop daemon** for continuous workspace maintenance
- A single **static binary** with zero runtime dependencies

### What v2.0 is NOT

- NOT an AI agent orchestrator
- NOT a chat-based workflow engine
- NOT a marker-file state machine
- NOT a Python-only tool

---

## 2. Installation

### Prerequisites

- A **C/C++ compiler** (e.g., GCC, MSVC, or Clang) — required by the Rust toolchain
- **Git** (for git-native state management)

### Install via Rustup

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows, you may also need the GNU toolchain:
# rustup toolchain install stable-gnu
# rustup default stable-gnu
```

### Build from Source

```bash
git clone <repository-url>
cd kodehold-v2
cargo build --release

# The binary is at: ./target/release/kodehold
# Add to your PATH or copy it:
cp target/release/kodehold ~/.local/bin/   # Linux/macOS
# or just use: cargo run -- <args>
```

### Verify Installation

```bash
kodehold --version
# → KodeHold v2.0.0
```

---

## 3. Quick Start

```bash
# 1. Create a new project
kodehold init my-project --language rust

# 2. Enter the project
cd my-project

# 3. Verify it (runs checks + lint)
kodehold verify

# 4. Check the current state
kodehold state

# 5. Write a specification (optional)
cat > spec.khold << 'EOF'
project "my-project" {
  version = "0.1.0"
  language = "rust"
}

type UserId = u64
type UserName = String

interface UserRepository {
  fn find(id: UserId) -> Result<User, Error>
  fn save(user: User) -> Result<(), Error>
}
EOF

# 6. Compile the spec to code
kodehold spec spec.khold

# 7. Run the full pipeline
kodehold pipeline

# 8. Ship a new version
kodehold ship --bump patch
```

---

## 4. Project Layout

A KodeHold workspace created with `kodehold init` looks like this:

```
my-project/
├── kodehold.yaml          # Project configuration (YAML)
├── .gitignore             # Language-specific gitignore
├── .kodehold/             # KodeHold internal state
│   ├── features/          # Feature request YAML files (loop daemon)
│   └── worktrees/         # Git worktrees for feature branches
├── hooks/                 # Plugin hook scripts (optional)
│   ├── pre-generate
│   ├── post-verify
│   └── on-ship
├── src/                   # Source code
│   └── ...
├── tests/                 # Tests
│   └── ...
└── spec.khold             # DSL specification (optional)
```

### Key Files

| File | Purpose |
|------|---------|
| `kodehold.yaml` | Project metadata, pipeline config, hook registration |
| `.kodehold/features/*.yaml` | Feature request definitions (created via `kodehold loop feature`) |
| `.kodehold/worktrees/` | Git worktrees isolated per feature |
| `hooks/` | Executable hook scripts (any language) |

---

## 5. Configuration (`kodehold.yaml`)

The `kodehold.yaml` file is the single configuration point for a workspace.

### Default Structure

```yaml
version: "0.1.0"
name: my-project
language: rust
pipeline:
  phases:
    - parse
    - analyze
    - generate
    - verify
    - optimize
  verify: []
  optimize: []
hooks:
  pre-generate: null
  post-verify: null
  on-ship: null
loops:
  features_dir: ".kodehold/features"
  worktrees_dir: ".kodehold/worktrees"
  auto_maintain: true
  maintain_interval_secs: 3600
  merge_strategy: "merge"
  auto_merge: false
  max_concurrent_features: 3
  auto_labels:
    - "auto"
```

### Fields

| Field | Description | Default |
|-------|-------------|---------|
| `version` | Semantic version of the project | `"0.1.0"` |
| `name` | Project name | *(required)* |
| `language` | Primary programming language | `"rust"` |
| `pipeline.phases` | Ordered list of pipeline phases | `[parse, analyze, generate, verify, optimize]` |
| `pipeline.verify` | Extra verification commands to run | `[]` |
| `pipeline.optimize` | Extra optimization commands to run | `[]` |
| `hooks.*` | Paths to hook scripts relative to project root | `null` |
| `loops.*` | Loop daemon configuration (see §7) | *(see defaults)* |

---

## 6. CLI Commands

### 6.1 `init` — Create a project

```bash
kodehold init <name> [--language <lang>] [--json]
```

Creates a new KodeHold workspace with:
- `kodehold.yaml` config
- Language-appropriate `.gitignore`
- `.kodehold/` directory
- Git repository initialized with `v2-init` tag

**Examples:**

```bash
# Create a Rust project
kodehold init my-app

# Create a Python project
kodehold init my-api --language python

# Create with JSON output (for scripting)
kodehold init my-app --json
```

---

### 6.2 `verify` — Run project checks

```bash
kodehold verify [--path <path>] [--json]
```

Runs verification checks:
- **Uncommitted files** — checks for dirty working tree
- **State tag** — reports current lifecycle state
- **Custom commands** — runs any commands listed in `pipeline.verify`

**Examples:**

```bash
# Verify the current directory project
kodehold verify

# Verify a specific project
kodehold verify --path /path/to/project

# JSON mode
kodehold verify --json
```

**Output (human):**
```
  ✓ uncommitted-files: Clean working tree
  ✓ state: State: INIT (tag v2-init)

  2 passed, 0 failed
```

**Output (JSON):**
```json
{
  "status": "pass",
  "checks": [
    { "name": "uncommitted-files", "status": "Pass", "message": "Clean working tree" },
    { "name": "state", "status": "Pass", "message": "State: INIT (tag v2-init)" }
  ],
  "summary": { "passed": 2, "failed": 0, "total": 2 }
}
```

---

### 6.3 `ship` — Version, tag, and release

```bash
kodehold ship [--bump <major|minor|patch>] [--path <path>] [--no-push] [--json]
```

Performs a complete release cycle:

1. **Runs `on-ship` hook** (if configured) — the hook can block the release
2. **Bumps version** in `kodehold.yaml` according to `--bump`
3. **Generates changelog** from commits since the last tag
4. **Creates git tag** with the new version (e.g., `v0.2.0`)
5. **Pushes** commits and tags to remote (unless `--no-push`)

**Examples:**

```bash
# Patch bump (0.1.0 → 0.1.1)
kodehold ship --bump patch

# Minor bump (0.1.0 → 0.2.0)
kodehold ship --bump minor

# Major bump (0.1.0 → 1.0.0)
kodehold ship --bump major

# Dry run (no push)
kodehold ship --bump patch --no-push
```

---

### 6.4 `state` — Show workspace state

```bash
kodehold state [--path <path>] [--json]
```

Displays the current lifecycle state derived from git tags.

**Output (human):**
```
  State: INIT
  Tag:   v2-init
  Commits: 1

  Lifecycle:
    → v2-init (INIT)
       Initial commit
```

**Output (JSON):**
```json
{
  "current": "INIT",
  "current_tag": "v2-init",
  "commit_count": 1,
  "history": [
    { "state": "INIT", "tag": "v2-init", "message": "KodeHold v2.0 — my-project initialized" }
  ]
}
```

---

### 6.5 `pipeline` — Run the compilation pipeline

```bash
kodehold pipeline [--path <path>] [--phases <phases>] [--json]
```

Runs the 5-phase compilation pipeline. See §9 for phase details.

**Examples:**

```bash
# Run full pipeline
kodehold pipeline

# Run specific phases only
kodehold pipeline --phases parse,analyze

# JSON output
kodehold pipeline --json
```

---

### 6.6 `spec` — Compile a `.khold` spec file

```bash
kodehold spec <path-to-spec> [--output <dir>] [--json]
```

Compiles a `.khold` DSL specification file into source code. See §8 for DSL syntax.

**Examples:**

```bash
# Compile with default output (spec's src/ directory)
kodehold spec spec.khold

# Compile to a specific output directory
kodehold spec spec.khold --output ./generated

# JSON output
kodehold spec spec.khold --json
```

---

### 6.7 `pattern` — Manage the pattern database

```bash
kodehold pattern list [--language <lang>] [--json]
kodehold pattern search <query> [--json]
kodehold pattern add <path-to-json> [--json]
```

Manages the constraint solver's known-solutions database.

**Examples:**

```bash
# List all patterns
kodehold pattern list

# List patterns for a specific language
kodehold pattern list --language rust

# Search patterns
kodehold pattern search "Vec<Price> -> Regime"

# Add a pattern from a JSON file
kodehold pattern add ./patterns/my-pattern.json
```

**Pattern JSON format:**
```json
{
  "id": "pat-001",
  "name": "sort-ascending",
  "signature": "fn(Vec<T>) -> Vec<T> where T: Ord",
  "language": "rust",
  "tags": ["sorting", "collection"],
  "template": "let mut v = input;\nv.sort();\nv",
  "tests": ["assert_eq!(apply(vec![3, 1, 2]), vec![1, 2, 3]);"],
  "usage_count": 0,
  "success_rate": 1.0
}
```

---

### 6.8 `listen` — Start the event daemon

```bash
kodehold listen [--path <path>] [--port <port>] [--webhook-secret <secret>]
                [--watch <path>...] [--no-watch]
```

Starts the event-driven daemon that listens for:
- **GitHub webhooks** (push, pull request, issues) on `POST /hooks/github`
- **Loop triggers** on `POST /hooks/loop`
- **File changes** via the built-in file watcher (watches `src/` and `tests/` by default)

**Examples:**

```bash
# Start with defaults (port 8080, watch src/ and tests/)
kodehold listen

# Custom port and watch paths
kodehold listen --port 9000 --watch src/ --watch config/

# File watcher only (no webhook server)
kodehold listen --no-watch

# With GitHub webhook secret verification
kodehold listen --webhook-secret my-secret
```

**Webhook endpoints:**

| Endpoint | Purpose |
|----------|---------|
| `POST /hooks/github` | GitHub webhook (push, PR, issues) |
| `POST /hooks/loop` | Trigger a loop cycle |
| `GET /health` | Health check |

---

### 6.9 `plugin` — Manage hooks and plugins

```bash
kodehold plugin list [--path <path>] [--json]
kodehold plugin run <hook-name> [--path <path>] [--json]
kodehold plugin init <hook-name> [--language <bash|python>] [--path <path>] [--json]
```

**Examples:**

```bash
# List all configured hooks
kodehold plugin list

# Run a specific hook
kodehold plugin run pre-generate

# Initialize a new hook script
kodehold plugin init pre-generate --language bash
kodehold plugin init post-verify --language python
```

---

### 6.10 `loop` — The continuous maintenance daemon

```bash
kodehold loop [--json] <subcommand> [args]
```

Subcommands:

| Subcommand | Description |
|------------|-------------|
| `start` | Start the loop daemon or run a single cycle |
| `feature` | Create a new feature request |
| `list` | List feature requests and active worktrees |
| `maintain` | Run a single maintenance cycle |
| `prune` | Prune abandoned worktrees |

See [§7 — The Loop Daemon](#7-the-loop-daemon) for full details.

**Examples:**

```bash
# Run a single loop cycle
kodehold loop start --once

# Start the daemon mode (runs every 60s)
kodehold loop start --interval 60

# Create a feature request
kodehold loop feature add-login \
  --title "Add user login" \
  --description "Implement OAuth2 login flow" \
  --priority P1 \
  --accept "User can log in with Google" \
  --accept "User can log in with GitHub" \
  --label frontend --label auth

# List all features and worktrees
kodehold loop list
```

---

### 6.11 `migrate` — Upgrading from v1.0

```bash
kodehold migrate [--path <path>] [--dry-run] [--json]
```

Detects and migrates v1.0 workspaces:
- Converts `.kodehold-state` files
- Removes legacy marker files (`.design_reviewed`, `.second_opinion_done`, etc.)
- Creates `kodehold.yaml` if missing
- Reports v1.0 scripts that should be ported

**Examples:**

```bash
# Preview what would be migrated
kodehold migrate --dry-run

# Run the migration
kodehold migrate

# JSON output
kodehold migrate --json
```

---

## 7. The Loop Daemon

The loop daemon is the continuous workspace maintenance system. It manages
feature requests through their lifecycle using git worktrees for isolation.

### 7.1 Feature Lifecycle

```
                    ┌──────────┐
                    │  PENDING │  ← Created via CLI, webhook, or file
                    └────┬─────┘
                         │ loop cycle picks it up
                    ┌────▼──────┐
                    │ IN PROGRESS│  ← Worktree created, pipeline runs
                    └────┬──────┘
                         │ verification passes
                    ┌────▼─────────┐
                    │ READY FOR    │
                    │ REVIEW       │  ← Awaiting human approval
                    └────┬─────────┘
                         │ approved
                    ┌────▼──────┐
                    │  APPROVED  │  ← Ready to merge
                    └────┬──────┘
                         │ merged to main
                    ┌────▼──────┐
                    │  MERGED    │  ← Done
                    └───────────┘

                    ┌───────────┐
                    │ CANCELLED  │  ← Abandoned at any stage
                    └───────────┘
```

### 7.2 Worktree Isolation

Each feature request gets its own **git worktree** in `.kodehold/worktrees/`.
This means:

- Features are developed in **isolated working trees** — changes don't interfere
- Each worktree is linked to a **feature branch** (e.g., `feature/add-login`)
- Worktrees can be **merged, rebased, or squashed** back to main
- Abandoned worktrees can be **pruned** cleanly

### 7.3 Creating a Feature Request

```bash
kodehold loop feature <name> \
  --title "<Short description>" \
  --description "<Detailed description>" \
  --accept "<Criterion>" \         # repeatable
  --priority <P0|P1|P2|P3> \
  --label <tag> \                  # repeatable
  --path <project-path>
```

Feature requests are stored as YAML in `.kodehold/features/<name>.yaml`:

```yaml
name: add-login
title: Add user login
description: Implement OAuth2 login flow with Google and GitHub providers
acceptance:
  - User can log in with Google
  - User can log in with GitHub
source: Cli
status: Pending
labels:
  - frontend
  - auth
priority: P1
```

### 7.4 Starting the Loop

#### Single Cycle

```bash
kodehold loop start --once
```

This runs one cycle:
1. Discover pending features
2. Start processing the highest-priority feature (create worktree, run pipeline)
3. Check for features ready to merge
4. Run maintenance tasks

#### Daemon Mode

```bash
# Default interval (60s)
kodehold loop start

# Custom interval
kodehold loop start --interval 120
```

In daemon mode, the loop runs continuously until `Ctrl+C`. Each cycle:

1. **Feature discovery** — scans `.kodehold/features/` for pending requests
2. **Worktree creation** — creates a git worktree and branch for the feature
3. **Pipeline execution** — runs the pipeline in the worktree
4. **Auto-merge** — if configured and verification passes, merge back
5. **Maintenance** — `git gc --auto`, dependency updates (if configured)
6. **Sleep** — wait for the configured interval

### 7.5 Maintenance & Pruning

#### Maintenance

```bash
# Run maintenance once
kodehold loop maintain

# Or let the daemon handle it automatically (auto_maintain: true)
```

Maintenance tasks:
- Runs `kodehold verify` on the main workspace
- Runs `git gc --auto` for garbage collection

#### Pruning

```bash
# Prune abandoned/merged worktrees
kodehold loop prune
```

This removes:
- Worktrees whose feature branch has been merged
- Worktrees marked as abandoned
- Runs `git worktree prune` to clean git internals

### Loop Configuration

In `kodehold.yaml` under the `loops` key:

| Field | Default | Description |
|-------|---------|-------------|
| `features_dir` | `.kodehold/features` | Where feature YAML files are stored |
| `worktrees_dir` | `.kodehold/worktrees` | Where git worktrees are created |
| `auto_maintain` | `true` | Run maintenance tasks each cycle |
| `maintain_interval_secs` | `3600` | How often to run maintenance (0 = once per start) |
| `merge_strategy` | `"merge"` | How to merge: `merge`, `rebase`, or `squash` |
| `auto_merge` | `false` | Auto-merge features that pass verification |
| `max_concurrent_features` | `3` | Maximum simultaneous feature worktrees |
| `auto_labels` | `["auto"]` | Labels that trigger auto-processing |

---

## 8. The `.khold` DSL

The KodeHold Specification Language (`.khold`) is used to describe software
designs declaratively.

### Syntax

```
// spec.khold — KodeHold Specification Language

// ── Project Declaration ──
project "my-trading-bot" {
  version = "0.1.0"
  language = "rust"
}

// ── Type Aliases ──
type Price = f64
type Timestamp = u64
type UserId = u64

// ── Struct Types ──
type OHLCV = struct {
  open: Price,
  high: Price,
  low: Price,
  close: Price,
  volume: u64,
}

// ── Enumerations ──
type Regime = enum {
  Bull,
  Bear,
  Neutral,
  Volatile,
}

// ── Interfaces ──
interface MarketDataFeed {
  fn fetch(
    symbol: String,
    from: Timestamp,
    to: Timestamp,
  ) -> Result<Vec<OHLCV>, Error>

  fn get_supported_symbols() -> Vec<String>
}

interface RegimeClassifier {
  fn classify(data: Vec<OHLCV>) -> Regime
}

// ── Functions with Constraints ──
fn calculate_sma(prices: Vec<Price>, period: u64) -> Vec<Price>
  where period > 0
```

### Features

| Construct | Description |
|-----------|-------------|
| `project` | Project metadata block |
| `type X = Y` | Type aliases, structs, and enums |
| `interface X { ... }` | Interface/trait definitions with function signatures |
| `fn name(...) -> T` | Standalone function signatures |
| `where` | Constraints on generic parameters |

---

## 9. Pipeline Phases

The pipeline transforms a `.khold` specification into verified source code
through five deterministic phases.

```
Spec (.khold) → Parse → Analyze → Generate → Verify → Optimize → Output
```

| Phase | Input | Output | AI? | Description |
|-------|-------|--------|-----|-------------|
| **1. Parse** | `.khold` file | AST | No | Tokenizes and parses the DSL into an Abstract Syntax Tree |
| **2. Analyze** | AST | Typed IR | No | Type-checks, resolves references, produces Intermediate Representation |
| **3. Generate** | Typed IR + patterns | Source code | Fallback | Matches IR patterns in the pattern DB; falls back to AI if no match |
| **4. Verify** | Source code + tests | Pass/Fail | No | Runs tests and verification commands |
| **5. Optimize** | Source code | Optimized code | No | Applies deterministic optimizations |

AI is used **only** in the Generate phase, and **only** when the constraint
solver cannot find a matching pattern in the known-solutions database.

---

## 10. Git-Native State Machine

KodeHold uses **git tags** to track workspace lifecycle state — no marker files,
no side-channel state.

### States

```
v2-init    ──  INIT      ──  After kodehold init
v2-active  ──  ACTIVE    ──  Pipeline running
v2-review  ──  REVIEW    ──  Verification complete, awaiting human review
v2-closed  ──  CLOSED    ──  Shipped / completed
v2-reopen-N   REOPEN(N)  ──  Re-opened after closure
```

### Usage

Tags are created by the relevant commands:

- `kodehold init` creates `v2-init`
- `kodehold pipeline` transitions `v2-init` → `v2-active` → `v2-review`
- `kodehold ship` creates the version tag and transitions to `v2-closed`

Check the current state with:

```bash
kodehold state
```

The state system enables:
- **Auditability** — full history of state transitions in git tags
- **CI/CD integration** — check tags to determine build stage
- **No hidden state** — everything is in git

---

## 11. Event System

The event daemon (`kodehold listen`) provides real-time response to external events.

### GitHub Webhooks

Configure your GitHub repository to send webhooks to `http://<your-host>:8080/hooks/github`.

The daemon handles:
- **Push events** — triggers pipeline
- **Pull request events** — triggers verification
- **Issue events** — dispatches to issue triage (creates feature requests)

### File Watcher

The built-in file watcher (powered by `notify`) monitors filesystem changes:

```bash
# Default: watches src/ and tests/
kodehold listen

# Custom paths
kodehold listen --watch src/ --watch docs/
```

### Health Check

```bash
curl http://localhost:8080/health
# → HTTP 200 OK
```

---

## 12. Plugin System

Plugins are hooks that run at specific points in the pipeline. They are
**executable scripts** (any language) that receive JSON on stdin and write
JSON to stdout. The core engine only cares about exit codes and stdout.

### Hook Points

| Hook | When it runs | Effect |
|------|-------------|--------|
| `pre-generate` | Before the Generate phase | Can prepare environment or validate inputs |
| `post-verify` | After the Verify phase | Can send notifications or record results |
| `on-ship` | Before `kodehold ship` executes | Can **block** the release by exiting non-zero |

### Creating a Hook

```bash
# Scaffold a bash hook
kodehold plugin init pre-generate --language bash

# Scaffold a Python hook
kodehold plugin init post-verify --language python
```

This creates an executable script in `hooks/` and registers it in `kodehold.yaml`.

### Hook Script Interface

**Input (stdin):** JSON with context information:
```json
{
  "project": "my-project",
  "language": "rust",
  "phase": "generate",
  "workspace": "/path/to/project"
}
```

**Output (stdout):** JSON with result:
```json
{
  "status": "ok",
  "message": "Environment prepared"
}
```

**Exit codes:**
- `0` — Success (continues)
- Non-zero — Failure (blocks the pipeline or hooks with `Blocked` status)

### Running Hooks Manually

```bash
kodehold plugin run pre-generate
```

### Listing Hooks

```bash
kodehold plugin list
```

Example output:
```
━━━ Hooks (3) ━━━
  ✓ hooks/pre-generate
  ✓ hooks/post-verify
  ✓ hooks/on-ship

  pre-generate: hooks/pre-generate
  post-verify: hooks/post-verify
  on-ship: hooks/on-ship
```

---

## 13. Pattern Database

The pattern database is the knowledge base used by the constraint solver during
the Generate phase. It stores known implementation patterns indexed by type
signature and tags.

### Pattern Format

```json
{
  "id": "pat-001",
  "name": "sort-ascending",
  "signature": "fn(Vec<T>) -> Vec<T> where T: Ord",
  "language": "rust",
  "tags": ["sorting", "collection", "utility"],
  "template": "let mut v = input;\nv.sort();\nv",
  "tests": [
    "assert_eq!(apply(vec![3, 1, 2]), vec![1, 2, 3]);",
    "assert_eq!(apply::<u32>(vec![]), vec![]);"
  ],
  "usage_count": 42,
  "success_rate": 0.98
}
```

### Commands

```bash
# List all patterns
kodehold pattern list

# Filter by language
kodehold pattern list --language rust

# Search by name, signature, or tags
kodehold pattern search "sort"

# Add a pattern from a JSON file
kodehold pattern add ./patterns/quick-sort.json
```

### How It Works

1. The **Analyze** phase produces a Typed IR with function signatures
2. The **Generate** phase queries the pattern DB for matching signatures
3. If a match is found, the pattern's template is instantiated
4. If no match is found, AI generation is attempted as a fallback
5. New successful patterns can be added to the DB for future reuse

---

## 14. JSON Output Mode

Most commands support `--json` for machine-readable output. This is useful for:

- **Scripting** — pipe output to other tools
- **CI/CD** — integrate with build pipelines
- **IDE plugins** — parse structured data

### Supported Commands

| Command | JSON Output |
|---------|-------------|
| `init --json` | `{ status, name, language, path, state, tag }` |
| `verify --json` | `{ status, checks: [{ name, status, message }], summary }` |
| `ship --json` | `{ status, old_version, new_version, tag, changelog }` |
| `state --json` | `{ current, current_tag, commit_count, history }` |
| `pipeline --json` | Phase results with status per phase |
| `spec --json` | `{ status, module, language, files, summary }` |
| `pattern list --json` | Array of pattern objects |
| `pattern search --json` | Array of matching pattern objects |
| `loop --json feature` | `{ status, feature, path, title, priority }` |
| `loop --json list` | `{ features: [], worktrees: [], summary }` |
| `loop --json start` | Pipeline report as JSON |
| `loop --json maintain` | `{ "status": "ok" }` |
| `loop --json prune` | `{ "status": "ok" }` |
| `migrate --json` | Array of migration items |
| `plugin list --json` | Hook configuration with all hooks |
| `plugin run --json` | `{ hook, status, message }` |
| `plugin init --json` | `{ status, path, language }` |

---

## 15. Migration from v1.0

If you have an existing KodeHold v1.0 workspace, the `migrate` command helps
transition to v2.0.

### What Migration Does

1. **Detects v1.0 state files** — reads `.kodehold-state` and converts relevant info
2. **Removes legacy markers** — deletes `.design_reviewed`, `.second_opinion_done`,
   `.testers_done`, `.code_reviewed`, `.impact_analysis_done`
3. **Creates `kodehold.yaml`** — if missing, generates a default config
4. **Reports v1.0 scripts** — notes any remaining Python scripts that need porting

### Running Migration

```bash
# Preview only
kodehold migrate --dry-run

# Run migration
kodehold migrate
```

### Manual Steps After Migration

1. Port any remaining v1.0 Python scripts to hooks (see §12)
2. Update CI/CD pipelines to use the new CLI syntax
3. Remove v1.0 directories if present: `rm -rf scripts/ config/tasks.yaml`

---

## 16. Development & Contributing

### Building

```bash
cargo build              # Debug build
cargo build --release    # Release build (optimized)
```

### Testing

```bash
cargo test               # All tests (unit + integration)
cargo test -- --nocapture # With stdout visible
cargo test <test-name>   # Single test
```

### Project Architecture

```
src/
├── main.rs           → Entry point
├── cli.rs            → CLI argument definitions (clap)
├── config.rs         → kodehold.yaml parsing
├── state.rs          → Git-native state machine
├── workspace.rs      → Workspace operations (init, verify, ship)
├── cmd/              → Command implementations
│   ├── init.rs
│   ├── verify.rs
│   ├── ship.rs
│   ├── state.rs
│   ├── pipeline.rs
│   ├── spec.rs
│   ├── pattern.rs
│   ├── listen.rs
│   ├── plugin.rs
│   ├── loop.rs
│   └── migrate.rs
├── dsl/              → .khold spec language
│   ├── parser.rs     → DSL parser
│   ├── ast.rs        → Abstract Syntax Tree
│   ├── ir.rs         → Intermediate Representation
│   └── codegen.rs    → Code generator
├── pipeline/         → 5-phase pipeline
├── solver/           → Constraint solver + pattern DB
├── loops/            → Loop daemon
│   ├── mod.rs        → LoopEngine, LoopConfig
│   ├── feature.rs    → Feature request store
│   └── worktree.rs   → Git worktree manager
├── events/           → Event system
│   ├── watcher.rs    → File watcher
│   └── webhook.rs    → Webhook server (axum)
├── plugin/           → Hook system
└── util/             → Shared utilities
```

### Key Files

| File | Purpose |
|------|---------|
| `AGENTS.md` | Development rules for AI coding agents |
| `CHANGES.md` | Full changelog since v0.19 |
| `VERSION.md` | Version history and scheme |
| `docs/design/README.md` | Living design document |
| `docs/adr/` | Architecture Decision Records (ADR-0001 to ADR-0010) |

---

## Appendix A: Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error (invalid args, missing files, etc.) |

## Appendix B: Environment

KodeHold v2.0 has no required environment variables. It reads only:

- `kodehold.yaml` from the project root for configuration
- The filesystem for feature request YAML files
- Git tags for state management

---

*For more details, see the [Design Document](design/README.md) and
[Architecture Decision Records](adr/).*
