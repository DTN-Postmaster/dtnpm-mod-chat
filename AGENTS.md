# Agent Guide — `dtnpm-mod-chat`

## 🎯 Mission
Provide the complete fullstack DTN instant messaging application module (`WebModule`) for **DTN Postmaster**, including the BPA service (`ChatService`), local SQLite persistence for conversations, the `/api/chat/v1/...` REST API, and the secure Preact UI component `<ChatView />`.

---

## 📝 Documentation Language

- **English by Default**: English must be used as the language for all documentation, unless the file is suffixed with `.fr.md` or another language is explicitly requested in the prompt.

---

## 🛡️ Security Frameworks and Requirements

### 1. Cyber Resilience Act (CRA) & BPSec (RFC 9172)
- **Strict BPSec Verification by Default**:
  - In `VerifyPolicy::Strict` mode, any inbound bundle whose status is not strictly `Valid` (i.e., `Unsigned`, `MissingKey`, or `Invalid`) MUST be rejected and dropped before storage or notification.
  - Permissive policies (`Warn`, `Ignore`) are explicit configurable opt-ins for test environments.
- **Bundle Lifetime Clamping**:
  - Expiration date calculations (`lifetime_secs * 1000`) must be saturated or clamped to prevent any `u64` arithmetic overflow.
- **Supply-Chain & VEX**:
  - `cargo-audit` and `cargo-deny` audits verified; `not_affected` VEX for `RUSTSEC-2026-0459`.
  - CI actions pinned by full 40-character SHA-1.
- **Security Policy**: Documented in [SECURITY.md](SECURITY.md).

### 2. OWASP & Frontend Security
- **XSS Protection on Markdown Rendering**:
  - The Markdown parser uses random cryptographic nonces (`crypto.getRandomValues`) for escaping tags and autolinks, preventing any collision or malicious injection.
  - Remote images are disabled by default (`allowExternalImages: false`) to prevent any metadata leakage or IP geolocation.
  - No linear regular expressions vulnerable to ReDoS are tolerated in the parser.
- **UI Robustness (`ChatView.tsx`)**:
  - Strict EID normalization (`normalizeChatTarget`) when filtering active conversations.
  - Silent handling of read-receipt acknowledgement errors (`.catch(() => {})`).

---

## 🦀 Rust Engineering Standards

1. **Zero Panic / Zero Unwrap Rule**:
   - All code under `src/` must be 100% free of `.unwrap()`, `.expect()`, and the `panic!` macro.
   - Strict linter: `-D warnings -D clippy::unwrap_used -D clippy::panic`.
2. **Mandatory ADRs**:
   - Any Chat/BPSec architectural decision must be recorded under `adr/` and indexed in [adr/README.md](adr/README.md).

---

## ⚡ Critical Verification Commands

- `cargo fmt --all -- --check`
- `cargo clippy --offline --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::panic`
- `cargo test --offline --all-features`
- `./scripts/check-security.sh --strict`
- `./scripts/check-coverage.sh --summary-only`
