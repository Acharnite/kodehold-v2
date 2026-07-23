---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0026: Second Opinion Same-Model Bias Enforcement

## Status

Superseded

> Superseded by dedicated second-opinion subagent with Google Gemma 3 12B via OpenRouter (cross-provider). The problem of same-model bias was solved by using a different provider/model rather than enriched gate markers.

## Context

ADR-0006 mandates cross-provider second opinions: "The secondary model MUST be from a different provider than the primary." ADR-0017 made second opinion mandatory for ADRs and design document updates. However, in practice:

1. **Only one provider is configured** — `opencode.json` defines Ollama (`qwen3:8b-opencode`) as the sole provider. No Anthropic, no OpenAI, no other provider exists.
2. **No runtime enforcement** — gate.sh checks for `.second_opinion_done` marker but cannot verify *which model* was used. The marker is an opaque touch file.
3. **No model selection API** — The Task tool's subagent model is determined by OpenCode's provider config, not by per-request overrides. Instruction-based hints ("use a different model") are unreliable.
4. **Same-provider fallback is undefined** — ADR-0006 says "skip second opinion if only one provider," but gate.sh unconditionally requires `.second_opinion_done`. There is no "skip" path.

The result: second opinions are either (a) not truly cross-provider (same Ollama model), or (b) blocked entirely when only one provider exists, with no documented fallback.

Key forces:
- Cross-provider validation provides genuine independence; same-provider validation has shared biases
- Forcing users to configure multiple providers before any work can begin is too restrictive
- The gate must not silently accept same-provider second opinions as equivalent to cross-provider
- Token cost of cross-provider second opinions (API calls to Anthropic/OpenAI) must be acknowledged

## Decision

We address this in three layers: (1) enriched marker metadata, (2) provider detection in gate.sh, (3) fallback policy for single-provider environments.

### Layer 1: Enriched `.second_opinion_done` Marker

The `.second_opinion_done` marker changes from a touch file to a structured file containing:

```json
{
  "timestamp": "2026-05-29T14:30:00Z",
  "primary_model": "ollama/qwen3:8b-opencode",
  "secondary_model": "ollama/qwen3:8b-opencode",
  "primary_provider": "ollama",
  "secondary_provider": "ollama",
  "cross_provider": false,
  "trigger": "adr_new",
  "verdict": "agree"
}
```

The `cross_provider` field is `true` only when `primary_provider != secondary_provider`.

**Reviewers responsibility:** When creating `.second_opinion_done`, Reviewers must populate this JSON with the model information from the current session context.

### Layer 2: Provider Detection in gate.sh

gate.sh gains a new function `check_second_opinion_quality()` that:

1. Reads `.second_opinion_done` as JSON
2. If JSON parse fails: warn "legacy marker format — treat as same-provider"
3. If `cross_provider: true`: pass "Cross-provider second opinion verified"
4. If `cross_provider: false`: issue a WARNING (not failure) — "Same-provider second opinion detected — reduced independence. Configure additional providers via `/connect` for true cross-provider validation."
5. If marker is a plain touch file (no JSON): warn and treat as legacy

This check runs in `init_to_active()` and `reopen_to_active()` alongside the existing marker check.

### Layer 3: Fallback Policy for Single-Provider Environments

When only one provider is configured (detected via `opencode.json`):

| Trigger Type | Current Behavior | New Behavior |
|-------------|-----------------|--------------|
| Non-critical (complex bug, minor docs) | gate requires `.second_opinion_done` | gate allows skip — add `--skip-second-opinion` flag or document that non-critical skips are acceptable |
| Critical (ADR, design update, security) | gate requires `.second_opinion_done` | gate requires `.second_opinion_done` AND warns about same-provider limitation. User is informed. |

**The `--skip-second-opinion` flag** is a new gate.sh option:
- Available only for non-critical transitions
- When used, gate.sh prints a warning: "Second opinion skipped — only one provider configured. Add Anthropic or OpenAI via `/connect` for full cross-provider validation."
- Logged in gate output for traceability

### Layer 4: Provider Availability Check

gate.sh adds a `check_provider_availability()` function that:
1. Reads `opencode.json` and counts configured providers
2. If only 1 provider: warn "Single provider configured — second opinion will be same-provider"
3. If 0 providers: fail "No providers configured"
4. If 2+ providers: pass "Multiple providers available for cross-provider validation"

This check runs at the START of every gate transition and is informational (does not block).

### Files Changed

| File | Change |
|------|--------|
| `scripts/gate.sh` | Add `check_second_opinion_quality()`, `check_provider_availability()`, `--skip-second-opinion` flag |
| `.opencode/agents/reviewers.md` | Update step 8: write structured JSON to `.second_opinion_done` instead of touch file |
| `docs/adr/ADR-0006-second-opinion.md` | Add "Fallback Policy" section referencing ADR-0026 |
| `docs/design/README.md` | Document enriched marker format in Quality Gates section |

### Migration

- Existing `.second_opinion_done` touch files are treated as legacy (same-provider warning, not failure)
- No migration of existing markers needed — they are ephemeral and deleted after gate transitions

## Consequences

- Positive: gate.sh can now detect and warn about same-provider second opinions
- Positive: Structured marker enables traceability — which model reviewed what, when
- Positive: Single-provider environments can proceed with documented limitations
- Positive: No breaking change — existing behavior continues, warnings are additive
- Negative: Reviewers must write JSON instead of `touch .second_opinion_done` — small overhead
- Negative: gate.sh gains JSON parsing complexity (uses `jq` if available, falls back to `grep`)
- Negative: `--skip-second-opinion` flag could be misused — must be documented as "non-critical only"
- Neutral: True cross-provider validation still requires user to configure multiple providers via `/connect`
