use sqlx::SqlitePool;

use crate::{
    cmd::events::history::ReadingHistoryPayload,
    data::{
        models::history::reading_history::ReadingHistory,
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

#[derive(Clone)]
pub struct ReadingHistoryRepository {
    pub base: Repository<ReadingHistory>,
    pool: SqlitePool,
}

impl ReadingHistoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    /// Inserts or updates a comic's reading progress.
    /// Upserts by `comic_directory_id` (the table's unique key).
    pub async fn upsert(&self, history: &ReadingHistory) -> Result<ReadingHistory, DbError> {
        let cols = ReadingHistory::columns().join(", ");
        let table = ReadingHistory::table_name();

        let result = sqlx::query_as::<_, ReadingHistory>(&format!(
            "INSERT INTO {} ({})
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(comic_directory_id) DO UPDATE SET
                 chapter_archive_id = excluded.chapter_archive_id,
                 last_page = excluded.last_page,
                 is_completed = excluded.is_completed,
                 updated_at = excluded.updated_at
             RETURNING *",
            table, cols
        ))
        .bind(history.comic_directory_id)
        .bind(history.chapter_archive_id)
        .bind(history.last_page)
        .bind(history.is_completed)
        .bind(history.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Returns a comic's reading history by `comic_directory_id`.
    pub async fn find_by_comic_id(
        &self, comic_directory_id: i64,
    ) -> Result<Option<ReadingHistory>, DbError> {
        let cols = ReadingHistory::columns().join(", ");
        let table = ReadingHistory::table_name();

        let result = sqlx::query_as::<_, ReadingHistory>(&format!(
            "SELECT {} FROM {} WHERE comic_directory_id = ?",
            cols, table
        ))
        .bind(comic_directory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Returns the full reading history, most recent first.
    pub async fn find_all_ordered(&self) -> Result<Vec<ReadingHistory>, DbError> {
        let cols = ReadingHistory::columns().join(", ");
        let table = ReadingHistory::table_name();

        let result = sqlx::query_as::<_, ReadingHistory>(&format!(
            "SELECT {} FROM {} ORDER BY updated_at DESC",
            cols, table
        ))
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    /// Returns a comic's enriched payload with comic and chapter data via JOIN.
    pub async fn find_payload_by_comic_id(
        &self, comic_directory_id: i64,
    ) -> Result<Option<ReadingHistoryPayload>, DbError> {
        let result = sqlx::query_as::<_, ReadingHistoryPayload>(
            "SELECT
                rh.comic_directory_id,
                rh.chapter_archive_id,
                rh.last_page,
                rh.is_completed,
                rh.updated_at,
                c.name  AS comic_name,
                c.cover AS comic_cover,
                ca.chapter AS chapter_name,
                c.name AS folder_name,
                ca.path AS chapter_path,
                ca.chapter_sort,
                ca.is_special,
                ca.last_modified
             FROM reading_history rh
             JOIN comic_directory c  ON rh.comic_directory_id = c.id
             JOIN chapter_archive ca ON rh.chapter_archive_id = ca.id
             WHERE rh.comic_directory_id = ?",
        )
        .bind(comic_directory_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Returns the whole enriched history, most recent first.
    pub async fn find_all_payload_ordered(&self) -> Result<Vec<ReadingHistoryPayload>, DbError> {
        let result = sqlx::query_as::<_, ReadingHistoryPayload>(
            "SELECT
                rh.comic_directory_id,
                rh.chapter_archive_id,
                rh.last_page,
                rh.is_completed,
                rh.updated_at,
                c.name  AS comic_name,
                c.cover AS comic_cover,
                ca.chapter AS chapter_name,
                c.name AS folder_name,
                ca.path AS chapter_path,
                ca.chapter_sort,
                ca.is_special,
                ca.last_modified
             FROM reading_history rh
             JOIN comic_directory c  ON rh.comic_directory_id = c.id
             JOIN chapter_archive ca ON rh.chapter_archive_id = ca.id
             ORDER BY rh.updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    /// Deletes the whole reading history.
    pub async fn delete_all(&self) -> Result<(), DbError> {
        let table = ReadingHistory::table_name();
        sqlx::query(&format!("DELETE FROM {}", table)).execute(&self.pool).await?;
        Ok(())
    }

    /// Deletes the reading history of a single comic — used when deleting the comic
    /// (the FK cascade does not run at runtime, see the comment in
    /// `VolumeRepository::delete_by_comic`). No-op if there is no history.
    pub async fn delete_by_comic_id(&self, comic_directory_id: i64) -> Result<(), DbError> {
        let table = ReadingHistory::table_name();
        sqlx::query(&format!("DELETE FROM {} WHERE comic_directory_id = ?", table))
            .bind(comic_directory_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ReadingHistoryRepository;
    use crate::{
        data::models::history::reading_history::ReadingHistory,
        tests::utils::setup_test_db::{
            insert_chapter_archive, insert_comic_directory, setup_test_db_with_comic,
        },
    };

    fn historico(
        comic_directory_id: i64, chapter_archive_id: i64, last_page: i64,
    ) -> ReadingHistory {
        ReadingHistory {
            comic_directory_id,
            chapter_archive_id,
            last_page,
            is_completed: false,
            updated_at: 1000,
        }
    }

    async fn setup() -> (sqlx::SqlitePool, ReadingHistoryRepository) {
        let pool = setup_test_db_with_comic().await;
        let repo = ReadingHistoryRepository::new(pool.clone());
        (pool, repo)
    }

    #[tokio::test]
    async fn test_upsert_insert() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let result = repo.upsert(&historico(1, 1, 5)).await.unwrap();

        assert_eq!(result.comic_directory_id, 1);
        assert_eq!(result.last_page, 5);
    }

    #[tokio::test]
    async fn test_upsert_updates_existing() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, '2', 'path', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.upsert(&historico(1, 1, 5)).await.unwrap();

        let atualizado = ReadingHistory {
            comic_directory_id: 1,
            chapter_archive_id: 2,
            last_page: 2,
            is_completed: true,
            updated_at: 2000,
        };
        let result = repo.upsert(&atualizado).await.unwrap();

        assert_eq!(result.chapter_archive_id, 2);
        assert_eq!(result.last_page, 2);
        assert!(result.is_completed);
    }

    #[tokio::test]
    async fn test_find_by_comic_id() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.upsert(&historico(1, 1, 5)).await.unwrap();

        let result = repo.find_by_comic_id(1).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().last_page, 5);
    }

    #[tokio::test]
    async fn test_find_by_comic_id_nonexistent() {
        let (_, repo) = setup().await;
        let result = repo.find_by_comic_id(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_all() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.upsert(&historico(1, 1, 5)).await.unwrap();

        let before = repo.find_all_ordered().await.unwrap();
        assert_eq!(before.len(), 1);

        repo.delete_all().await.unwrap();

        let after = repo.find_all_ordered().await.unwrap();
        assert_eq!(after.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_by_comic_id_removes_only_that_comic() {
        let (pool, repo) = setup().await;

        insert_comic_directory(&pool, 2, "Outro", "/outro").await;
        insert_chapter_archive(&pool, 1, 1).await;
        insert_chapter_archive(&pool, 2, 2).await;

        repo.upsert(&historico(1, 1, 5)).await.unwrap();
        repo.upsert(&historico(2, 2, 5)).await.unwrap();

        repo.delete_by_comic_id(1).await.unwrap();

        assert!(repo.find_by_comic_id(1).await.unwrap().is_none());
        assert!(repo.find_by_comic_id(2).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_delete_by_comic_id_without_history_does_not_fail() {
        let (_, repo) = setup().await;
        repo.delete_by_comic_id(999).await.unwrap();
    }
}
