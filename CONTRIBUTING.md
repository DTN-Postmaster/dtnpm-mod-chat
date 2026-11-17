# Contributing Guide — `dtnpm-mod-chat`

This guide outlines the development rules and security requirements for the `dtnpm-mod-chat` Chat module.

---

## 1. Core Principles

- **Zero Panic / Zero Unwrap**: No `.unwrap()` or `.expect()` calls in production code.
- **BPSec & XSS Security**:
  - Any inbound bundle under strict policy must be successfully authenticated before delivery.
  - UI-side Markdown rendering must strictly use cryptographic nonces and block remote images by default.
- **Architectural Traceability (ADR)**:
  - Any change to the database schema, API, or protocol must be documented in an ADR file under `adr/` and indexed in `adr/README.md`.

---

## 2. Local Verification

```bash
cargo fmt --all -- --check
cargo clippy --offline --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::panic
cargo test --offline --all-features
./scripts/check-security.sh --strict
```
