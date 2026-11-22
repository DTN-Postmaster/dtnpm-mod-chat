//! Built-in permanent Chat service handling incoming `dtn://<station>/chat` bundles.

use std::sync::Arc;

use bytes::Bytes;
use hardy_async::async_trait;
use hardy_bpa::services::{Result as ServiceResult, Service, ServiceSink, StatusNotify};
use hardy_bpa::stream::{Receiver, Segment, buffer_stream};
use hardy_bpv7::bundle::Id;
use hardy_bpv7::eid::Eid;
use hardy_bpv7::parse::parse;
use hardy_bpv7::status_report::ReasonCode;
use time::OffsetDateTime;
use tokio::sync::OnceCell;
use tracing::{debug, info, warn};

use crate::chat_manager::{ChatManager, ChatMessage, normalize_chat_target};
use dtnpm_core::internal_storage::status_notify_to_parts;
use dtnpm_core::security::{AuthStatus, DirectionalKeyStore, VerifyPolicy, verify_parsed_bundle};
use dtnpm_core::service_tracker::ServiceTracker;

/// Permanent DTN BPv7 service for text messaging and chat conversations.
#[derive(Clone)]
pub struct ChatService {
    chat_manager: ChatManager,
    service_tracker: ServiceTracker,
    keystore: Arc<DirectionalKeyStore>,
    sink: Arc<OnceCell<Box<dyn ServiceSink>>>,
}

impl ChatService {
    /// Creates a new Chat service bound to a chat storage manager, service tracker and keystore.
    pub fn new(
        chat_manager: ChatManager,
        service_tracker: ServiceTracker,
        keystore: Arc<DirectionalKeyStore>,
    ) -> Self {
        Self {
            chat_manager,
            service_tracker,
            keystore,
            sink: Arc::new(OnceCell::new()),
        }
    }

    async fn handle_delivered_bundle(&self, data: Bytes) -> ServiceResult<()> {
        let Ok(parsed) = parse(data) else {
            debug!("ChatService: failed to parse delivered bundle");
            return Ok(());
        };

        let auth_status = verify_parsed_bundle(&parsed, &self.keystore);
        let policy = self.keystore.get_policy();
        if policy == VerifyPolicy::Strict && auth_status != AuthStatus::Valid {
            warn!(
                auth_status = ?auth_status,
                "ChatService: dropping delivered bundle due to strict policy without valid BPSec signature"
            );
            return Ok(());
        }

        let bundle = &parsed.bundle;
        let source_eid = &bundle.primary.id.source;
        let dest_eid = &bundle.primary.destination;

        // Skip null source or administrative records
        if source_eid.is_null() || bundle.primary.flags.is_admin_record {
            debug!("ChatService: ignoring null source or admin bundle");
            return Ok(());
        }

        // Extract payload block (#1)
        let Some(payload_slice) = bundle
            .blocks
            .get(&1)
            .and_then(|block| block.payload(&parsed.data))
        else {
            debug!("ChatService: delivered bundle has no payload block");
            return Ok(());
        };

        let content = String::from_utf8_lossy(payload_slice).to_string();
        let bundle_id = bundle.primary.id.to_key();
        let conversation_peer = normalize_chat_target(&source_eid.to_string());

        let dt = bundle
            .primary
            .id
            .timestamp
            .as_datetime()
            .unwrap_or_else(OffsetDateTime::now_utc);

        let created_at = dt
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| dt.to_string());

        let creation_time_ms = (dt.unix_timestamp_nanos() / 1_000_000) as i64;
        let lifetime_secs = bundle.primary.lifetime.as_secs().min(365 * 86_400);
        let expiry_timestamp =
            creation_time_ms.saturating_add((lifetime_secs as i64).saturating_mul(1000));

        let msg = ChatMessage {
            id: bundle_id.clone(),
            conversation_peer,
            sender_eid: source_eid.to_string(),
            recipient_eid: dest_eid.to_string(),
            content,
            direction: "inbound".to_string(),
            status: "received".to_string(),
            is_read: false,
            is_sticky: false,
            timestamp: creation_time_ms,
            expiry_timestamp,
            created_at,
            auth_status: auth_status.as_str().to_string(),
            delivery_status: None,
            delivery_reason: None,
            delivery_updated_at: None,
        };

        if let Err(e) = self.chat_manager.insert_message(&msg) {
            warn!("Failed to store inbound chat message {}: {}", bundle_id, e);
        } else {
            info!(
                "Stored new inbound chat message from {} (id: {})",
                source_eid, bundle_id
            );
            self.service_tracker.broadcast_chat_message(msg);
        }

        Ok(())
    }
}

#[async_trait]
impl Service for ChatService {
    async fn on_register(&self, _source: &Eid, sink: Box<dyn ServiceSink>) {
        let _ = self.sink.set(sink);
        debug!("ChatService registered with BPA");
    }

    async fn on_unregister(&self) {
        debug!("ChatService unregistered from BPA");
    }

    async fn on_status_notify(
        &self,
        bundle_id: &Id,
        from: &Eid,
        kind: StatusNotify,
        reason: ReasonCode,
        timestamp: Option<OffsetDateTime>,
    ) {
        let (status_str, reason_u64, ts_ms) = status_notify_to_parts(kind, reason, timestamp);
        let key = bundle_id.to_key();

        match self
            .chat_manager
            .update_chat_delivery_status(&key, status_str, reason_u64, ts_ms)
        {
            Ok(true) => {
                debug!(
                    bundle_id = %key,
                    from = %from,
                    delivery_status = status_str,
                    reason = reason_u64,
                    "Updated chat bundle delivery receipt"
                );
                if let Ok(Some(updated)) = self.chat_manager.get_chat_message(&key) {
                    self.service_tracker.broadcast_chat_message(updated);
                }
            }
            Ok(false) => debug!(
                bundle_id = %key,
                delivery_status = status_str,
                "Ignored chat status notification (unknown bundle or non-progressive status)"
            ),
            Err(err) => warn!(
                bundle_id = %key,
                delivery_status = status_str,
                "Failed to persist chat delivery receipt: {err}"
            ),
        }
    }

    async fn on_deliver(
        &self,
        bundle_id: &Id,
        _expiry: OffsetDateTime,
        total_len: u64,
        stream: &mut dyn Receiver<Segment>,
    ) -> ServiceResult<()> {
        debug!("ChatService on_deliver called for bundle_id={bundle_id:?}, len={total_len}");
        let data = buffer_stream(stream, total_len).await?;
        self.handle_delivered_bundle(data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dtnpm_core::MockBpa;
    use dtnpm_core::internal_storage::InternalStorage;
    use dtnpm_core::security::{Direction, KeyEntry, sign_bundle_with_key};
    use hardy_bpv7::builder::Builder;
    use hardy_bpv7::creation_timestamp::CreationTimestamp;
    use std::time::Duration;

    fn make_chat_service(keystore: Arc<DirectionalKeyStore>) -> (ChatService, InternalStorage) {
        let storage = InternalStorage::new_in_memory().unwrap();
        let bpa = Arc::new(MockBpa::new("dtn://stata/"));
        let tracker = ServiceTracker::new(bpa);
        (
            ChatService::new(storage.clone(), tracker, keystore),
            storage,
        )
    }

    fn build_chat_bundle(src: &str, dst: &str, text: &str, lifetime_secs: u64) -> Vec<u8> {
        let src_eid: Eid = src.parse().unwrap();
        let dst_eid: Eid = dst.parse().unwrap();
        let (_bundle, raw) = Builder::new(src_eid, dst_eid)
            .with_lifetime(Duration::from_secs(lifetime_secs))
            .with_payload(text.as_bytes().to_vec().into())
            .build(CreationTimestamp::now())
            .unwrap();
        raw.into_vec()
    }

    #[tokio::test]
    async fn strict_policy_rejects_unsigned_missing_key_and_invalid_accepts_valid() {
        let keystore = Arc::new(DirectionalKeyStore::empty());
        keystore.set_policy(VerifyPolicy::Strict).unwrap();
        let valid_secret = b"chat_inbound_secret_key_32_bytes".to_vec();
        keystore
            .add_key(
                Direction::Inbound,
                "dtn://f4jxq/*".to_string(),
                valid_secret.clone(),
                None,
            )
            .unwrap();

        let (service, storage) = make_chat_service(keystore);

        // 1. Unsigned bundle must be dropped
        let unsigned = build_chat_bundle("dtn://f4jxq/chat", "dtn://stata/chat", "unsigned", 3600);
        service
            .handle_delivered_bundle(Bytes::from(unsigned.clone()))
            .await
            .unwrap();
        assert!(storage.list_conversations().unwrap().is_empty());

        // 2. Signed by peer without a matching inbound key (MissingKey) must be dropped
        let unknown_raw = build_chat_bundle(
            "dtn://unknown/chat",
            "dtn://stata/chat",
            "missing key",
            3600,
        );
        let unknown_parsed = parse(Bytes::from(unknown_raw.clone())).unwrap();
        let unknown_key = KeyEntry {
            id: "u1".into(),
            direction: Direction::Outbound,
            pattern: "dtn://stata/*".into(),
            label: None,
            secret: b"another_secret_key_32_bytes_long".to_vec(),
            created_at: "now".into(),
        };
        let missing_key_signed =
            sign_bundle_with_key(&unknown_parsed.bundle, &unknown_raw, &unknown_key, None).unwrap();
        service
            .handle_delivered_bundle(Bytes::from(missing_key_signed))
            .await
            .unwrap();
        assert!(storage.list_conversations().unwrap().is_empty());

        // 3. Signed with wrong secret for matching peer (Invalid) must be dropped
        let parsed = parse(Bytes::from(unsigned.clone())).unwrap();
        let wrong_key = KeyEntry {
            id: "bad".into(),
            direction: Direction::Outbound,
            pattern: "dtn://stata/*".into(),
            label: None,
            secret: b"wrong_secret_key_32_bytes_123456".to_vec(),
            created_at: "now".into(),
        };
        let invalid_signed =
            sign_bundle_with_key(&parsed.bundle, &unsigned, &wrong_key, None).unwrap();
        service
            .handle_delivered_bundle(Bytes::from(invalid_signed))
            .await
            .unwrap();
        assert!(storage.list_conversations().unwrap().is_empty());

        // 4. Valid signature (AuthStatus::Valid) must be accepted
        let good_key = KeyEntry {
            id: "good".into(),
            direction: Direction::Outbound,
            pattern: "dtn://stata/*".into(),
            label: None,
            secret: valid_secret,
            created_at: "now".into(),
        };
        let valid_signed =
            sign_bundle_with_key(&parsed.bundle, &unsigned, &good_key, None).unwrap();
        service
            .handle_delivered_bundle(Bytes::from(valid_signed))
            .await
            .unwrap();
        let convs = storage.list_conversations().unwrap();
        assert_eq!(convs.len(), 1);
        assert_eq!(convs[0].total_messages, 1);
    }

    #[tokio::test]
    async fn warn_and_ignore_policies_accept_unsigned_bundles() {
        for policy in [VerifyPolicy::Warn, VerifyPolicy::Ignore] {
            let keystore = Arc::new(DirectionalKeyStore::empty());
            keystore.set_policy(policy).unwrap();
            let (service, storage) = make_chat_service(keystore);
            let unsigned =
                build_chat_bundle("dtn://f4jxq/chat", "dtn://stata/chat", "opt-in", 3600);
            service
                .handle_delivered_bundle(Bytes::from(unsigned))
                .await
                .unwrap();
            let convs = storage.list_conversations().unwrap();
            assert_eq!(convs.len(), 1, "policy {policy:?} should accept unsigned");
        }
    }

    #[tokio::test]
    async fn huge_bundle_lifetime_is_clamped_without_overflow() {
        let keystore = Arc::new(DirectionalKeyStore::empty());
        keystore.set_policy(VerifyPolicy::Warn).unwrap();
        let (service, storage) = make_chat_service(keystore);
        let raw = build_chat_bundle(
            "dtn://f4jxq/chat",
            "dtn://stata/chat",
            "huge lifetime",
            u64::MAX / 1000,
        );
        service
            .handle_delivered_bundle(Bytes::from(raw))
            .await
            .unwrap();
        let msgs = storage
            .get_conversation_messages("dtn://f4jxq/chat")
            .unwrap();
        assert_eq!(msgs.len(), 1);
        let delta_ms = msgs[0].expiry_timestamp - msgs[0].timestamp;
        assert_eq!(delta_ms, 365 * 86_400 * 1000);
    }

    #[tokio::test]
    async fn status_notify_persists_delivery_receipt_without_breaking_status_check() {
        let keystore = Arc::new(DirectionalKeyStore::empty());
        let (service, storage) = make_chat_service(keystore);

        let src: Eid = "dtn://stata/chat".parse().unwrap();
        let dst: Eid = "dtn://statb/chat".parse().unwrap();
        let (bundle, _) = Builder::new(src, dst)
            .with_lifetime(Duration::from_secs(3600))
            .with_payload(b"outbound receipt test".to_vec().into())
            .build(CreationTimestamp::now())
            .unwrap();
        let bundle_key = bundle.primary.id.to_key();

        let outbound = ChatMessage {
            id: bundle_key.clone(),
            conversation_peer: "dtn://statb/chat".to_string(),
            sender_eid: "dtn://stata/chat".to_string(),
            recipient_eid: "dtn://statb/chat".to_string(),
            content: "outbound receipt test".to_string(),
            direction: "outbound".to_string(),
            status: "sent".to_string(),
            is_read: true,
            is_sticky: false,
            timestamp: 1_700_000_000_000,
            expiry_timestamp: 1_700_003_600_000,
            created_at: "2026-11-22T12:00:00Z".to_string(),
            auth_status: "valid".to_string(),
            delivery_status: None,
            delivery_reason: None,
            delivery_updated_at: None,
        };
        storage.insert_message(&outbound).unwrap();

        let mut rx = service.service_tracker.subscribe();
        let report_from: Eid = "dtn://statb/".parse().unwrap();
        let ts = OffsetDateTime::from_unix_timestamp(1_700_000_500).unwrap();
        service
            .on_status_notify(
                &bundle.primary.id,
                &report_from,
                StatusNotify::Delivered,
                ReasonCode::NoAdditionalInformation,
                Some(ts),
            )
            .await;

        let msgs = storage
            .get_conversation_messages("dtn://statb/chat")
            .unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, bundle_key);
        assert_eq!(msgs[0].status, "sent");
        assert_eq!(msgs[0].delivery_status.as_deref(), Some("delivered"));
        assert_eq!(msgs[0].delivery_reason, Some(0));
        assert_eq!(msgs[0].delivery_updated_at, Some(1_700_000_500_000));

        let ev = rx.try_recv().expect("expected SSE ChatMessage event");
        match ev {
            dtnpm_core::service_tracker::ServiceEvent::ChatMessage { message } => {
                assert_eq!(message["id"], bundle_key);
                assert_eq!(message["delivery_status"], "delivered");
            }
            other => panic!("unexpected service event: {other:?}"),
        }
    }
}
