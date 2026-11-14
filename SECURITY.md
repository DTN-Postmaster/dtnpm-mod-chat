# Security Policy — `dtnpm-mod-chat`

## Supported Versions

| Version | Supported | Security Updates |
|---------|-----------|------------------|
| 0.2.x   | Yes       | Active           |
| < 0.2   | No        | End of Life      |

## Security Architecture & EU Cyber Resilience Act (CRA) Alignment

`dtnpm-mod-chat` implements the DTN BPv7 Chat service (`dtn://<station>/chat`), SQLite message store, HTTP API routes, and Preact UI for `dtn-postmaster`:

- **Memory Safety & Panic-Free Production Targets**: Built in Rust (`edition = "2024"`) and gated in CI with `-D warnings -D clippy::unwrap_used -D clippy::panic`.
- **BPSec Enforcement (`src/chat_service.rs`)**: Inbound bundles are verified against `DirectionalKeyStore`. Under `VerifyPolicy::Strict`, only bundles with `AuthStatus::Valid` are accepted (`Unsigned`, `MissingKey`, and `Invalid` are dropped with a warning). Explicit opt-in policies (`VerifyPolicy::Warn`, `VerifyPolicy::Ignore`) remain available for unauthenticated interoperability.
- **Bounded Lifetime Arithmetic (`src/chat_service.rs`, `src/routes.rs`)**: Bundle lifetimes are clamped to 1 year (`365 * 86_400` seconds) and converted to millisecond expirations using saturating arithmetic (`saturating_mul`, `saturating_add`).
- **XSS-Hardened Markdown Rendering (`ui/src/utils/markdown.ts`)**: Uses per-call random nonces (stripping NUL bytes from input), linear link regexes, autolink extraction prior to HTML escaping, and blocks SVG/external image URLs unless explicitly opted in via `allowExternalImages: true`.
- **Standalone Secret Handling (`src/main.rs`)**: Supports `--token-file` (`DTN_HDY_TOKEN_FILE`) alongside `--token` (`DTN_HDY_TOKEN`) and Unix Domain Sockets (`--socket`) so bearer credentials need not appear on process command lines.
- **Supply Chain & VEX**: `serde_yaml` (`RUSTSEC-2024-0449`) is not present in the dependency graph. `RUSTSEC-2026-0459` (`libsqlite3-sys` SQLite `session` extension) is documented in `.cargo/audit.toml` and `deny.toml` as VEX Not Affected because `dtnpm-mod-chat` and `dtnpm-core` do not enable the SQLite `session` extension.
- **SBOM & CI Hardening**: CI runs with `permissions: contents: read`, SHA-pinned GitHub Actions, and CycloneDX SBOM generation.

## Reporting a Vulnerability

Do **not** open a public GitHub issue for security vulnerabilities.
Report vulnerabilities privately via GitHub Security Advisories for `dtn-postmaster/dtnpm-mod-chat` or by contacting the maintainers directly. Please include:

- Component version and commit hash
- Configuration flags (`verify_policy`, standalone CLI/UI options, etc.)
- Minimal proof-of-concept or reproduction steps

We aim to acknowledge reports within 72 hours and coordinate a fix before public disclosure.
