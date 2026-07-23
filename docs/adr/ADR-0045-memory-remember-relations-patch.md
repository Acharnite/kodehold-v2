# ADR-0045: Patch mem::remember to Create KV.relations Entry on Supersede

## Status

Accepted

**Version:** 1.1
**Last Updated:** 2026-06-06

## Context

### The Gap: Inconsistent Relation Tracking

Agentmemory has two code paths that create new versions of existing memories by superseding the old one:

1. **`mem::remember`** (called by the `memory_save` MCP tool) — automatically supersedes existing memories when Jaccard similarity > 70%. It sets `parentId`, `supersedes[]`, and increments `version` on the new memory, and marks the old one as `isLatest: false`. However, it does **not** create an entry in `KV.relations`.

2. **`mem::evolve`** (no MCP tool — internal SDK function) — does the same versioning AND also creates a `KV.relations` entry with `type: "supersedes"` (lines 6176-6184).

This inconsistency causes a visible gap: the `/agentmemory/relations` endpoint returns empty results for memories created via `memory_save`, and the KodeHold Viewer shows "Relations: 0" despite active versioning chains.

### Current Code

**`mem::remember` (lines 5666-5668)** — marks old memory as non-latest but creates no relation:

```javascript
if (supersededMemory) {
    supersededMemory.isLatest = false;
    await kv.set(KV.memories, supersededMemory.id, supersededMemory);
}
```

**`mem::evolve` (lines 6176-6184)** — creates a KV.relations entry:

```javascript
const relation = {
    type: "supersedes",
    sourceId: evolved.id,
    targetId: existing.id,
    createdAt: now,
    confidence: 1
};
const relationId = generateId("rel");
await kv.set(KV.relations, relationId, relation);
```

**`mem::get-related` (MCP: `memory_relations`)** — already reads from BOTH memory fields (`parentId`, `supersedes`) AND `KV.relations`, so adding the relation entry would make it visible in both the MCP tool AND the viewer.

### Key Forces

1. **Consistency.** Two functions that do the same thing (supersede a memory) should produce the same data structures. Currently `mem::remember` is missing the relation entry that `mem::evolve` creates.

2. **Discoverability.** The `/agentmemory/relations` endpoint and the KodeHold Viewer's "Relations" tab depend on `KV.relations` entries. Without them, version chains are invisible to users browsing relations.

3. **Minimal risk.** The patch is 4 lines that exactly mirror code already proven in production (`mem::evolve`). No new logic, no new dependencies.

4. **Upstream compatibility.** The fix should be submitted upstream to `rohitg00/agentmemory` so the inconsistency is resolved at the source.

## Decision

Patch `mem::remember` to create a `KV.relations` entry with `type: "supersedes"` when it supersedes an existing memory, matching `mem::evolve`'s behavior exactly.

### Implementation

Add 4 lines to `mem::remember` after line 5668 (after `await kv.set(KV.memories, supersededMemory.id, supersededMemory)`):

```javascript
const relation = {
    type: "supersedes",
    sourceId: memory.id,
    targetId: supersededMemory.id,
    createdAt: now,
    confidence: 1
};
await kv.set(KV.relations, generateId("rel"), relation);
```

**Patch location:** `/usr/local/lib/node_modules/@agentmemory/agentmemory/dist/src-D-5qPkWr.mjs` (and the equivalent in `index.mjs` if it exists).

### Resulting Code (after patch)

```javascript
if (supersededMemory) {
    supersededMemory.isLatest = false;
    await kv.set(KV.memories, supersededMemory.id, supersededMemory);
    const relation = {
        type: "supersedes",
        sourceId: memory.id,
        targetId: supersededMemory.id,
        createdAt: now,
        confidence: 1
    };
    await kv.set(KV.relations, generateId("rel"), relation);
}
```

### Why Not Use mem::evolve Instead?

`mem::evolve` is an internal SDK function with no MCP tool binding. Agents only have access to `memory_save` (which calls `mem::remember`). Adding an MCP tool for `mem::evolve` would be a larger change requiring OpenCode configuration updates. The 4-line patch to `mem::remember` is the minimal fix.

### Follow-up

Submit upstream PR to `rohitg00/agentmemory` with the same patch.

## Consequences

### Positive

1. **Consistent behavior.** `mem::remember` and `mem::evolve` both create `KV.relations` entries when superseding. No more silent inconsistency.

2. **Relations endpoint works.** `/agentmemory/relations` will now show supersedes chains for memories created via `memory_save`. Previously it returned empty for these memories.

3. **Viewer shows relations.** The KodeHold Viewer's "Relations" display will show version chains for all memories, not just those created via `mem::evolve`.

4. **`memory_relations` MCP tool finds them.** `mem::get-related` already reads from `KV.relations` (line 6206: `const allRelations = await kv.list(KV.relations)`), so the new entries are immediately discoverable via the MCP tool with no additional changes.

5. **Minimal diff.** 4 lines added, 0 lines changed. The patch is trivially reviewable.

### Negative

1. **Tiny storage increase.** Each supersede operation creates one additional `KV.relations` entry (~200 bytes). For typical usage patterns (a few hundred memory saves per session), this is negligible.

### Risks

1. **None.** `mem::evolve` already does this exact pattern in production. The patch is a copy-paste of proven code. No new failure modes are introduced.

2. **Ordering with cascade trigger.** The relation entry is written at line 5668, before the `mem::cascade-update` trigger fires at line 5683. This is the correct ordering — the relation exists in `KV.relations` before any cascade reads it.

### Neutral

1. **Local patch, not upstream (yet).** The patch is applied to the installed `node_modules` copy. It will be overwritten on `npm update`. The upstream PR is the permanent fix.

## Compliance

| Requirement | Verification |
|-------------|-------------|
| `mem::remember` creates KV.relations entry on supersede | Visual inspection of patched code |
| Relation entry matches `mem::evolve` format | Compare fields: type, sourceId, targetId, createdAt, confidence |
| `mem::get-related` finds the new entries | No code change needed — it already reads KV.relations |
| Viewer shows relations for memory_save memories | Open viewer, check Relations tab for a memory created via memory_save |

## Notes

### Related ADRs

- **ADR-0035** (Custom KodeHold Viewer) — the viewer's Relations tab depends on `KV.relations` data. This patch makes it work for all memories, not just those created via `mem::evolve`.

### Open Questions

1. **Should `mem::remember` also create a reverse relation?** `mem::evolve` only creates one direction (`sourceId: evolved.id, targetId: existing.id`). `mem::get-related` handles traversal in both directions by checking `sourceId` and `targetId` in its filter (line 6220). One entry is sufficient.

2. **Should we also patch `mem::forget` to clean up relations?** Currently, `mem::forget` deletes the memory from `KV.memories` but does not clean up related `KV.relations` entries. This is a pre-existing issue that affects both `mem::remember` and `mem::evolve` paths. Deferred — it is out of scope for this ADR.

3. **Should `mem::remember` add audit calls?** `mem::evolve` uses `safeAudit` for both the supersede mark and the relation creation. `mem::remember` currently has no audit calls. Adding them would be a larger change and is out of scope for this ADR, but the inconsistency is noted.

### Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.2 | 2026-06-06 | Promoted from Proposed → Accepted after Reviewers approval. |
| 1.1 | 2026-06-06 | Added cascade trigger ordering note to Risks. Added audit trail gap to Open Questions. |
| 1.0 | 2026-06-06 | Initial proposal |