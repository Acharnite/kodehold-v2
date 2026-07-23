# KodeHold Protocol — Shared Reference

## CLI Operations

Use OpenCode's native tools for all file and code operations:
- `glob` for file pattern matching
- `grep` for content search
- `read` for file reading
- `bash` for shell commands

## Inter-Agent Communication

When work requires a different team:

```
### Suggested next agent
Agent: <team-name>
Reason: <why this team is needed>
Context: <what context to pass>
```

## Persistent Memory (opencode-mem)

All persistent memory is stored via opencode-mem MCP tools (`add_memory`, `search_memories`).
Every `add_memory` and `search_memories` call MUST include `scope: "project"`.

Use tags to categorize: `decision`, `pattern`, `bug`, `lesson`, `metrics`, `prospective`, `release`.

Examples:
- Store a decision: `add_memory(content="...", tags=["decision"], scope="project")`
- Store a bug finding: `add_memory(content="...", tags=["bug"], scope="project")`
- Search prior work: `search_memories(query="<topic>", scope="project")`

## Quality Gate Checklist

Before transitioning between lifecycle states, verify:
- [ ] Design doc is current and approved
- [ ] ADRs cover all significant decisions
- [ ] Code matches design doc specs
- [ ] Tests exist and pass
- [ ] Memory up to date (relevant context stored via `add_memory`)

## Shipping Gate Checklist

Before every push, PR, or release, verify:
- [ ] VERSION.md updated with bump rationale
- [ ] CHANGES.md entry added (version + date + structured changes)
- [ ] TODO.md: completed items marked `[x]`, follow-ups added
- [ ] Test suite green: `bash tests/run.sh` — all pass
- [ ] Release summary stored via `add_memory(tags=["release"], scope="project")`
- [ ] Commit message follows `<type>(<scope>): <description>` format
- [ ] PR created if on feature branch (`gh pr create`)
- [ ] Tag applied for releases (`git tag v<version> && git push origin v<version>`)

### Blockers

Ship is BLOCKED if:
- Any test fails (smoke / init / integration)
- VERSION.md or CHANGES.md not updated
- Design doc differs from implementation without an ADR
- Release summary not stored via `add_memory`
