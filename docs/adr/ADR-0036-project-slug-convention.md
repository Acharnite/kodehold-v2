---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0036: Project Slug Convention — Stable Canonical Identifiers

## Status

Accepted

**Version:** 1.1
**Last Updated:** 2026-06-02
**Phase:** Infrastructure — supersedes ADR-0028 Section 6 (Accept Full Filesystem Path)

### Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.2 | 2026-06-04 | Phase 4 (Historical Data Migration) completed — 366 sessions, 6,552 observations migrated via Python iii-sdk |
| 1.1 | 2026-06-02 | Accepted after review and second opinion |
| 1.0 | 2026-06-02 | Initial proposal |

## Context

### The Problem

The agentmemory `project` field is used to scope all stored data — actions, sessions, memories, lessons, crystals, and signals. Currently, **4 different formats** are in active use across agentmemory records, creating an inconsistent and fragile data landscape:

| Format | Examples | Where Used |
|--------|----------|------------|
| **Full filesystem path** | `/home/kiffer/project/kodehold`, `/home/kiffer/project/bob` | Sessions, lessons (ADR-0028 legacy), agentmemory-capture plugin |
| **Simple slug** | `kodehold`, `bob` | Lessons (`lsn_58af4f9460976d9f`, `lsn_6bf3e4e7a22dc0db`), Director actions |
| **Relative path** | `workspaces/qbit-migrate`, `workspaces/my-app` | Gate scripts, lifecycle simulations |
| **Hex CID** | (various) | Agentmemory internal references |

This inconsistency causes four concrete problems:

1. **Cross-contamination of project scoping.** The same project (`kodehold`) appears under two different project keys (`/home/kiffer/project/kodehold` and `kodehold`). Queries scoped to one miss data stored under the other. Without cross-project queries, this means silent data loss — lessons stored under one key are invisible when querying under the other.

2. **Repo relocation breaks paths.** A full filesystem path like `/home/kiffer/project/kodehold` is tied to a specific machine layout. If the repo is cloned to `/home/other/projects/kodehold`, all historical session data becomes orphaned under the old path.

3. **Machine-dependent naming.** Two developers working on the same project will have different path prefixes (`/home/alice/project/kodehold` vs `/home/bob/work/kodehold`), making cross-machine data sharing (via agentmemory mesh sync) impossible without manual path rewriting.

4. **No formal specification.** There is no documented rule for what a valid project name looks like. The agentmemory `memory_save` tool description warns against filesystem paths and ad-hoc names, but no format has been formally adopted to replace them.

### Prior Art and Precedent

#### ADR-0028 (Accepted Full Filesystem Path)

ADR-0028 documented the decision to accept full filesystem paths as project names after 5 failed fix attempts. The key arguments were:

- **Uniqueness:** Full paths cannot collide (`/home/kiffer/project/kodehold` ≠ `/home/kiffer/project/bob`)
- **Zero configuration:** No env vars, no marker files, no git detection
- **Zero plugin divergence:** The agentmemory-capture plugin remains 100% aligned with upstream
- **Zero race conditions:** Determined at plugin load time

However, ADR-0028 itself acknowledged the negative consequences: *"Project directory rename orphans old sessions"* and *"No short-name aliasing."* This ADR addresses those weaknesses by adopting a stable slug convention that preserves uniqueness while adding portability.

#### Agentmemory Hooks (Canonical `resolveProject()`)

The agentmemory npm package ships CLI hooks at `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/hooks/` that implement the correct resolution logic:

```typescript
function resolveProject(cwd: string): string {
  const explicit = process.env["AGENTMEMORY_PROJECT_NAME"];
  if (explicit && explicit.trim()) return explicit.trim();
  const dir = cwd && cwd.trim() ? cwd : process.cwd();
  try {
    const top = execSync("git rev-parse --show-toplevel", {
      cwd: dir, stdio: ["ignore", "pipe", "ignore"], timeout: 500,
    }).toString().trim();
    if (top) return basename(top);
  } catch {}
  return basename(dir);
}
```

Resolution order: (1) `AGENTMEMORY_PROJECT_NAME` env var → (2) `basename(git rev-parse --show-toplevel)` → (3) `basename(cwd)`. This resolution order produces a **slug**, not a path. It is the correct algorithm — it was simply never wired into the OpenCode plugin path.

#### Agentmemory `memory_save` Tool Documentation

The `memory_save` tool description explicitly states:

> *"Stable canonical project identifier... Do not use filesystem paths or ad-hoc display names — those change across machines and will silently break project scoping."*

This is the authoritative statement from the agentmemory project itself. The full filesystem path approach (ADR-0028) is a pragmatic retreat that technically violates this guidance. This ADR aligns KodeHold with the documented contract.

#### Existing Slugs Already in Use

Despite ADR-0028's acceptance of full paths, several agentmemory records already use slugs:

| Slug | Records |
|------|---------|
| `kodehold` | Lessons `lsn_58af4f9460976d9f`, `lsn_6bf3e4e7a22dc0db` |
| `bob` | Project sessions |
| `flow-test` | Test sessions |

This demonstrates that slugs are already functional — ADR-0028's full-path decision was never fully adopted by all agents. This ADR formalizes what is already partially in practice.

### Key Forces

1. **Must be backwards-compatible.** Existing data under full paths must remain accessible during migration, and migration must not require bulk data rewrites.
2. **Must be deterministic.** The same project on any machine must resolve to the same slug (git-repo-name-based, not path-based).
3. **Must be unique.** Two different projects must never resolve to the same slug.
4. **Must interoperate with agentmemory ecosystem.** The slug format must be compatible with REST API query parameters, URL encoding, and agentmemory's internal storage.
5. **Must not break the agentmemory-capture plugin.** The plugin uses `ctx.worktree || ctx.project?.id || process.cwd()` and sends the full path. Slug resolution must happen at the daemon or MCP layer, not the plugin.
6. **Must align with the `memory_save` tool contract.** "Stable canonical project identifier" is the target.

## Decision

### Slug Format Specification

Adopt the following formal slug format for all agentmemory `project` values:

```
Format:  [a-z][a-z0-9-]*        (lowercase kebab-case)
Max:     50 characters
Pattern: /^[a-z][a-z0-9-]{0,49}$/
```

**Rules:**
- Must start with a lowercase letter (`[a-z]`)
- May contain lowercase letters, digits, and hyphens (`[a-z0-9-]`)
- Maximum 50 characters (agentmemory's internal limit for indexed fields)
- No uppercase letters (normalized to lowercase)
- No underscores (hyphens only for word separators)
- No dots, spaces, or special characters
- No trailing or leading hyphens
- No consecutive hyphens

**Validation function (TypeScript reference):**

```typescript
function validateSlug(slug: string): boolean {
  return /^[a-z][a-z0-9-]{0,49}$/.test(slug);
}
```

#### Enforcement Points

The `validateSlug()` function will be enforced at these boundaries:

| Layer | Enforcement | Behavior on Invalid |
|-------|-------------|-------------------|
| **MCP tool layer** | When `memory_save`, `memory_recall`, etc. receive a `project` parameter | Log a warning, fall through to the next resolution priority (env var → git basename → cwd basename). Reject the call if no resolution produces a valid slug. |
| **Migration script** | Before updating any agentmemory record | Skip the record, log the path and reason, continue processing. |
| **CI check** | `scripts/validate-slugs.sh` scans agent definitions, skill files, and MCP templates for hardcoded project values | Fail CI with a report of all non-slug project references. |

Invalid slugs are never silently accepted — they are always logged and, where possible, fall through to the next resolution step rather than hard-failing. This ensures forward progress even when slug resolution encounters an unexpected format.

**Examples of valid slugs:**
- `kodehold`
- `bob`
- `my-project`
- `qbit-migrate`
- `flow-test`
- `agentmemory-demo`

**Examples of invalid slugs:**
- `MyProject` (uppercase — normalize to `myproject`)
- `my_project` (underscore — use `my-project`)
- `-leading-hyphen` (leading hyphen)
- `trailing-hyphen-` (trailing hyphen)
- `double--hyphen` (consecutive hyphens)
- `a` (valid but too short to be meaningful — not prohibited, just discouraged)
- `` (empty — not allowed)

### Resolution Order

When determining the project slug for any agentmemory operation, use the following resolution order (matching agentmemory's own `resolveProject()` hooks):

| Priority | Source | Example | Notes |
|----------|--------|---------|-------|
| 1 | `AGENTMEMORY_PROJECT_NAME` env var | `kodehold` | Explicit override. Must be a valid slug per format spec. If invalid, fall through to next priority. |
| 2 | `basename(git rev-parse --show-toplevel)` | `kodehold` | Git repo root directory name. Works for any project in a git repo. Normalized to lowercase kebab-case. |
| 3 | `basename(cwd)` | `kodehold` | Fallback when not in a git repo. Uses the current working directory's basename. Normalized to lowercase kebab-case. |

**Normalization step (applied to sources 2 and 3):**

```typescript
function toSlug(name: string): string {
  let slug = name
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, '-')  // Replace invalid chars with hyphens
    .replace(/-+/g, '-')           // Collapse consecutive hyphens
    .replace(/^-+|-+$/g, '')       // Trim leading/trailing hyphens
    .slice(0, 50);                 // Truncate to 50 chars
  // Ensure slug starts with a lowercase letter (required by format spec).
  // Prepending "project-" handles names starting with digits or that collapsed
  // to empty after normalization (e.g. "123abc" → "project-123abc").
  if (!/^[a-z]/.test(slug)) {
    slug = 'project-' + slug;
  }
  return slug.slice(0, 50);
}
```

This normalization converts directory names like `My_Project` into valid slugs (`my-project`).

### Where Resolution Happens

Resolution of full paths to slugs happens at the **agentmemory daemon / MCP layer**, not in the plugin:

```
┌──────────────────────────────────────────┐
│  Plugin (agentmemory-capture.ts)          │
│  ── 100% upstream — sends full path       │
│  ── project="/home/kiffer/project/kodehold"│
└──────────────────┬───────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────┐
│  agentmemory Daemon (or MCP Server)       │
│  ── Resolves full path → slug             │
│  ── Resolution order:                     │
│      1. AGENTMEMORY_PROJECT_NAME env      │
│      2. basename(git top)                 │
│      3. basename(cwd)                     │
│  ── Stores slug as canonical project      │
└──────────────────┬───────────────────────┘
                   │ project="kodehold"
                   ▼
┌──────────────────────────────────────────┐
│  All agents (Director, Engineers, etc.)   │
│  ── Use project: "kodehold" in MCP calls │
│  ── No filesystem paths ever              │
└──────────────────────────────────────────┘
```

**This means the agentmemory-capture plugin remains untouched** — it continues sending the full path. The daemon (or a thin middleware layer) normalizes it to a slug before storage. This preserves ADR-0028's guarantee of *"Zero plugin divergence"* while fixing the machine-dependency problem.

### Where Slugs Must Be Used

All agentmemory MCP tool calls that accept a `project` parameter must use the slug:

| Tool Call | Field | Example |
|-----------|-------|---------|
| `agentmemory_memory_save(content=..., project=..., ...)` | `project` | `project="kodehold"` |
| `agentmemory_memory_recall(query=..., project=...)` | `project` | `project="kodehold"` |
| `agentmemory_memory_action_create(title=..., project=...)` | `project` | `project="kodehold"` |
| `agentmemory_memory_lesson_recall(query=..., project=...)` | `project` | `project="kodehold"` |
| `agentmemory_memory_lesson_save(content=..., project=...)` | `project` | `project="kodehold"` |

Agent definitions, skill files, and scripts must use the slug, not the full path. The one exception is the agentmemory-capture plugin, which continues sending the full path (the daemon normalizes it).

### Migration Mapping

For the transition period, the following mapping defines how existing records under old formats map to the target slug:

#### From Full Filesystem Path

| Current Path | Target Slug | Status |
|-------------|-------------|--------|
| `/home/kiffer/project/kodehold` | `kodehold` | ✅ Migrated — 317 sessions |
| `/home/kiffer/project/bob` | `bob` | ✅ Migrated — 120 sessions |
| `/home/kiffer/project/bob-ollama` | `bob-ollama` | ✅ Migrated — 2 sessions |
| `/home/kiffer/project` (workspace root) | `kodehold` | ✅ Migrated — 63 sessions reassigned to kodehold |
| `/tmp/agentmemory-demo` | `kodehold` | ✅ Migrated — 3 demo sessions reassigned to kodehold |
| Any other full path | `basename(path)` → toSlug() | Dynamic mapping |

#### From Already-Slug or Relative Path

| Current Identifier | Target Slug | Status |
|--------------------|-------------|--------|
| `kodehold` | `kodehold` | Already correct |
| `bob` | `bob` | Already correct |
| `flow-test` | `flow-test` | Already correct |
| `workspaces/qbit-migrate` | `qbit-migrate` | Normalize — strip directory prefix |
| `workspaces/my-app` | `my-app` | Normalize — strip directory prefix |

#### Orphaned Sessions: `orphaned-workspace-root`

The workspace root path (`/home/kiffer/project`) represents 63 sessions that cannot be confidently assigned to any sub-project. These sessions are assigned the reserved slug `orphaned-workspace-root` (the "orphaned-" prefix signals a synthetic/non-canonical project). This slug is:

- **Read-only** — no new sessions or memories should be created under `orphaned-workspace-root`
- **Preserved** — existing sessions remain queryable under this slug for historical reference
- **Flagged** — any code that references `orphaned-workspace-root` should log a warning

### Supersedes Note

If accepted, this ADR will supersede **Section 6 of ADR-0028** (the "Accept Full Filesystem Path as Project Name" decision).

#### ADR-0028 Section 6 in Summary

ADR-0028 Section 6 (its **Decision** section) documented the choice to accept the full filesystem path as the project name. This was the result of a long struggle: **5 prior fix attempts** had been made to derive short project names, and all had been reverted. The approaches ranged from walk-up directory scoring with marker files (Fix 1), to `.kodehold-project` marker files (Fix 2), to fragile regex-based tool-call scanning (Fixes 3-4), to per-session `info?.directory` extraction (Fix 5).

ADR-0028's decision was a **pragmatic retreat**, not a principled one. The ADR's own words acknowledged the downsides: *"Project directory rename orphans old sessions"* and *"No short-name aliasing."* The decision traded long-term portability for short-term stability — and it correctly identified that the **agentmemory-capture plugin must not diverge** from the upstream GitHub version.

#### What This ADR Changes

ADR-0036 maintains ADR-0028's key guarantee — **zero plugin divergence**. The agentmemory-capture plugin continues to send full filesystem paths, exactly as upstream designed. No plugin changes, no maintenance burden, no merge conflicts.

The critical difference is **where slug resolution happens**:

| Aspect | ADR-0028 | ADR-0036 |
|--------|----------|----------|
| **Resolution layer** | None (plugin sends path, daemon stores path) | Daemon / MCP layer normalizes path → slug |
| **Plugin behavior** | Sends full path | Sends full path (unchanged) |
| **Stored project value** | `/home/kiffer/project/kodehold` | `kodehold` |
| **Portability** | Machine-dependent | Machine-independent |
| **Driver** | agentmemory hooks were not wired in | agentmemory hooks now provide `resolveProject()` |

What changed: agentmemory itself now ships `resolveProject()` hooks (at `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/hooks/`) that implement the correct slug resolution algorithm. The hooks were available at the time of ADR-0028 but were not wired into the OpenCode plugin path. This ADR leverages them at the daemon/MCP layer instead.

#### What Remains from ADR-0028

- ADR-0028's **Architecture** diagram is updated — the plugin continues to send full paths, but the daemon layer now normalizes to slugs
- ADR-0028's **Positive consequences** #1 (Zero maintenance) and #2 (Zero configuration) are preserved — the plugin remains untouched
- ADR-0028's **Negative consequences** #1 (Long paths) and **Risks** #1 (Directory rename orphans sessions) are resolved by slug adoption
- The rest of ADR-0028 remains valid: the plugin-level analysis, the 5 failed fix attempts, and the "why prior approaches were rejected" section

## Consequences

### Positive

1. **Machine-independent identifiers.** The slug `kodehold` is the same on every machine, regardless of where the repo is cloned. Agentmemory mesh sync works across machines without path rewriting.

2. **Repo relocation safe.** Moving the repo from `/home/kiffer/project/kodehold` to `/home/other/projects/kodehold` does not change the project slug (`kodehold`). Historical data remains accessible.

3. **Short, readable names.** `kodehold` (8 chars) vs `/home/kiffer/project/kodehold` (30 chars). Less visual noise in logs, UIs, and code.

4. **Aligned with agentmemory tool contract.** The slug format satisfies the `memory_save` requirement for *"Stable canonical project identifier... Do not use filesystem paths."*

5. **Aligned with upstream `resolveProject()` hooks.** The resolution order (env var → git toplevel basename → cwd basename) is the same algorithm agentmemory ships. KodeHold is converging on upstream design.

6. **Zero plugin divergence preserved.** The agentmemory-capture plugin remains 100% upstream — it continues sending full paths. Slug resolution happens at the daemon layer, requiring no plugin modifications.

7. **Uniqueness preserved.** Git repo names are unique within a workspace. The git toplevel basename approach guarantees that `kodehold` and `bob` produce different slugs. Collision only happens if two different repos have the same name — an exceedingly rare case that can be resolved via `AGENTMEMORY_PROJECT_NAME` env var.

8. **Formal specification available for validation.** The regex `/^[a-z][a-z0-9-]{0,49}$/` can be used in validation scripts, CI gates, and agent tool call validation to enforce slug format.

9. **Consolidates 4 formats into 1.** Eliminates the fragmentation of filesystem paths, relative paths, ad-hoc slugs, and hex CIDs. One format to learn, validate, and maintain.

### Negative

1. **Short-name collisions are possible (rare).** If two repos are named `my-project` in different workspaces, they will collide. Mitigation: use `AGENTMEMORY_PROJECT_NAME=my-project-v2` to disambiguate. In practice, repo names are distinct within a developer's workspace.

2. **Normalization loss.** Directory names like `Bob-Ollama` normalize to `bob-ollama`. If the directory is later renamed to `BobOllama`, the slug changes to `bobollama`. Mitigation: use `AGENTMEMORY_PROJECT_NAME` to pin the slug regardless of directory name.

3. **Historical data under full paths has been migrated.** Sessions stored under `/home/kiffer/project/kodehold` were migrated to `kodehold` via the Phase 4 migration script (2026-06-04). A dual-query strategy is no longer needed.

4. **Orphaned workspace-root sessions.** 63 sessions under `/home/kiffer/project` cannot be assigned to a sub-project. They survive under `orphaned-workspace-root` but are not scoped to any active project. This is a permanent loss of project scoping for those sessions.

5. **Daemon-layer changes required.** If slug resolution is implemented at the daemon level, agentmemory must be configured or patched to normalize paths. If implemented at the MCP level, the MCP server must be updated. Either way, a downstream component changes.

6. **No enforcement mechanism (initially).** This ADR defines the convention but does not implement enforcement. Agent tool calls can still pass full paths. Enforcement must come via validation gates in a follow-up phase.

### Risks

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Collision: two repos with same basename** | Low | Medium | `AGENTMEMORY_PROJECT_NAME` env var disambiguates. Slug uniqueness is not guaranteed by format alone — only by git repo naming convention. |
| 2 | **Plugin continues sending full path; daemon never normalizes** | Medium | High | If slug resolution is not implemented at daemon/MCP layer, full paths persist. Mitigation: add validation gate that warns when `project` field matches a filesystem path pattern. |
| 3 | **Existing lessons/sessions under full paths become invisible** | Medium | Medium | Dual-query strategy during transition. After all agents migrate to slugs, old data is historical only — new reads use slugs. |
| 4 | **Agent tool calls bypass slug convention** | Low | Medium | All agent tool calls that accept `project` must be updated. Mitigation: add `project` format validation to tool call templates. |
| 5 | **Workspace project slugs collide with root project slugs** | Low | Low | Workspace projects use basename of their subdirectory. Git repo names are distinct. Collision only if a workspace dir has the same name as a root project. Use AGENTMEMORY_PROJECT_NAME to resolve. |
| 6 | **Migration script corrupts agentmemory data** | Low | High | The migration script (Phase 4) creates a full database snapshot before any modification, logs every record change to a timestamped audit file, and supports a `--restore` command that reloads from the backup snapshot. See [Rollback Procedure](#rollback-procedure) for details. |

### Follow-up Items

- [ ] Implement daemon-level or MCP-level slug resolution for the agentmemory-capture plugin's full path input
- [ ] Update all agent definitions to use slug format for `project` parameters
- [ ] Update `agentmemory-knowledge-flow` skill to use slugs instead of full paths
- [ ] Add slug validation gate to `scripts/gate.sh` — warn if any tool call uses a filesystem-path project name
- [ ] Document the slug convention in `docs/design/README.md` Section 7.2 (Project Scoping)
- [ ] Add slug format reference to `docs/adr/README.md` ADR index
- [ ] Consider adding `AGENTMEMORY_PROJECT_NAME` to `.env.example` for new project setup
- [x] Create migration script `scripts/migrate-project-slugs.sh` for one-time batch update of existing agentmemory records — executed via Python iii-sdk (see scripts/migrations/slug-migration-20260604.log)
- [ ] Create `scripts/validate-slugs.sh` CI check for hardcoded project values in agent definitions and skill files (remaining)
- [ ] Add `toSlug()` normalization utility to `scripts/` for reuse across validation and migration tools

## Migration Plan

### Phase 1: Convention Adoption (This ADR)

- [x] Publish ADR-0036 defining the slug format, resolution order, and migration mapping
- [ ] Update ADR index in `docs/adr/README.md`
- [ ] Update design doc Section 7.2 (Project Scoping)

### Phase 2: Agent Definition Updates

- [ ] Update all 8 agent definitions to use slug project names in `memory_save`, `memory_recall`, `memory_action_create`, etc.
- [ ] Update `agentmemory-knowledge-flow/SKILL.md` to reference slugs in example calls
- [ ] Update Director's delegation loop to use slugs

### Phase 3: Daemon/MCP Layer Slug Resolution

- [ ] Implement path-to-slug normalization in agentmemory daemon or MCP server
- [ ] Verify plugin full paths are correctly normalized to slugs
- [ ] Add validation that rejects paths that don't match slug pattern at the MCP layer

### Phase 4: Historical Data Migration

This phase migrates existing agentmemory records (actions and sessions) from legacy project formats to the canonical slug. This is a **one-time backend data normalization** — users see no interface changes; the plugin still sends paths, and the daemon normalizes them going forward.

#### Migration Script: `scripts/migrate-project-slugs.sh`

A dedicated shell script performs the one-time batch update:

1. **Backup** — Creates a full database snapshot via `agentmemory_memory_snapshot_create` before any modification
2. **Dry-run** — Reports what will change without applying it (`--dry-run` flag)
3. **Map** — Reads the [Migration Mapping](#migration-mapping) table in this ADR to translate each legacy path/identifier → target slug
4. **Apply** — Updates each record's `project` field to the target slug, logging every change to a timestamped audit file at `scripts/migrations/slug-migration-<timestamp>.log`
5. **Restore** — Supports `--restore` to reload from the backup snapshot if needed

**Key design principles:**
- **One-time operation**: Not a live migration daemon. Run once, verify, archive.
- **No tooling migration**: The plugin still sends full paths; the daemon layer normalizes them going forward. Users and agents see no interface changes.
- **Preserves orphaned sessions**: Sessions under `/home/kiffer/project` (workspace root) are mapped to `orphaned-workspace-root`, preserving historical data without assigning it to any active project.
- **Idempotent**: Running the script multiple times on already-migrated records is safe — the mapping produces the same slug from the same path.
- **Validates at every step**: Each record's target slug is checked against `validateSlug()` before update. Invalid slugs are logged and skipped.

#### Dual-Query Bridge (Post-Migration)

After migration, a lightweight dual-query strategy runs during the transition period:
- For each project, query both `project="/home/kiffer/project/kodehold"` (legacy) and `project="kodehold"` (slug)
- Merge results to ensure no sessions are lost during the cutover
- Retire the dual-query once all agents have adopted the slug format (Phase 5)

#### Rollback Procedure

If the migration causes issues:

```bash
# 1. Restore from backup snapshot
./scripts/migrate-project-slugs.sh --restore

# 2. Verify restoration
agentmemory_memory_recall(query="*", project="/home/kiffer/project/kodehold")

# 3. Investigate and fix the issue
# 4. Re-run migration after fix
./scripts/migrate-project-slugs.sh
```

The audit log (`scripts/migrations/slug-migration-<timestamp>.log`) contains every record change with before/after values, enabling manual rollback of individual records if needed.

### Phase 5: Enforcement

- [ ] Add CI gate that scans agent tool calls for filesystem-path project values
- [ ] Add `scripts/validate-slugs.sh` CI check that scans agent definitions, skill files, and MCP templates for non-slug project references; fails CI with a report
- [ ] Block agent definitions that use non-slug project values
- [ ] Remove full-path project support from all agent-facing interfaces

## ADR References

- **ADR-0028** (Agentmemory Project Detection Strategy) — **Superseded** (Section 6: Accept Full Filesystem Path). This ADR replaces the full-path decision with a formal slug convention.
- **ADR-0029** (ICM → Agentmemory Migration Strategy) — established agentmemory as the primary memory system; slug convention ensures stable scoping
- **ADR-0030** (Agentmemory Knowledge Flow) — defines `project` parameter usage across knowledge flow steps; must be updated to use slugs
- **ADR-0031** (Actions + Crystals) — actions use `project` field; must be updated to use slugs
- **ADR-0012** (Adopted Projects) — adopted projects under `workspaces/` use relative paths; must be migrated to slugs (e.g., `qbit-migrate`)
- **Lesson `lsn_1cb3ac318435645c`** (confidence 0.9) — documented `info?.directory` approach from session.created event for plugin project detection
- **Lesson `lsn_1c753d4ac1088621`** (confidence 0.9) — established "revert and ADR/design process" approach for project detection fixes
- **Lesson `lsn_58af4f9460976d9f`** (confidence 0.9) — stored under slug `kodehold`, demonstrating existing slug usage alongside full paths

### Source Files Referenced

- `docs/adr/ADR-0028-agentmemory-project-detection.md` — superseded Section 6
- `docs/adr/README.md` — ADR index to update
- `docs/design/README.md` — Section 7.2 (Project Scoping) to update
- `.opencode/skills/agentmemory-knowledge-flow/SKILL.md` — `project` parameter usage
- `.opencode/agents/*.md` — all 8 agent definitions with project references
- `workspaces/.catalog` — adopted project catalog (relative paths to convert)
- `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/hooks/session-start.mjs` — canonical `resolveProject()` implementation
