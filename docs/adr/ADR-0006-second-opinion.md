---
phase:
  current: 1
  total: 1
  status:
    1: done
---

# ADR-0006: Second Opinion Protocol

## Status

Accepted

## Context

No single LLM is perfect. Different models have different strengths in reasoning, code generation, security analysis, and creative design. For critical decisions — architecture choices, security-sensitive code, complex bugs — relying on a single model's output increases risk. A second opinion from a different model provides a cross-check.

Key forces:
- Second opinions cost additional tokens — they must be reserved for high-impact decisions
- The secondary model should be meaningfully different (different architecture, different training data)
- The process must not block the primary workflow — second opinions run in parallel or asynchronously where possible

## Decision

### Trigger Conditions

The Director automatically triggers a second opinion when any of these conditions are met:

1. **Architecture Decision**: Any new ADR or significant design change
2. **Security-Critical Code**: Authentication, authorization, encryption, input sanitization
3. **Complex Bug**: A bug that has escaped initial review and testing
4. **Ambiguous Design**: Design review results in a split decision (Reviewers disagree)
5. **Manual Request**: Any team member can request a second opinion via the Director

### Protocol

```
1. Director identifies trigger condition
2. Context is packaged:
   - Relevant design doc excerpt (max 2k tokens)
   - Code snippet or diff (max 4k tokens)
   - Specific question (max 500 tokens)
   - Primary model's proposed solution (max 2k tokens)
3. Request sent to secondary LLM (different model/provider)
4. Secondary response received
5. Comparison:
   - Agreement → proceed with confidence
   - Minor disagreement → Reviewers evaluate and reconcile
   - Major disagreement → Director escalates to Architects for design review
6. Result recorded in ICM by Scribes
```

### Secondary Model Selection — Cross-Provider Mandatory

The secondary model **must** come from a different provider than the primary. Same-provider models share architectural biases and training data — they do not provide an independent cross-check.

| Primary | Acceptable Secondary | Reason |
|---------|---------------------|--------|
| Ollama / local (any model) | `anthropic/claude-*` or `openai/gpt-*` / `openai/codex-*` | True cross-provider, different training |
| `opencode/*` (OpenCode Zen) | `anthropic/claude-*` or `openai/*` | Different architecture and data |
| `anthropic/claude-*` | `openai/codex-*` or Ollama (any) | Reverse the direction |

**Valid second opinion providers** (in priority order):
1. `anthropic/claude-*` — Claude Sonnet/Haiku/Opus
2. `openai/codex-*` — OpenAI Codex (code-specialised)
3. `openai/gpt-*` — OpenAI GPT models
4. Any other provider that differs from the primary

**If no secondary provider is available** (e.g. only Ollama is configured):
- The Director MUST skip second opinion for non-critical triggers
- For critical triggers (security, architecture decisions), the Director MUST inform the user that a secondary provider is required and prompt them to configure one via `/connect`

### Cross-Provider Cost Management

- Second opinion is limited to 3 concurrent requests per project phase
- Token cost is logged per second opinion in ICM
- If token budget is constrained, only security-critical triggers activate
- Cross-provider calls may incur API costs (Claude/Codex) — user should have API keys configured via OpenCode `/connect`

### Protocol Update — Secondary Model Invocation

When executing a second opinion, the Director must:

1. **Identify the primary model** — read from current OpenCode context or `model`/`small_model` config
2. **Select a different provider** — prefer `anthropic/claude-*` or `openai/codex-*`
3. **Invoke via Reviewers** — package context, pass model hint in the Task prompt
4. **If the same model would be used** — abort and warn: "Second opinion requires a different provider. Run `/connect` to add Anthropic or OpenAI."

The OpenCode CLI supports per-request model selection. The Task tool's prompt should include:
```
Use a different model/provider than the current default.
Preferred: anthropic/claude-sonnet-4 or openai/codex-5.
```

## Consequences

- Positive: Cross-model validation catches errors single-model workflows miss
- Positive: Structured protocol prevents second opinions from blocking the main workflow
- Positive: ICM logging creates a trace of which decisions were cross-checked
- Negative: Additional token cost per second opinion request
- Negative: Adds latency to critical decisions (mitigated by parallel execution)
- Negative: Requires a second available LLM provider (extraneous configuration)
