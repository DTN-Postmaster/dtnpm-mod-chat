# ADR 0001: BPSec Strict Verification, Bounded Lifetimes, and Token File Auth

- **Status**: Accepted
- **Scope**: `dtnpm-mod-chat` (`src/chat_service.rs`, `src/routes.rs`, `src/main.rs`)

## Context
Three security risks affected chat bundle ingestion, expiration arithmetic, and standalone BPA authentication:
1. Under `VerifyPolicy::Strict`, `ChatService::handle_delivered_bundle` previously dropped only `AuthStatus::Invalid`, permitting `AuthStatus::Unsigned` and `AuthStatus::MissingKey` bundles through.
2. Bundle lifetime conversion `lifetime_secs as i64 * 1000` and `timestamp + ...` could overflow on extreme `u64` lifetimes (`BPv7` primary block lifetime or API overrides).
3. Passing gRPC bearer tokens solely via `--token` exposes secrets in process listings when environment variables are inconvenient.

## Decision
1. Enforce `auth_status == AuthStatus::Valid` whenever `VerifyPolicy::Strict` is active, logging and dropping `Unsigned`, `MissingKey`, and `Invalid` bundles, while preserving explicit opt-in acceptance under `VerifyPolicy::Warn` and `VerifyPolicy::Ignore`.
2. Clamp chat bundle lifetimes to `365 * 86_400` seconds (1 year) in both `ChatService` and `routes.rs`, and compute `expiry_timestamp` with `saturating_mul` and `saturating_add`.
3. Add `--token-file <PATH>` (`DTN_HDY_TOKEN_FILE`) to `src/main.rs` with whitespace trimming and explicit validation for empty or unreadable files.

## Consequences
- `Strict` policy provides true end-to-end cryptographic authenticity without breaking unauthenticated amateur-radio operation when `Warn` or `Ignore` is explicitly chosen.
- Malformed or extreme bundle lifetimes cannot cause integer overflow or debug-mode panics.
- Systemd/container deployments can mount gRPC tokens via read-only credential files instead of CLI arguments.
