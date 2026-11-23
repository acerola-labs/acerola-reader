use sqlx::SqlitePool;

use crate::{
    data::{
        models::sync::SyncHistoryLogEntry,
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

/// Persisted history of P2P sync sessions — terminal states only (`complete`/`error`),
/// never `started`/`progress`. `kind` is `"history"` or `"files"`, `status` is `"complete"`
/// or `"error"`. Follows the same `Entity`/`Bindable`/`Repository<T>` pattern as every model
/// in this folder (see `SyncHistoryLogEntry`); `find_recent` is custom because the generic
/// `Repository::find_all` does not support ORDER BY/LIMIT.
#[derive(Clone)]
pub struct SyncHistoryLogRepository {
    pub base: Repository<SyncHistoryLogEntry>,
    pool: SqlitePool,
}

impl SyncHistoryLogRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    /// Last `limit` rows, most recent first — used to fill the activity log when the
    /// network screen opens and to compute the "last sync" per peer.
    pub async fn find_recent(&self, limit: i64) -> Result<Vec<SyncHistoryLogEntry>, DbError> {
        let rows = sqlx::query_as::<_, SyncHistoryLogEntry>(
            "SELECT id, peer_id, kind, status, message, created_at
             FROM sync_history_log
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Deletes the whole persisted sync history — used by the "Clear" button on the Network
    /// screen. Does not affect the frontend's in-memory live log (current session), only what
    /// survives a restart.
    pub async fn delete_all(&self) -> Result<(), DbError> {
        let table = SyncHistoryLogEntry::table_name();
        sqlx::query(&format!("DELETE FROM {}", table)).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SyncHistoryLogRepository;
    use crate::{
        data::models::sync::SyncHistoryLogEntry,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    #[tokio::test]
    async fn insert_and_find_recent_orders_newest_to_oldest() {
        let pool = setup_test_db_with_comic().await;
        let repo = SyncHistoryLogRepository::new(pool);

        repo.base
            .insert(&SyncHistoryLogEntry::new("peer-a", "history", "complete", None))
            .await
            .unwrap();
        repo.base
            .insert(&SyncHistoryLogEntry::new("peer-a", "files", "error", Some("boom")))
            .await
            .unwrap();

        let recent = repo.find_recent(10).await.unwrap();
        assert_eq!(recent.len(), 2);
        // The second insert (files/error) must come first (most recent).
        assert_eq!(recent[0].kind, "files");
        assert_eq!(recent[0].status, "error");
        assert_eq!(recent[0].message.as_deref(), Some("boom"));
        assert_eq!(recent[1].kind, "history");
        assert_eq!(recent[1].message, None);
    }

    #[tokio::test]
    async fn find_recent_respects_the_limit() {
        let pool = setup_test_db_with_comic().await;
        let repo = SyncHistoryLogRepository::new(pool);

        for _ in 0..5 {
            repo.base
                .insert(&SyncHistoryLogEntry::new("peer-a", "history", "complete", None))
                .await
                .unwrap();
        }

        let recent = repo.find_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn delete_all_removes_every_persisted_row() {
        let pool = setup_test_db_with_comic().await;
        let repo = SyncHistoryLogRepository::new(pool);

        repo.base
            .insert(&SyncHistoryLogEntry::new("peer-a", "history", "complete", None))
            .await
            .unwrap();
        repo.base
            .insert(&SyncHistoryLogEntry::new("peer-a", "files", "error", Some("boom")))
            .await
            .unwrap();

        repo.delete_all().await.unwrap();

        let recent = repo.find_recent(10).await.unwrap();
        assert!(recent.is_empty());
    }
}
