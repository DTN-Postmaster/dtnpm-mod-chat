CREATE TABLE IF NOT EXISTS chat_conversations (
    peer TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id TEXT PRIMARY KEY,
    conversation_peer TEXT NOT NULL,
    sender_eid TEXT NOT NULL,
    recipient_eid TEXT NOT NULL,
    content TEXT NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('inbound', 'outbound')),
    status TEXT NOT NULL CHECK(status IN ('queued', 'sent', 'received')),
    is_read INTEGER NOT NULL DEFAULT 0,
    is_sticky INTEGER NOT NULL DEFAULT 0,
    timestamp INTEGER NOT NULL,
    expiry_timestamp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    auth_status TEXT NOT NULL DEFAULT 'unsigned'
);

CREATE INDEX IF NOT EXISTS idx_chat_peer ON chat_messages(conversation_peer, timestamp);
CREATE INDEX IF NOT EXISTS idx_chat_expiry ON chat_messages(is_sticky, expiry_timestamp);

INSERT OR IGNORE INTO chat_conversations (peer, created_at)
SELECT DISTINCT conversation_peer, created_at FROM chat_messages;
