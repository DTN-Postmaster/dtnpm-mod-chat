# dtnpm-mod-chat

Fullstack **DTN Chat** plugin for **DTN Postmaster** (`dtnpm`), implementing `dtnpm_web_core::WebModule`.

## Features

- **BPA Service (`ChatService`)**: Registers `dtn://<node>/chat`, receives inbound chat bundles, verifies directional BPsec signatures, and notifies WebUI clients via SSE.
- **SQLite Persistence**: Provides `migrations/0001_chat_schema.sql` (`chat_messages`) over the shared SQLite `Arc<Mutex<Connection>>`.
- **HTTP API (`/api/chat/v1/...`)**: Conversation listing, history retrieval, outbound bundle dispatch (`ChatManager`), unread counters, and read-receipt acknowledgement.
- **Standalone Mode**: Includes `dtnpm-mod-chat` daemon (`cli` feature) and `examples/test_server.rs`.
- **WebUI Component (`ui/`)**: Preact `<ChatView />` component and Playwright E2E spec.

## Build and Test

Requires sibling checkouts of `../dtnpm-core`, `../dtnpm-web-core`, and `../../hardy/*`.

```bash
cargo test --all-targets
```

Run the standalone mock server example:

```bash
cargo run --example test_server
```

## Security & Compliance

- **BPSec (RFC 9172)**: Strict signature verification by default; rejection of unsigned bundles and missing keys under `VerifyPolicy::Strict`.
- **Cyber Resilience Act (CRA)**: Token file security (`--token-file`), lifetime calculation clamping preventing integer overflows, VEX audit documentation (`RUSTSEC-2026-0459`), and SHA-pinned CI actions.
- **Frontend XSS Hardening**: Markdown parser uses random cryptographic nonces, disables remote external images by default, and avoids ReDoS regexes.
- **Rust Quality Standards**: 100% `unwrap`-free and `panic!`-free production codebase.
- **Documentation**: See [`AGENTS.md`](AGENTS.md), [`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md), and Architectural Decision Records in [`adr/`](adr/).

## License

Dual-licensed under the Apache License, Version 2.0 or the MIT License (crate metadata: `Apache-2.0`). See [LICENSE.md](LICENSE.md).
