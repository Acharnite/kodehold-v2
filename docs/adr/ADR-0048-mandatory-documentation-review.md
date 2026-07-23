---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0048: Mandatory Tool Documentation Review Before Implementation

## Status

Accepted

## Context

### The Problem

The **deepresearch** project selected LiteLLM as its LLM-routing dependency but implemented it incorrectly. The implementation used assumptions about LiteLLM's API (e.g., passing bare model names like `gpt-4o` instead of the required `openrouter/gpt-4o` provider prefix), leading to runtime failures, wasted tokens on debugging, and significant rework.

The root cause was not a bad tool selection — it was that **nobody read the official LiteLLM documentation before writing code that used it**. The existing process only requires researching technology *options* during selection; there is no step that requires reading the *official documentation* of the selected tool before implementation.

### Current Safeguards and Their Gaps

| Safeguard | What It Covers | Gap |
|-----------|---------------|-----|
| **"Research before designing"** (architects.md, lines 82-83) | Technology options, prior art, best practices during selection | Does NOT require reading official docs of the *selected* tool |
| **Agentmemory Knowledge Flow** (ADR-0030) | Recalling past decisions and lessons from internal memory | Internal memory has no substitute for external, up-to-date official docs |
| **Knowledge Recall Protocol** (ADR-0038) | Internal knowledge retrieval from agentmemory | Same gap — internal recall cannot replace external documentation |
| **Pre-Flight Knowledge Check** (ADR-0039) | Checking agentmemory before delegation | Checks *internal* memory only — does not require reading *external* documentation |

All four safeguards operate on *internal knowledge* (prior sessions, lessons, ADRs). None requires agents to consult the *external, authoritative source* — the official documentation of the tools they are about to use.

### Why This Gap Exists

The existing "Research before designing" step (architects.md line 82-83) reads:

> *"Research before designing — use `webfetch` and `websearch` to research technology options, prior art, and best practices before making architectural decisions."*

This step was written to inform **selection** (which tool to choose), not **correct usage** (how to use the chosen tool correctly). Once a tool is selected, there is no process step that says "now read its official documentation."

### Key Forces

1. **LLMs have training data cutoffs.** An LLM may have incomplete or outdated knowledge of a tool's API, especially for rapidly evolving projects. Official documentation is the only ground truth.
2. **Tool documentation changes.** What was true in training data may no longer be true. Official docs reflect the current version.
3. **All teams touch tool APIs.** Engineers implement against them, Testers mock/stub them, FLS hotfixes them, Reviewers evaluate correctness. All need the same ground truth.
4. **Low time cost.** Reading the relevant sections of official docs typically takes 10-30 minutes — negligible compared to the cost of debugging an incorrect implementation.
5. **High rework cost.** The deepresearch LiteLLM incident cost multiple debugging sessions, wasted API calls, and a full re-implementation cycle. All of this was preventable with a 15-minute documentation read.
6. **ADR value increases.** ADRs that link to official documentation become more useful references for future reopenings and code reviews.
7. **Reviewers need a reference.** Without official docs to validate against, reviewers can only check implementation against the design doc and general knowledge — not against the tool's actual API contract.

## Decision

### 1. Documentation Review Requirement

For EVERY external dependency, library, framework, or tool selected in an ADR or design decision, the official documentation MUST be:

| Step | Action | Owner | Gate |
|------|--------|-------|------|
| **Identify** | Find the official documentation URL (e.g., `https://docs.litellm.ai`) | Architects (during ADR writing) | ADR must contain the URL |
| **Read** | Read the relevant sections before writing any code that uses the tool | Implementing agent (Engineers, FLS, Testers) | Implementation must reflect documented API patterns |
| **Reference** | Include a "Documentation" section in the ADR with links and key API concepts | Architects (during ADR writing) | Reviewer checks for this section |

### 2. Workflow Integration by Phase and Team

| Phase | Team | Action |
|-------|------|--------|
| **ADR writing** | Architects | After selecting a technology, add a "Documentation" section to the ADR with official doc links, key API concepts, version, and gotchas |
| **Before implementation** | Engineers | Read the linked official docs for ALL dependencies in scope before writing any code. If the ADR lacks a Documentation section, flag it to Director |
| **Before testing** | Testers | Read API documentation to understand correct mock/stub contracts and expected request/response formats before writing test fixtures |
| **Before code review** | Reviewers | Verify that the implementation matches official documentation patterns — not just the design doc. If implementation contradicts official docs, BLOCK the review |
| **Before hotfix** | FLS | Read relevant doc sections for the affected dependency before applying changes. If the dependency is unfamiliar, do a full doc read — not just a skim |
| **Before reopening** | Architects | Re-read relevant doc sections for any dependencies whose APIs may have changed since the project was closed. Update the ADR's Documentation section if needed |

### 3. ADR Documentation Section Format

Every ADR that selects or adopts a tool MUST include a "Documentation" section with the following fields:

```markdown
## Documentation

| Field | Value |
|-------|-------|
| **Tool** | <name of tool/dependency> |
| **Official docs** | <URL to official documentation> |
| **Version documented** | <version number or range> |
| **Key sections read** | <list of sections, chapters, or pages> |
| **Key API concepts** | <brief summary: authentication, endpoint patterns, configuration requirements, known gotchas, version-specific behaviors> |
| **Configuration prerequisites** | <environment variables, config files, service accounts, etc.> |
```

The "Key API concepts" field is particularly important — it captures the mental model a developer needs to use the tool correctly. This includes:

- Required API patterns (e.g., "provider prefix required in model name")
- Authentication requirements
- Configuration file formats
- Rate limiting or quota considerations
- Version-specific breaking changes
- Known pitfalls or gotchas documented by the tool authors

#### When the Documentation Section is NOT Required

- Tools used only incidentally (e.g., `requests` library for a simple HTTP call) — if the usage is trivial and well-understood, the section is optional at the implementer's discretion
- Language standard libraries — assumed baseline knowledge
- Tools already documented in a previous ADR (cross-reference with `See ADR-NNNN for Documentation`)

#### When Documentation is Unavailable or Incomplete

If the selected tool lacks official documentation, or the documentation is clearly outdated:

1. **Document the gap** in the ADR's Documentation section: `No official documentation available as of <date>. Reference: <alternative source like GitHub README, source code, community wiki>.`
2. **Flag the risk** to the Director: "This tool has no official documentation — implementation carries higher discovery risk."
3. **Consider alternatives** during ADR review: a tool without documentation may not be suitable for production use.

### 4. Enforcement

Enforcement is a shared responsibility across two review stages:

#### 4.1 ADR Review Enforcement (Reviewers)

| Check | Criterion | Action |
|-------|-----------|--------|
| Documentation section exists | Every ADR that selects a tool has a `## Documentation` section | PASS / BLOCK |
| Documentation section is complete | All fields populated (Tool, Official docs, Version, Key sections, Key API concepts) | Request fill-in for missing fields |
| Key API concepts are specific | Not generic boilerplate — contains tool-specific patterns (e.g., "provider prefixes required" vs. "it's an API") | Request specific details |
| Version documented | Version or version range is stated | Request version clarification |
| Gotchas captured | Known pitfalls from the docs are listed | Request gotcha summary |

**If the Documentation section is missing entirely → BLOCK the ADR.** Return to Architects with: "ADR-NNNN is missing the mandatory Documentation section (per ADR-0048). Architecture Decision Records that select external tools must document their official documentation reference."

#### 4.2 Code Review Enforcement (Reviewers)

| Check | Criterion | Action |
|-------|-----------|--------|
| API patterns match docs | Implementation uses documented API patterns (endpoints, parameter names, config structure) | PASS / BLOCK |
| Configuration matches docs | Config files, environment variables, and initialization follow documented format | Request fix if mismatch |
| Error handling matches docs | Error types, retry logic, and fallback behavior align with documented error responses | Request fix if mismatch |
| Version constraints match | Dependency version in `requirements.txt`, `package.json`, etc. is within the documented version range | Request update if mismatch |

**If the implementation contradicts official documentation → BLOCK the code review.** Return to Engineers with: "Implementation at <file:line> uses `<actual pattern>` but the official documentation specifies `<expected pattern>`. See ADR-NNNN Documentation section: <link>. Fix to match documented API."

#### 4.3 Escalation Path

- **ADR blocked:** Director routes back to Architects for Documentation section addition
- **Code review blocked:** Director routes back to Engineers for reimplementation per docs
- **Disputed interpretation:** If the implementer believes the documentation is wrong or ambiguous, they escalate to Architects. Architects resolve via the Second Opinion protocol (ADR-0006, ADR-0017)

### 5. Integration into Team Agents

Each team agent file must be updated to include documentation review as a mandatory workflow step.

#### 5.1 Architects (architects.md)

**Update "Research before designing" (current line 82-83):**

Current:
```
2. **Research before designing** — use `webfetch` and `websearch` to research technology options, prior art, and best practices before making architectural decisions. Document findings in the ADR Context section
```

New:
```
2. **Research before designing** — use `webfetch` and `websearch` to research technology options, prior art, and best practices before making architectural decisions. Document findings in the ADR Context section
3. **Document selected tools** — after selecting a technology in an ADR, identify its official documentation (URL, version) and add a `## Documentation` section (per ADR-0048 Section 3). Include key API concepts, configuration prerequisites, and known gotchas
```

**Renumber subsequent steps** (old 3 → new 4, etc.).

#### 5.2 Engineers (engineers.md)

**Insert after current step 2 ("Read all relevant ADRs for architectural context"):**

```
2b. **Read official documentation of selected tools** — for every external dependency referenced in the ADR's Documentation section (per ADR-0048), read the linked official documentation before writing any code. Pay attention to: API patterns, configuration requirements, authentication, and version-specific behaviors.
     - If the ADR is missing a Documentation section for a tool you are implementing, flag this to Director: "ADR-NNNN is missing the Documentation section per ADR-0048 — cannot implement safely."
     - If official docs contradict your assumptions, follow the documented API — not your assumption.
```

#### 5.3 Testers (testers.md)

**Insert after current step 2 ("Read the code under test"):**

```
2b. **Read API documentation** — for any external dependency whose API you are mocking, stubbing, or testing, read the relevant sections of its official documentation (per the ADR's Documentation section per ADR-0048). Ensure mocks/stubs match the documented request/response contracts, not assumptions from the implementation.
```

#### 5.4 Reviewers (reviewers.md)

**Add to the Review Checklist:**

```
- [ ] ADR Documentation sections exist and are complete (per ADR-0048) — all ADRs that select tools have a `## Documentation` section with URL, version, key concepts, and gotchas
- [ ] Implementation matches official tool documentation — verify API patterns, config, error handling against the docs linked in the ADR's Documentation section
```

#### 5.5 FLS (fls.md)

**Insert after current step 3a ("Load context: read design doc, relevant ADRs, affected code"):**

```
3a-ii. **Read official documentation** — before applying a hotfix to code that uses an external dependency, read the relevant sections of that dependency's official documentation (linked from the ADR's Documentation section per ADR-0048). For unfamiliar dependencies, do a full read of the key sections — not just the affected area.
```

### 6. Concrete Example

The following illustrates what the LiteLLM ADR for deepresearch **would have included** under this policy:

```markdown
## Documentation

| Field | Value |
|-------|-------|
| **Tool** | LiteLLM |
| **Official docs** | https://docs.litellm.ai |
| **Version documented** | v1.45+ |
| **Key sections read** | Provider Routing, Model Configuration, Cost Tracking, Error Handling |
| **Key API concepts** | |
| | - Endpoint: `/v1/chat/completions` (OpenAI-compatible) |
| | - Provider prefixes REQUIRED in model names: `openrouter/gpt-4o`, `anthropic/claude-3`, `gemini/gemini-pro` |
| | - Models configured via `model_list` in YAML or `LITELLM_MODEL_LIST` env var |
| | - Cost tracking via `litellm.success_callback=["langsmith"]` or `["prometheus"]` |
| | - Rate limiting: configure RPM/TPM per model in `router_settings` |
| | - Error types: `RateLimitError`, `ContextWindowExceededError`, `AuthenticationError` — each with specific retry behavior |
| | - **Gotcha:** Provider prefix is silently dropped if invalid — no error, but request routes to wrong provider |
| **Configuration prerequisites** | `OPENROUTER_API_KEY` (or equivalent provider key), `LITELLM_MASTER_KEY` for proxy auth |
```

This Documentation section would have surfaced the **provider prefix requirement** (`openrouter/gpt-4o` not `gpt-4o`) before any code was written, preventing the entire class of bugs that the deepresearch team encountered.

## Alternatives Considered

### Option 1: Trust the Agent's General Knowledge (Rejected)

Rely on LLM training data to provide correct API usage patterns for selected tools.

**Positive:**
- Zero process overhead
- No ADR format changes
- Works well for well-known, stable tools

**Negative:**
- LLMs have training data cutoffs — may not know recent API changes
- LLMs hallucinate API patterns under ambiguity (producing plausible but incorrect code)
- No single source of truth to reference during code review
- Deepresearch LiteLLM incident demonstrates the failure mode concretely

**Why rejected:** The LiteLLM incident proves this approach fails. LLM knowledge of tool APIs is probabilistic, not authoritative. Official documentation is the only ground truth.

### Option 2: Add Documentation Reading Only to Architects (Rejected)

Require Architects to read docs and summarize key patterns in the ADR, but do not require Engineers, Testers, or FLS to read docs directly.

**Positive:**
- Single point of responsibility
- ADRs become richer references
- Engineers can work from the ADR summary

**Negative:**
- Architecture summaries are second-hand knowledge — errors in the summary propagate to implementation
- Engineers working at the API level need first-hand understanding of request/response contracts
- Testers need to understand mock contracts independently — a summary is insufficient
- FLS may encounter edge cases not covered in the summary

**Why rejected:** Second-hand knowledge degrades. Engineers and Testers both work at the API surface and need direct documentation access. The Architects' summary is a *supplement* to reading the docs, not a *substitute*.

### Option 3: Auto-Generate Documentation References from Code (Rejected)

After implementation, extract dependency usage patterns and generate documentation references automatically.

**Positive:**
- Zero additional work for agents
- Leverages the actual implementation as documentation
- Catches mismatches retroactively

**Negative:**
- Circular dependency: you can't auto-generate correct usage documentation from incorrect code
- Cannot identify what the code **should** do differently — only what it **does**
- Too late: the incorrect implementation already exists, tokens already wasted
- Does not prevent the problem — only detects it after the fact

**Why rejected:** Auto-generation is a post-hoc diagnostic tool, not a preventive measure. The ADR-0048 requirement is about *preventing* incorrect implementation, not detecting it afterward.

### Option 4: Post-Implementation Documentation Review (Rejected)

Allow implementation to proceed based on general knowledge, then review against official docs during code review.

**Positive:**
- No delay before implementation
- Reviewers catch errors before merge
- "Fail fast" approach to knowledge gaps

**Negative:**
- Implementation time is wasted if the approach is wrong — rework costs dominate
- Reviewers may not catch all docs-vs-implementation mismatches (review fatigue)
- Debugging time between implementation and review is wasted
- The deepresearch LiteLLM bug would have passed code review because the reviewer also lacked LiteLLM documentation knowledge

**Why rejected:** "Shift left" principle applies — finding issues earlier is cheaper. Post-implementation review is too late; the incorrect implementation is already written, and reviewers may lack the same documentation knowledge as the implementer.

### Option 5: Documentation Section Only, No Team Workflow Changes (Rejected)

Add the Documentation section to ADRs but do not add explicit documentation-reading steps to team workflows.

**Positive:**
- Low change surface (ADR format change only)
- Documentation is available for those who choose to read it

**Negative:**
- Without explicit workflow steps, the documentation will not be read — same failure mode as the current "nice to have" process
- Agents optimize for speed — they will skip optional reading in favor of immediate implementation
- No enforcement mechanism

**Why rejected:** The same pattern as ADR-0039 (Pre-Flight Knowledge Check Enforcement). A "nice to have" process step that is not structurally enforced will be routinely skipped. The workflow steps in team agent files are the enforcement mechanism.

### Do Nothing (Rejected)

Keep the current process: research options during selection, implement from general knowledge, debug issues as they arise.

**Why rejected:** The deepresearch LiteLLM incident is a concrete, recent example of the cost of this approach. Continued inaction guarantees similar incidents in the future. The cost of reading documentation (10-30 min) is dwarfed by the cost of debugging and rework (hours to days).

## Consequences

### Positive

1. **Fewer incorrect implementations.** API assumptions are validated against official documentation before code is written. The LiteLLM provider-prefix class of error becomes impossible because the requirement is documented before implementation.

2. **Less rework and fewer debugging cycles.** The deepresearch team spent hours debugging a problem that would have been caught by a 15-minute documentation read. This time is saved project-wide.

3. **ADRs become more valuable references.** Every ADR that selects a tool includes its documentation URL, version, key API concepts, and gotchas. This makes ADRs useful not just during design but throughout the implementation, review, testing, and hotfix phases.

4. **Reviewers have a concrete reference to validate against.** Instead of checking "does this look right?", reviewers check "does this match the documented API?" — an objective, verifiable criterion.

5. **Testers write correct mocks and stubs.** With documented API contracts as reference, test fixtures accurately reflect real request/response formats, reducing false positives and false negatives in test suites.

6. **Hotfixes land correctly on first attempt.** FLS agents read the relevant doc sections before applying fixes, reducing the "fix the fix" cycle common when working with unfamiliar dependencies.

7. **Documentation gaps are surfaced.** If a selected tool lacks official documentation, that fact is explicitly noted in the ADR, flagging a risk that can be addressed (or the tool can be reconsidered).

### Negative

1. **Slightly longer implementation time.** Reading relevant documentation sections adds 10-30 minutes per dependency. For projects using many small dependencies, this can add up. Mitigation: agents can parallelize doc reads (read multiple docs in a single batch call) and focus on the specific sections relevant to their implementation.

2. **Documentation may be out of date or incorrect.** Some tools have stale documentation that diverges from actual behavior. Mitigation: this is useful information — document the discrepancy in the ADR and escalate to Architects for resolution. The knowledge that "docs say X but code does Y" is itself valuable.

3. **ADR format change.** All existing ADRs that select tools do not have a Documentation section. Mitigation: the Documentation section is required for NEW ADRs only. Existing ADRs are updated on an as-needed basis when the project is reopened or significantly revised.

4. **Summary quality varies.** The "Key API concepts" field in the Documentation section depends on the agent's ability to extract the right information from the docs. Mitigation: Reviewers check for specificity during ADR review (see Section 4.1). Generic summaries are rejected.

### Risks and Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **Agents read docs superficially** | Medium | Medium | Reviewers check implementation against docs during code review (Section 4.2). Superficial reading produces implementation mismatches that get caught. |
| 2 | **Documentation section becomes boilerplate** | Medium | Low | Reviewers check for specificity during ADR review (Section 4.1). Generic entries like "it's an HTTP API" are rejected. |
| 3 | **Tool has no official documentation** | Low | Medium | Document the gap (Section 3, "When Documentation is Unavailable"). Flag risk to Director. Consider alternative tool. |
| 4 | **Docs change between ADR writing and implementation** | Low | Medium | Implementer re-reads relevant sections before implementation (this is part of the Engineers' workflow step). Update ADR if significant changes found. |
| 5 | **Time overhead for simple tools** | Medium | Low | Exemption for trivial/standard-library usage (Section 3, "When the Documentation Section is NOT Required"). Implementer uses judgment — if unsure, read docs. |
| 6 | **LLM context window constraints** | Low | Medium | Agents should read docs via `webfetch` and summarize the relevant sections. Full doc dumps are not required — targeted section reading is sufficient. |

### Follow-up Actions

1. Update `architects.md` — replace "Research before designing" (lines 82-83) with the expanded version that includes tool documentation identification (Section 5.1)
2. Insert new step 2b into `engineers.md` — official documentation reading requirement (Section 5.2)
3. Insert new step 2b into `testers.md` — API documentation reading for mock/stub contracts (Section 5.3)
4. Add two checklist items to `reviewers.md` — ADR Documentation section completeness + implementation-vs-docs verification (Section 5.4)
5. Insert step 3a-ii into `fls.md` — documentation reading before hotfixes (Section 5.5)
6. Add ADR-0048 to the design document's ADR Index table (Section 5)

## ADR References

- **ADR-0030** (Agentmemory Knowledge Flow) — internal knowledge recall, which this ADR complements with external documentation
- **ADR-0038** (Knowledge Recall Protocol) — fixed internal recall path, same complement relationship
- **ADR-0039** (Pre-Flight Knowledge Check Enforcement) — established structural enforcement pattern for process steps, which this ADR follows
- **ADR-0047** (Universal Test Execution Standard) — established the pattern of precise team-agent workflow integration in ADRs

### Source Files Modified

- `.opencode/agents/architects.md` — expand "Research before designing" step, add tool documentation identification
- `.opencode/agents/engineers.md` — insert doc-reading step before implementation
- `.opencode/agents/testers.md` — insert API doc-reading step before writing test fixtures
- `.opencode/agents/reviewers.md` — add two checklist items to Review Checklist
- `.opencode/agents/fls.md` — insert doc-reading step before hotfix application
- `docs/design/README.md` — add ADR-0048 to ADR Index table

### Files NOT Modified

- `.opencode/agents/director.md` — Director delegates to teams; the documentation requirement is enforced at the team level
- `.opencode/agents/scribes.md` — Scribes do not implement against tool APIs; no change needed
- `.opencode/skills/` — no skill changes needed; this is a workflow enforcement, not a new skill

### Open Questions

1. **Should the Documentation section also be added to existing ADRs retroactively?** Deferred — applied on an as-needed basis during project reopening or major revisions. A bulk retroactive update would be high effort for low immediate value.

2. **Should there be a "Documentation review" sentinel?** Considered but deferred — the Reviewers' checklist (Section 5.4) provides the same enforcement without infrastructure overhead.

3. **What about dependencies introduced during implementation (not in the original ADR)?** The Engineer who introduces the dependency must add a Documentation section to the ADR retrospectively (or create a mini-ADR if the change is significant enough). This is covered by the general rule: "if you use it, document it."

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| 1.0 | 2026-06-13 | Initial ADR — Mandatory Tool Documentation Review Before Implementation |
