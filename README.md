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

## License

Dual-licensed under the Apache License, Version 2.0 or the MIT License (crate metadata: `Apache-2.0`). See [LICENSE.md](LICENSE.md).
