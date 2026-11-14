//! DTN Chat fullstack module implementing [`dtnpm_web_core::WebModule`].

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use hardy_async::async_trait;
use hardy_bpv7::eid::Service;
use rusqlite::Connection;

use dtnpm_web_core::{ModuleContext, WebModule};

pub mod chat_manager;
pub mod chat_service;
pub mod routes;

pub use chat_manager::{
    ChatManager, ChatMessage, ConversationSummary, extract_callsign, normalize_chat_endpoint,
    normalize_chat_target, normalize_peer_node,
};
pub use chat_service::ChatService;

/// SQL schema applied on the shared SQLite connection when registering [`ChatModule`].
pub const CHAT_SCHEMA_SQL: &str = include_str!("../migrations/0001_chat_schema.sql");

/// Modular DTN Chat plugin exposing `/api/chat/v1/...` routes and registering `dtn://<node>/chat`.
#[derive(Debug, Default, Clone)]
pub struct ChatModule;

impl ChatModule {
    /// Creates a new [`ChatModule`] instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WebModule for ChatModule {
    fn id(&self) -> &'static str {
        "chat"
    }

    fn name(&self) -> &'static str {
        "DTN Chat"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn services(&self) -> Vec<String> {
        vec!["chat".to_string()]
    }

    fn migrate(&self, conn: &Connection) -> anyhow::Result<()> {
        conn.execute_batch(CHAT_SCHEMA_SQL)?;
        Ok(())
    }

    async fn register_bpa(&self, ctx: &ModuleContext) -> anyhow::Result<()> {
        let chat_service = Arc::new(ChatService::new(
            ctx.internal_storage.clone(),
            ctx.service_tracker.clone(),
            ctx.keystore.clone(),
        ));
        ctx.service_tracker
            .register_service_with_type(
                Service::Dtn("chat".into()),
                chat_service,
                "Built-in Chat",
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register Chat service: {e}"))?;
        Ok(())
    }

    fn api_routes(&self, ctx: &ModuleContext) -> Router<()> {
        Router::new()
            .route(
                "/conversations",
                get(routes::list_conversations).post(routes::create_conversation),
            )
            .route(
                "/conversations/{peer}",
                get(routes::get_conversation_messages).post(routes::send_chat_message),
            )
            .route(
                "/conversations/{peer}/read",
                post(routes::mark_conversation_read),
            )
            .route(
                "/messages",
                get(routes::get_conversation_messages_query).post(routes::send_chat_message_body),
            )
            .route("/read", post(routes::mark_conversation_read_body))
            .route("/messages/{id}/sticky", post(routes::toggle_chat_sticky))
            .with_state(ctx.clone())
    }
}
