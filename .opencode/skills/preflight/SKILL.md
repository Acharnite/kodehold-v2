# Skill: preflight

# Pre-Flight Knowledge Search

## Delegation Checklist (MANDATORY — complete BEFORE any action)

Before touching any file, writing any code, or making any change,
answer ALL three questions in your response:

```
❓ Hvad skal gøres?
❓ Hvilket team skal gøre det?  (architects / engineers / testers / reviewers / scribes / fls)
❓ Er Task tool kaldt?           (JA / NEJ — hvis NEJ, stop og delegér)
```

**Regel:** Hvis du er i tvivl, delegér. ALDRIG implementer, review,
test, eller dokumentér direkte.

---

Old Public Functions

Surfaces prior art before ANY delegation — code structure from
graphify + runtime learnings from opencode-mem + cross-reference
between both.

## Protocol (mandatory before every Task tool delegation)

Run BOTH searches independently. If either fails, log a warning
and continue — never block delegation on search failure.

### Step 1 — graphify: code/structural context

```
graphify query "<delegation-topic> <team>"
graphify query "<delegation-topic>"
```

> **Error handling:** If `graphify query` fails (timeout/error),
> log a warning and continue. Never block delegation on search failure.

Capture the output. Extract filenames and node names from the
graphify result — these are used as additional context in Step 2.

### Step 2 — search_memories: runtime learnings

```
search_memories(query="<delegation-topic> <team> lessons bugs", scope="project")
```

> **Error handling:** If `search_memories` fails (timeout/error),
> log a warning and continue. Never block delegation on search failure.

### Step 3 — Cross-reference (ADR-0024 enhancement)

Merge graphify filenames into the memory search for higher precision:

```
search_memories(query="<delegation-topic> <filnavne_fra_graphify>", scope="project")
```

This finds learnings that are specific to the EXACT files that
graphify identified as relevant — instead of just matching on
the delegation topic alone.

### Step 4 — Context assembly

Include in the Task prompt under a `Relevant Context` section:

```
Relevant Context:
<graphify query results>    # max 800 chars — top-2 most relevant snippets
Relevant Memories:
<search_memories results>   # max 800 chars — top-2 most relevant
```

### Priority topic matching

When delegation topic contains these keywords, always query with
the primary topic first:

| Task keyword | Query with |
|--------------|------------|
| "agent" / "agents" / "config" | `agent` |
| "design" / "doc" / "readme" | `design` |
| "adr" | `adr` |
| "version" / "release" / "changelog" | `version` |
| "plugin" / "capture" | `plugin` |
| "deploy" / "ship" / "gate" | `release` |

### Hotfix exemption

For P0/emergency situations, pre-flight may be skipped with
explicit user approval and logged reason.
