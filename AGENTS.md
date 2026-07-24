# AGENTS.md — KodeHold v2.0 Development Rules

These rules are loaded by opencode before work begins.

## Safety

- Never push or merge without human approval.
- Never edit `.env`, `.env.*`, `auth/`, `payments/`, `secrets/`, or `credentials/`.
- Run `cargo test` before proposing any fix.
- Max 3 fix attempts per item; escalate after that.

## Development

- This is a Rust single-binary project. No Python, no scripts/ directory.
- All state lives in git tags (v2-init, v2-active, v2-review, v2-closed).
- Use `cargo build` to compile, `cargo test` to verify.
- ADRs are in `docs/adr/`, design doc is `docs/design/README.md`.
