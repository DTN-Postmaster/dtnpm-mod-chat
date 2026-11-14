use std::sync::Arc;

use dtnpm_mod_chat::ChatModule;
use dtnpm_web_core::TestServerBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let srv = TestServerBuilder::new()
        .node_id("dtn://chat-dev/")
        .callsign("CHATDEV")
        .with_module(Arc::new(ChatModule::new()))
        .build()
        .await?;

    let (addr, handle) = srv.spawn_http().await?;
    println!("dtnpm-mod-chat test server listening on http://{addr}");
    println!("Capabilities: http://{addr}/api/capabilities");
    println!("Chat API:     http://{addr}/api/chat/v1/conversations");

    tokio::signal::ctrl_c().await?;
    handle.abort();
    Ok(())
}
