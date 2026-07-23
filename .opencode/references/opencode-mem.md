# Memory Tools (opencode-mem)

All agents have access to opencode-mem MCP tools for persistent memory across sessions.

> **CRITICAL: Every `search_memories` and `add_memory` call MUST include `scope: "project"`.** KodeHold shares an opencode-mem instance with other agents. Without explicit project scoping, memories from other projects will bleed into KodeHold results. There are NO exceptions.

**Before starting work** — search for prior learnings:
```
search_memories(query="<topic>", scope="project")
```

**After completing work** — store what you learned:
```
add_memory(content="<learning>", scope="project")
```

Use `graphify query` for code retrieval. Use `search_memories` for runtime learnings and session context. They are complementary, not competing.
