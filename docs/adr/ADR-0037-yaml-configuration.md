---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0037: YAML-Based Agent and Task Configuration

## Status

Accepted

**Version:** 1.0
**Last Updated:** 2026-06-02
**Phase:** Design — schema definition and migration planning

## Context

### Current State

KodeHold defines 8 agents as `.md`-files in `.opencode/agents/` with YAML frontmatter + prose prompts. Each file contains both machine-readable configuration (`name`, `mode`, `permission`, `model`) and human-readable instructions (the prose body).

Here is the current breakdown of every attribute stored in YAML frontmatter across all 8 agents:

| Attribute | Director | Architects | Engineers | Reviewers | Testers | Scribes | FLS | Second Opinion |
|-----------|----------|------------|-----------|-----------|---------|---------|-----|----------------|
| `name` | director | architects | engineers | reviewers | testers | scribes | fls | second-opinion |
| `description` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `mode` | `all` | `subagent` | `subagent` | `subagent` | `subagent` | `subagent` | `subagent` | `subagent` |
| `hidden` | — | — | — | — | — | — | — | `true` |
| `model` | — | — | — | — | — | — | — | `openrouter/google/gemma-3-12b-it` |
| `permission.read` | allow | allow | allow | allow | allow | allow | allow | allow |
| `permission.write` | deny | allow | allow | allow | allow | allow | allow | deny |
| `permission.edit` | deny | allow | allow | allow | allow | allow | allow | deny |
| `permission.glob` | allow | allow | allow | allow | allow | allow | allow | allow |
| `permission.grep` | allow | allow | allow | allow | allow | allow | allow | allow |
| `permission.bash` | allow | allow | allow | allow | allow | allow | allow | deny |
| `permission.task` | allow | deny | deny | deny | deny | deny | deny | deny |
| `permission.skill` | allow | — | allow | allow | — | — | allow | allow |
| `permission.webfetch` | allow | allow | — | — | — | — | — | — |
| `permission.websearch` | allow | allow | — | — | — | — | — | — |
| `external_directory` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |

### Problems with the Current Approach

1. **Configuration mixed with documentation.** The YAML frontmatter (machine config) and the prose body (human instructions) live in the same file. Changing a permission bit means editing a 50+ line markdown file where the actual change is buried. Git diffs show both config and prose changes intermixed.

2. **No schema validation.** There is no JSON Schema or equivalent validating the frontmatter. A typo like `permision:` instead of `permission:` is not caught until OpenCode tries to parse the file at runtime — and even then, OpenCode silently ignores unknown fields. Errors surface only as mysterious permission denials.

3. **Hard to reuse.** Agent configurations cannot be composed or inherited. If two agents share the same `external_directory` rules, those rules must be copy-pasted across both `.md` files. There is no `defaults` or `extends` mechanism.

4. **No machine-readable task/workflow definitions.** Workflows (implement-flow, adr-flow, ship-gate) are currently documented only in prose inside `director.md` — in the Routine Templates table and the Triage-Check Protocol. They cannot be parsed, validated, or versioned independently.

5. **Trigger keywords are hidden.** Each agent's triggers are listed in its `description` field as part of the prose string (e.g., `Triggers: design, ADR, architecture`). There is no structured `triggers: [...]` field. This prevents OpenCode or any tooling from doing trigger-based routing.

6. **Model override has no standard place.** Only `second-opinion` uses a `model:` field in its frontmatter. OpenCode supports per-agent model configuration (as seen in the `agent` block of `opencode.json`), but the current structure makes it unclear where `model` should live — in the `.md` frontmatter, in `opencode.json`, or in a separate config file.

7. **Token waste.** YAML frontmatter in a markdown file that is 90% prose still means the entire file must be loaded and parsed by the LLM as agent context. Separating config from prose would reduce context size for agents that only need the config.

### Key Forces

1. **Backwards compatibility is mandatory.** OpenCode currently only supports loading agents from `.opencode/agents/*.md` files. The YAML config must be an **overlay** that enriches the agent definitions — not a replacement that breaks agent loading. Agents must continue to work even if the YAML file is missing or empty.

2. **Schema validation must be available.** The config must be validateable with standard tools (`yaml` + Python `jsonschema`, or `ajv` for Node.js). A JSON Schema file must accompany the YAML file.

3. **Separation of concerns.** Configuration (model, permissions, triggers, tools) must be cleanly separated from documentation (prose instructions, workflow descriptions, state awareness rules).

4. **Forward-compatible.** The schema must leave room for: per-agent model overrides (Multi-Model Task Routing #41), tools/plugin restrictions, budget limits, and constraint definitions — even if these are not used today.

5. **Trigger-driven routing.** Agent triggers should be a first-class, structured field — not buried in the prose description. This enables tooling to route tasks by trigger matching.

## Decision

### 1. File Layout

We introduce a `config/` directory at the project root with two YAML files:

```
kodehold/
├── config/
│   ├── agents.yaml        # Agent configuration (all 8 agents)
│   ├── tasks.yaml          # Workflow and gate definitions
│   └── agents.schema.json  # JSON Schema for agents.yaml validation
```

Additionally, we define a JSON Schema file (`agents.schema.json`) for validating `agents.yaml`.

### 2. `config/agents.yaml` Schema

The YAML schema covers all attributes currently in the `.md` frontmatter plus forward-looking fields:

```yaml
# agents.yaml — KodeHold Agent Configuration
# Schema: config/agents.schema.json
# Version: 1.0

# Default values applied to all agents unless overridden
defaults:
  mode: subagent
  permission:
    read: allow
    write: allow
    edit: allow
    glob: allow
    grep: allow
    bash: allow
    task: deny
    skill: allow
    webfetch: deny
    websearch: deny
  external_directory:
    "*": ask
    "/home/kiffer/project/**": allow
    "/tmp/**": allow
    "/home/kiffer/docker/**": allow

agents:
  - name: director
    # ... full config per agent
  - name: architects
    # ...
```

**Schema fields:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | yes | — | Agent identifier, lowercase kebab-case. Must match directory name. |
| `description` | string | yes | — | Human-readable description of the agent's role. |
| `mode` | enum | no | `subagent` | Agent mode: `all` (autonomous) or `subagent` (called by other agents). |
| `hidden` | boolean | no | `false` | If true, agent is not listed in auto-discovery. Used for second-opinion. |
| `model` | string | no | — | Model override for this agent. Format: `<provider>/<model-id>`. Enables per-agent model routing. |
| `triggers` | list[string] | no | `[]` | Keywords/phrases that activate this agent. |
| `tags` | list[string] | no | `[]` | Arbitrary tags for categorization and filtering. |
| `permission` | object | no | defaults | Tool permissions. See Permission Object below. |
| `tools` | object | no | — | MCP tool restrictions. See Tools Object below. |
| `constraints` | object | no | — | Operational constraints. See Constraints Object below. |

**Permission Object:**

| Sub-field | Type | Values | Description |
|-----------|------|--------|-------------|
| `read` | enum | `allow`, `deny`, `ask` | File read permission |
| `write` | enum | `allow`, `deny`, `ask` | File write permission |
| `edit` | enum | `allow`, `deny`, `ask` | File edit permission |
| `glob` | enum | `allow`, `deny`, `ask` | Glob pattern matching |
| `grep` | enum | `allow`, `deny`, `ask` | Content search |
| `bash` | enum | `allow`, `deny`, `ask` | Shell command execution |
| `task` | enum | `allow`, `deny`, `ask` | Task tool delegation |
| `skill` | enum | `allow`, `deny`, `ask` | Skill loading |
| `webfetch` | enum | `allow`, `deny`, `ask` | URL fetching |
| `websearch` | enum | `allow`, `deny`, `ask` | Web search |
| `external_directory` | object | pattern → enum | Directory-specific access rules |

**Permission Precedence:**

When multiple configuration layers define permissions, the following precedence order applies (highest wins):

```
Precedence (highest → lowest):
1. Agent-specific permission override   (agents[].permission)
2. defaults block                        (defaults.permission)
3. Hard-coded system fallback            (deny — safest default)
```

**Rules:**

- If a permission is defined at the **agent level** (e.g., `agents[0].permission.write: deny`), that value wins unconditionally.
- If a permission is **absent** at the agent level but present in the **`defaults`** block, the default is used.
- If a permission is **absent in both** the agent config and the `defaults` block, the system falls back to **`deny`** — the safest possible default. This ensures that an omitted permission never accidentally grants access.

**Example:**

```yaml
defaults:
  permission:
    read: allow
    write: allow
    bash: deny

agents:
  - name: director
    # inherits read=allow, write=allow from defaults
    # inherits bash=deny from defaults
  - name: second-opinion
    permission:
      write: deny           # overrides default — explicit deny
      bash: deny            # explicit (same as default, but clearer)
      webfetch: allow       # extends defaults — new permission not in defaults
    # read: allow           # inherited from defaults
```

In this example, `director` has `bash: deny` via defaults, while `second-opinion` explicitly denies `write` and `bash`. If a new permission `edit` were omitted from both the agent config and defaults, it would resolve to `deny` — preventing accidental escalation.

**`external_directory` precedence** follows the same rule: agent-specific directory rules override defaults. If neither defines a pattern for a given path, the system asks (equivalent to `"*": ask` behavior).

**Tools Object:**

| Sub-field | Type | Description |
|-----------|------|-------------|
| `allowed` | list[string] | MCP tool names this agent may use |
| `denied` | list[string] | MCP tool names this agent may NOT use |

**Constraints Object:**

| Sub-field | Type | Description |
|-----------|------|-------------|
| `max_tokens_per_response` | integer | Maximum output tokens per response |
| `budget` | object | Per-phase token budgets |
| `light_mode_compatible` | boolean | Whether agent supports KODEHOLD_LIGHT=1 |

### 3. `config/tasks.yaml` Schema

A separate YAML file for workflow and gate definitions, extracting this from the prose in `director.md`:

```yaml
# tasks.yaml — KodeHold Task and Workflow Configuration
# Version: 1.0

workflows:
  - id: implement-flow
    name: "Standard Implementation Flow"
    description: "Feature implementation from design through testing"
    steps:
      - id: design
        team: architects
        description: "Create/update design doc and ADRs"
      - id: design-review
        team: reviewers
        gate: design_review
        description: "Review design quality"
      - id: implement
        team: engineers
        description: "Implement per design doc specification"
      - id: code-review
        team: reviewers
        gate: code_review
        description: "Review implementation against design"
      - id: test
        team: testers
        gate: tests_pass
        description: "Write and execute tests"
      - id: comprehensive-review
        team: reviewers
        gate: comprehensive_review
        description: "Final review before state transition"

  - id: adr-flow
    name: "ADR Creation Flow"
    description: "Create and validate an Architecture Decision Record"
    steps:
      - id: write-adr
        team: architects
        description: "Author ADR following Nygard format"
      - id: review-adr
        team: reviewers
        description: "Review ADR for completeness and consistency"
      - id: second-opinion
        team: second-opinion
        description: "Cross-model validation of ADR"

  - id: bugfix-flow
    name: "Bug Fix Flow"
    description: "Triage, fix, and verify a bug"
    steps:
      - id: triage
        team: fls
        description: "Determine if minor (fix) or major (escalate)"
      - id: fix
        team: engineers
        description: "Apply the fix"
      - id: verify
        team: testers
        description: "Verify the fix works"

  - id: ship-gate
    name: "Shipping Gate"
    description: "Pre-release verification and release steps"
    steps:
      - id: team-meeting
        type: manual
        description: "All 6 teams review and sign off"
      - id: pre-ship-verification
        type: automated
        script: scripts/ship.sh
        description: "Automated pre-ship checks"
      - id: version-bump
        team: scribes
        description: "Bump VERSION.md"
      - id: changelog
        team: scribes
        description: "Update CHANGES.md"
      - id: commit-and-tag
        type: automated
        description: "Commit, tag, and push release"

gates:
  - id: design_review
    marker: .design_reviewed
    description: "Design quality approved by Reviewers"
    checks:
      - design_doc_sections_complete
      - adrs_written
      - second_opinion_obtained
  - id: code_review
    marker: .code_reviewed
    description: "Code reviewed against design doc"
    checks:
      - matches_design_spec
      - no_security_issues
      - follows_conventions
  - id: tests_pass
    marker: .testers_done
    description: "Tests written and passing"
    checks:
      - unit_tests_pass
      - integration_tests_pass
      - no_regressions
  - id: comprehensive_review
    marker: .testers_done
    description: "Final gate before ACTIVE → REVIEW"
    checks:
      - all_gates_pass
      - design_doc_current
  - id: impact_analysis
    marker: .impact_analysis_done
    description: "Architects completed impact assessment"
    checks:
      - design_doc_updated
      - adrs_written
```

> **Note:** `tasks.yaml` does not yet have a dedicated JSON Schema file. A `config/tasks.schema.json` should be created to validate workflow and gate definitions. The current schema (workflows with steps, gates with checks) serves as the specification. For now, tasks are validated via integration tests in `tests/init/` rather than schema validation.

### 4. Relationship Between YAML and `.md` Files

The YAML config is an **overlay** — it does not replace the `.md` files. The relationship is:

```
.opencode/agents/architects.md     config/agents.yaml
┌─────────────────────┐            ┌──────────────────┐
│ YAML Frontmatter    │            │ agents:          │
│   name: architects  │   SOURCE   │   - name: arch.. │
│   mode: subagent    │ ─────────►│     permission:.. │
│   permission: {...} │  OF TRUTH  │     triggers: .. │
├─────────────────────┤            │     model: ...   │
│ Prose Instructions  │            └──────────────────┘
│ # Architects        │
│ You are the design  │   DOCUMENTATION ONLY
│ authority...        │   (Prose stays in .md)
│                     │
│ ## Responsibilities │
│ 1. Create design..  │
└─────────────────────┘
```

**Rules:**
- The YAML file is the **source of truth** for machine configuration (permissions, model, triggers, mode, hidden, tools)
- The `.md` files retain their **prose documentation** (instructions, responsibilities, workflows for human readers)
- The YAML frontmatter in `.md` files becomes **optional** after migration — it may be removed entirely once OpenCode supports reading from YAML
- Both files must remain synchronized (see Migration Strategy). This synchronization is required for consistency, not because the YAML lacks stand-alone value. See §8 below.

### 5. Validation

A JSON Schema file (`config/agents.schema.json`) validates `agents.yaml`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "defaults": { "$ref": "#/definitions/agentDefaultsConfig" },
    "agents": {
      "type": "array",
      "items": { "$ref": "#/definitions/agentConfig" },
      "minItems": 1
    }
  },
  "required": ["agents"],
  "definitions": {
    "agentConfig": {
      "type": "object",
      "properties": {
        "name": { "type": "string", "pattern": "^[a-z][a-z0-9-]*$" },
        "description": { "type": "string" },
        "mode": { "type": "string", "enum": ["all", "subagent"] },
        "hidden": { "type": "boolean" },
        "model": { "type": "string" },
        "triggers": { "type": "array", "items": { "type": "string" } },
        "tags": { "type": "array", "items": { "type": "string" } },
        "permission": { "$ref": "#/definitions/permissionConfig" },
        "tools": { "$ref": "#/definitions/toolsConfig" },
        "constraints": { "$ref": "#/definitions/constraintsConfig" }
      },
      "required": ["name", "description"]
    },
    "agentDefaultsConfig": {
      "type": "object",
      "properties": {
        "name": { "type": "string", "pattern": "^[a-z][a-z0-9-]*$" },
        "description": { "type": "string" },
        "mode": { "type": "string", "enum": ["all", "subagent"] },
        "hidden": { "type": "boolean" },
        "model": { "type": "string" },
        "triggers": { "type": "array", "items": { "type": "string" } },
        "tags": { "type": "array", "items": { "type": "string" } },
        "permission": { "$ref": "#/definitions/permissionConfig" },
        "tools": { "$ref": "#/definitions/toolsConfig" },
        "constraints": { "$ref": "#/definitions/constraintsConfig" }
      }
    },
    "permissionConfig": {
      "type": "object",
      "properties": {
        "read": { "enum": ["allow", "deny", "ask"] },
        "write": { "enum": ["allow", "deny", "ask"] },
        "edit": { "enum": ["allow", "deny", "ask"] },
        "glob": { "enum": ["allow", "deny", "ask"] },
        "grep": { "enum": ["allow", "deny", "ask"] },
        "bash": { "enum": ["allow", "deny", "ask"] },
        "task": { "enum": ["allow", "deny", "ask"] },
        "skill": { "enum": ["allow", "deny", "ask"] },
        "webfetch": { "enum": ["allow", "deny", "ask"] },
        "websearch": { "enum": ["allow", "deny", "ask"] },
        "external_directory": {
          "type": "object",
          "additionalProperties": { "enum": ["allow", "deny", "ask"] }
        }
      }
    },
    "toolsConfig": {
      "type": "object",
      "properties": {
        "allowed": { "type": "array", "items": { "type": "string" } },
        "denied": { "type": "array", "items": { "type": "string" } }
      }
    },
    "constraintsConfig": {
      "type": "object",
      "properties": {
        "max_tokens_per_response": { "type": "integer", "minimum": 1 },
        "budget": { "type": "object" },
        "light_mode_compatible": { "type": "boolean" }
      }
    }
  }
}
```

### 6. Trigger Extraction

Currently, triggers are embedded in each agent's `description` field as free text:

```markdown
description: >
  Code review against design doc and standards...
  Triggers: review, code review, design review, standards
```

In the new schema, triggers become a first-class array field:

```yaml
triggers:
  - review
  - code review
  - design review
  - standards
```

> **Note on Director triggers:** The Director has no triggers defined — as orchestrator, it handles all incoming requests by routing to subagents. Trigger-based routing applies to subagents only.

#### 6.1 Trigger Namespace Convention

To prevent unpredictable behavior from overlapping triggers, we define a lightweight namespace convention:

**Principle:** Agent triggers are inherently namespaced by the agent they belong to. The routing pipeline (Director → trigger match → subagent) is:

```
Incoming request → Director parses intent
                  → Matches trigger keywords against agent trigger lists
                  → Director delegates to the matching subagent (or best match)
```

Since each trigger belongs to exactly one agent, there is no ambiguity at the delegation point — the Director always knows *which* agent a matched trigger belongs to.

**Rules:**

1. **No duplicate triggers across agents.** CI validation enforces that every trigger keyword appears in at most one agent's trigger list. If two agents legitimately need the same keyword (e.g., both `reviewers` and `architects` handle "review"), the keyword must be qualified.

2. **Qualification by context:** Triggers should be specific enough to avoid false matches:
   - ✅ `design` → only `architects`
   - ✅ `code review` → only `reviewers`
   - ✅ `bug triage` → only `fls`
   - ❌ `review` (too generic — could match reviewers, architects, or testers)

3. **Central trigger registry.** The complete set of all triggers across all agents is documented in `config/agents.yaml` as a machine-readable comment block. A `scripts/list-triggers.sh` helper extracts and displays the registry:

   ```yaml
   # config/agents.yaml
   # === CENTRAL TRIGGER REGISTRY ===
   # architects:    design, ADR, architecture, design review
   # engineers:     implement, code, feature, refactor
   # reviewers:     code review, design review, comprehensive review
   # testers:       test, unit test, integration test, regression
   # fls:           bug, error, fix, investigate, debug, triage
   # scribes:       document, changelog, publish, write docs
   # second-opinion: second opinion, cross-model, validate-adr
   # === END REGISTRY ===
   ```

4. **No prefix namespace required at this scale.** For 8 agents with <5 triggers each, namespace prefixes like `reviewers:review` or `architects:design` are overkill. If the system grows beyond 20 agents or triggers per agent, a `team:keyword` prefix convention should be re-evaluated.

**Delegation authority:** The Director retains final routing authority. Triggers are **advisory metadata** that guide the Director's routing decision — they do not override the Director's triage logic. If the Director determines a different agent is appropriate, it may override trigger-based routing.

This enables:
- Tooling to match incoming requests to agents by trigger keywords
- CI validation that every trigger is unique (no overlap between agents)
- A single, authoritative trigger registry visible in one YAML file
- Future trigger-based routing without natural language parsing

### 7. Backwards Compatibility

The YAML configuration is designed to be a pure overlay:

1. **OpenCode continues to load agents from `.opencode/agents/*.md`** — the YAML file is not consumed by OpenCode at this time
2. **`opencode.json` remains unchanged** — the `agent` block continues to reference `.md` files
3. **The YAML file is validated independently** — `scripts/validate-config.sh` (or equivalent) validates `agents.yaml` against `agents.schema.json`
4. **A sync script (`scripts/sync-agent-config.sh`)** bidirectionally syncs YAML frontmatter between `.md` files and `agents.yaml`.  
   **Conflict resolution (tiebreaker):**
   - `.md → YAML` direction: the `.md` frontmatter values win
   - `YAML → .md` direction: the YAML values win
   - `diff` direction (detect only): report mismatches and exit non-zero without modifying files
5. **If OpenCode ever adds native YAML support**, the migration path is: update the `file` field in `opencode.json` to point to `config/agents.yaml` instead of `.opencode/agents/*.md`

### 8. Immediate Value (Independent of OpenCode Native Support)

The YAML configuration files deliver value **immediately**, regardless of whether OpenCode ever adds native YAML agent loading. OpenCode-native YAML support is Phase 6 — a future bonus, not a prerequisite. The following capabilities are available from Phase 1 onward:

**Validation and correctness:**
- `scripts/validate-config.sh` validates `config/agents.yaml` against `config/agents.schema.json` — catching typos, invalid permissions, and missing fields before runtime
- The CI pipeline (`kodehold-ci.yml`) runs this validation on every change, preventing config drift from reaching production
- Unit tests in `tests/init/` assert schema behavior with known-good and known-bad inputs

**Consistency enforcement:**
- `scripts/sync-agent-config.sh` bidirectionally synchronizes YAML frontmatter between `.md` files and `config/agents.yaml`
- The sync script is idempotent and can run in `diff` mode (report-only, exit non-zero on mismatch) to gate CI
- Gate.sh includes a config-validation step, so state transitions fail if the YAML config is inconsistent

**Tooling and automation:**
- Editors and IDE extensions can parse `config/agents.yaml` directly to show agent permissions, triggers, and model overrides — without parsing 200-line markdown files
- A workflow monitor tool (future) can read `config/tasks.yaml` to visualize pipeline state, track gate progress, and generate status reports
- Structured triggers enable automated task routing without natural language parsing — even without OpenCode support, custom tooling can use them

**Human-readable single source of truth:**
- Team members can review all agent configurations in one concise YAML file rather than across 8 markdown files
- `git diff` on a permission change shows 2 lines in `agents.yaml` instead of a 200-line markdown diff
- The `defaults:` block eliminates duplicated configuration — proving immediately useful for the 8-agent setup today

**Summary:** OpenCode-native YAML support (Phase 6) would eliminate the sync script and allow removing `.md` frontmatter entirely. But the YAML files are a net positive from day one — they provide validation, consistency, and tooling value that the `.md`-only approach cannot. The architecture is designed so that even if Phase 6 never happens, the system is strictly better than the status quo ante.

## Consequences

### Positive

1. **Clean separation of concerns.** Machine configuration (permissions, model, triggers) lives in YAML. Human documentation (prose instructions, workflows) stays in `.md`. Changing a permission requires editing a focused YAML file, not searching through 200+ lines of prose.

2. **Schema validation.** JSON Schema catches typos, missing fields, and invalid values before runtime. A `scripts/validate-config.sh` script can be integrated into CI.

3. **Reusable defaults.** The `defaults:` block eliminates 7 instances of copy-pasted `external_directory` rules. Adding a new agent is a single YAML entry.

4. **Trigger-driven routing.** Structured `triggers` arrays enable automated task-to-agent routing. No more parsing English phrases from `description` fields.

5. **Forward-compatible.** Empty slots for `model`, `tools`, and `constraints` are ready for Multi-Model Task Routing (#41), tool restrictions, and token budgets — no schema changes needed when those features arrive.

6. **Machine-readable task definitions.** `config/tasks.yaml` makes workflows parseable and validateable. The Routine Templates table in `director.md` can be generated from this file rather than maintained by hand.

7. **Smaller agent context.** Once `.md` files drop their YAML frontmatter, the files are pure prose — slightly smaller and cleaner when loaded as agent context.

8. **Git diff clarity.** Changes to permissions, triggers, or model routing produce focused diffs in a short YAML file, not 200-line markdown diffs with hidden config changes.

### Negative

1. **Two-file synchronization burden.** Until OpenCode supports YAML config natively, changes must be made in two places: the `.md` frontmatter (for OpenCode to consume) and `config/agents.yaml` (for validation and tooling). A sync script mitigates but does not eliminate this.

2. **Migration effort.** All 8 agent files must have their frontmatter extracted into `agents.yaml`. The `.md` frontmatter must then be marked as deprecated (or removed once OpenCode catches up).

3. **New tooling dependency.** A YAML validator and JSON Schema processor become part of the toolchain. This adds `pyyaml` and `jsonschema` (or `ajv`) as validation dependencies.

4. **Potential drift.** If teams update `.md` frontmatter but forget to sync `agents.yaml`, the two sources diverge. CI validation catches this, but it adds a process step.

5. **Over-engineering risk.** For a project with only 8 agents, a full YAML config system might feel like overkill. However, the benefits (validation, triggers, forward compatibility) justify the investment.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **OpenCode never adds native YAML support** | Medium | Low | YAML already provides immediate value (see §8): JSON Schema validation, sync-enforcement in CI, single-source-of-truth for all 8 agents, structured triggers for tooling, and streamlined git diffs. OpenCode-native support is a Phase 6 bonus, not a prerequisite. The architecture is strictly better with YAML regardless of future OpenCode support. |
| 2 | **Two-file drift** | Medium | Medium | CI validation (`validate-config.sh`) compares `agents.yaml` against `.md` frontmatter and fails on mismatch. Sync script `diff` mode gates merges. |
| 3 | **Schema changes require updating both schema and sync script** | Low | Low | Schema and sync script live in the same repo and are updated together. |
| 4 | **Trigger overlap between agents** | Low | Low | CI validation checks for trigger keyword overlap and warns about ambiguity. A central trigger registry is documented in `config/agents.yaml` (see §6.1 Trigger Namespace Convention). |

### Follow-up Items

- [ ] Create `config/agents.yaml` with all 8 agents
- [ ] Create `config/agents.schema.json` for validation
- [ ] Create `config/tasks.yaml` with workflow and gate definitions
- [ ] Create `scripts/validate-config.sh` — validates `agents.yaml` against schema
- [ ] Create `scripts/sync-agent-config.sh` — syncs YAML frontmatter between `.md` files and `agents.yaml`
- [ ] Reduce `.md` files — remove YAML frontmatter, keep only prose
- [ ] Update `opencode.json` if needed (depends on OpenCode YAML support)
- [ ] Update design doc §10 (File Layout) to include `config/` directory
- [ ] Add CI step for config validation
- [ ] Add schema validation tests to `tests/init/`

## Migration Plan

### Phase 1: Create YAML Config Files (Engineers)

- [ ] Create `config/agents.yaml` with all 8 agents and `defaults:` block
- [ ] Create `config/agents.schema.json` for validation
- [ ] Create `config/tasks.yaml` with workflow and gate definitions
- [ ] Create `scripts/validate-config.sh` — validates YAML against JSON Schema
- [ ] Verify: `scripts/validate-config.sh` passes

**Files created:** `config/agents.yaml`, `config/agents.schema.json`, `config/tasks.yaml`, `scripts/validate-config.sh`

### Phase 2: Sync and Validate (Reviewers + Scribes)

- [ ] Run validation against all agent configurations
- [ ] Create `scripts/sync-agent-config.sh` — bidirectionally syncs frontmatter between `.md` and `agents.yaml`
- [ ] Verify sync script is idempotent (running twice produces the same result)
- [ ] Remove `Triggers:` suffixes from `description` fields in `.md` frontmatter — triggers are now a first-class YAML field
- [ ] Scribes updates design doc §10 (File Layout) with `config/` directory

**Files created:** `scripts/sync-agent-config.sh`
**Files modified:** `docs/design/README.md`

### Phase 3: Add Deprecation Notices (Scribes)

- [ ] Do NOT remove YAML frontmatter yet — OpenCode still requires it for agent loading
- [ ] Add deprecation notice at top of each `.md` file pointing to `config/agents.yaml`:
  ```markdown
  > **Note:** Agent configuration (permissions, model, triggers) is now
  > managed in `config/agents.yaml`. This file contains documentation only.
  ```
- [ ] The frontmatter remains synchronized via the sync script until native YAML support lands
- [ ] Future: When OpenCode supports YAML natively, remove frontmatter entirely

### Phase 4: Schema Validation Tests (Testers)

- [ ] Add tests in `tests/init/`:
  - YAML parses correctly
  - All agent names match expected set
  - All permission values are valid (`allow`, `deny`, `ask`)
  - JSON Schema validation produces expected errors for invalid input
  - Sync script is idempotent

### Phase 5: CI Integration

- [ ] Add `scripts/validate-config.sh` to CI pipeline (`kodehold-ci.yml`)
- [ ] Add `scripts/validate-config.sh` to gate.sh as a config check
- [ ] Optionally: add `validate-config` step to `scripts/ship.sh`

### Phase 6: Future — OpenCode Native YAML Support

When/if OpenCode adds native YAML agent configuration:

- [ ] Update `opencode.json` `agent.*.file` to point to `config/agents.yaml` entries (or remove `.md` references entirely)
- [ ] Remove sync script (no longer needed)
- [ ] Archive `.opencode/agents/*.md` (or keep as pure documentation)

## ADR References

- **ADR-0002** (Organizational Structure — Director and Teams) — defines the 6 team structure that this YAML config formalizes
- **ADR-0015** (Director Delegation Enforcement via Tool Permissions) — defines the permission model that this YAML schema captures
- **ADR-0032** (Routine Templates for Standard Flows) — defines the workflow concepts that `config/tasks.yaml` formalizes
- **ADR-0036** (Project Slug Convention) — defines naming conventions that the `name` field in `agents.yaml` follows
- **ADR-0007** (Token Optimization Strategy) — token-aware design: YAML is more compact than markdown frontmatter

### Source Files Referenced

- `.opencode/agents/*.md` — all 8 agent definitions whose frontmatter is extracted
- `opencode.json` — OpenCode's native agent configuration, which remains the primary loading mechanism
- `docs/design/README.md` — design doc §7.1 (OpenCode Compatibility), §10 (File Layout)
- `docs/adr/ADR-0032-routine-templates.md` — routine template definitions
