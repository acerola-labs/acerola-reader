use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::{query::Query, sqlite::SqliteArguments, Sqlite};

use crate::data::repositories::{Bindable, Entity};

/// One row of the persisted P2P sync session history (history and files) —
/// terminal states only (`complete`/`error`), not `started`/`progress`. It exists to survive
/// an app restart; the in-memory "live" log still exists separately for events that
/// arrive in real time during the current session.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncHistoryLogEntry {
    pub id: i64,
    pub peer_id: String,
    pub kind: String,
    pub status: String,
    pub message: Option<String>,
    pub created_at: i64,
}

/// Contract with the generic [`crate::data::repositories::Repository`] — same pattern as
/// every model in this folder (`ChapterArchive`, `ChapterRead`, etc), even though the table is
/// append-only: `id` is computed in Rust (does not rely on SQLite AUTOINCREMENT) and bound
/// explicitly on insert, just like `ChapterArchive::id` uses `path_hash(...)`.
impl Entity for SyncHistoryLogEntry {
    fn columns() -> &'static [&'static str] {
        &["id", "peer_id", "kind", "status", "message", "created_at"]
    }
    fn table_name() -> &'static str {
        "sync_history_log"
    }
    fn id(&self) -> i64 {
        self.id
    }
}

impl Bindable for SyncHistoryLogEntry {
    fn bind_insert<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(self.id)
            .bind(&self.peer_id)
            .bind(&self.kind)
            .bind(&self.status)
            .bind(&self.message)
            .bind(self.created_at)
    }

    fn bind_update<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(&self.peer_id)
            .bind(&self.kind)
            .bind(&self.status)
            .bind(&self.message)
            .bind(self.created_at)
            .bind(self.id) // <- id for WHERE id = ?
    }
}

impl SyncHistoryLogEntry {
    /// `id` computed in Rust (epoch in nanoseconds) instead of letting SQLite
    /// autoincrement — same convention as `ChapterArchive::id` (via `path_hash`), so
    /// `Bindable::bind_insert` can bind the id explicitly. A collision would require two
    /// calls in the same nanosecond, which doesn't happen for sync events (the network is
    /// orders of magnitude slower than that).
    pub fn new(peer_id: &str, kind: &str, status: &str, message: Option<&str>) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();

        Self {
            id: now.as_nanos() as i64,
            peer_id: peer_id.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            message: message.map(str::to_string),
            created_at: now.as_millis() as i64,
        }
    }
}
