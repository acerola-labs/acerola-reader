use sqlx::SqlitePool;

use crate::{
    data::{
        models::history::chapter_read::ChapterRead,
        repositories::{Entity, Repository},
    },
    infra::error::DbError,
};

#[derive(Clone)]
pub struct ChapterReadRepository {
    pub base: Repository<ChapterRead>,
    pool: SqlitePool,
}

impl ChapterReadRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    /// Marks a chapter as read. Silently ignored if it already exists
    /// (composite key `comic_directory_id` + `chapter_archive_id`).
    pub async fn insert_or_ignore(&self, chapter_read: &ChapterRead) -> Result<(), DbError> {
        let cols = ChapterRead::columns().join(", ");
        let table = ChapterRead::table_name();

        sqlx::query(&format!(
            "INSERT INTO {} ({})
             VALUES (?, ?, ?)
             ON CONFLICT(comic_directory_id, chapter_archive_id) DO NOTHING",
            table, cols
        ))
        .bind(chapter_read.comic_directory_id)
        .bind(chapter_read.chapter_archive_id)
        .bind(chapter_read.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Returns the IDs of a comic's read chapters.
    pub async fn find_ids_by_comic(&self, comic_directory_id: i64) -> Result<Vec<i64>, DbError> {
        let table = ChapterRead::table_name();

        let rows = sqlx::query_as::<_, (i64,)>(&format!(
            "SELECT chapter_archive_id FROM {} WHERE comic_directory_id = ?",
            table
        ))
        .bind(comic_directory_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Removes a chapter's read record (marks it as unread). Does not use the
    /// base abstraction (`Repository::delete`, which deletes by `id`): here the key is
    /// `comic_directory_id` + `chapter_archive_id`, hence the more explicit name.
    pub async fn delete_by_comic_and_chapter(
        &self, comic_directory_id: i64, chapter_archive_id: i64,
    ) -> Result<(), DbError> {
        let table = ChapterRead::table_name();

        sqlx::query(&format!(
            "DELETE FROM {} WHERE comic_directory_id = ? AND chapter_archive_id = ?",
            table
        ))
        .bind(comic_directory_id)
        .bind(chapter_archive_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Marks multiple chapters as read in a single query. Silently ignores
    /// those that already exist.
    pub async fn insert_batch(
        &self, comic_directory_id: i64, chapter_ids: &[i64], created_at: i64,
    ) -> Result<usize, DbError> {
        if chapter_ids.is_empty() {
            return Ok(0);
        }

        let table = ChapterRead::table_name();
        let cols = ChapterRead::columns().join(", ");
        let placeholders = chapter_ids.iter().map(|_| "(?, ?, ?)").collect::<Vec<_>>().join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {} ON CONFLICT(comic_directory_id, chapter_archive_id) DO NOTHING",
            table, cols, placeholders
        );

        let mut query = sqlx::query(&sql);
        for &chapter_id in chapter_ids {
            query = query.bind(comic_directory_id).bind(chapter_id).bind(created_at);
        }

        let rows_affected = query.execute(&self.pool).await?.rows_affected();
        Ok(rows_affected as usize)
    }

    /// Returns every "read" marker already translated to the natural key (comic name +
    /// chapter `chapter_sort`), used by P2P sync to build the local manifest — autoincrement
    /// IDs are not comparable across SQLite databases of different devices. Uses
    /// `chapter_sort`, not the label (`chapter`): the label is not comparable across devices
    /// (each one names the file however it wants), `chapter_sort` is.
    pub async fn find_all_with_natural_keys(&self) -> Result<Vec<(String, String, i64)>, DbError> {
        let rows = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT c.name, ca.chapter_sort, cr.created_at
             FROM chapter_read cr
             JOIN comic_directory c  ON cr.comic_directory_id = c.id
             JOIN chapter_archive ca ON cr.chapter_archive_id = ca.id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Same as [`Self::find_all_with_natural_keys`], but restricted to ONE comic and a
    /// subset of `chapter_sort` — used by the explicit push of selected chapter(s) to a
    /// peer (`acerola/sync-history-entry/1`), which must not leak markers of other
    /// chapters of the same comic.
    pub async fn find_natural_keys_for_chapters(
        &self, comic_directory_id: i64, chapter_sorts: &[String],
    ) -> Result<Vec<(String, i64)>, DbError> {
        if chapter_sorts.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = chapter_sorts.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT ca.chapter_sort, cr.created_at
             FROM chapter_read cr
             JOIN chapter_archive ca ON cr.chapter_archive_id = ca.id
             WHERE cr.comic_directory_id = ? AND ca.chapter_sort IN ({})",
            placeholders
        );

        let mut query = sqlx::query_as::<_, (String, i64)>(&sql).bind(comic_directory_id);
        for sort in chapter_sorts {
            query = query.bind(sort);
        }

        Ok(query.fetch_all(&self.pool).await?)
    }

    /// Removes every "read" marker of a whole comic — used when deleting the comic
    /// (the FK cascade does not run at runtime, see the comment in
    /// `VolumeRepository::delete_by_comic`). No-op if no chapter is marked.
    pub async fn delete_by_comic(&self, comic_directory_id: i64) -> Result<u64, DbError> {
        let table = ChapterRead::table_name();

        let rows_affected =
            sqlx::query(&format!("DELETE FROM {} WHERE comic_directory_id = ?", table))
                .bind(comic_directory_id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        Ok(rows_affected)
    }

    /// Removes the read record of multiple chapters in batch.
    pub async fn delete_batch(
        &self, comic_directory_id: i64, chapter_ids: &[i64],
    ) -> Result<usize, DbError> {
        if chapter_ids.is_empty() {
            return Ok(0);
        }

        let table = ChapterRead::table_name();
        let placeholders = chapter_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        let sql = format!(
            "DELETE FROM {} WHERE comic_directory_id = ? AND chapter_archive_id IN ({})",
            table, placeholders
        );

        let mut query = sqlx::query(&sql).bind(comic_directory_id);
        for &chapter_id in chapter_ids {
            query = query.bind(chapter_id);
        }

        let rows_affected = query.execute(&self.pool).await?.rows_affected();
        Ok(rows_affected as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::ChapterReadRepository;
    use crate::{
        data::models::history::chapter_read::ChapterRead,
        tests::utils::setup_test_db::{insert_comic_directory, setup_test_db_with_comic},
    };

    fn read_chapter(comic_directory_id: i64, chapter_archive_id: i64) -> ChapterRead {
        ChapterRead { comic_directory_id, chapter_archive_id, created_at: 1000 }
    }

    async fn setup() -> (sqlx::SqlitePool, ChapterReadRepository) {
        let pool = setup_test_db_with_comic().await;
        let repo = ChapterReadRepository::new(pool.clone());
        (pool, repo)
    }

    #[tokio::test]
    async fn test_insert_or_ignore_records_read() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.insert_or_ignore(&read_chapter(1, 1)).await.unwrap();

        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 1);
    }

    #[tokio::test]
    async fn test_insert_or_ignore_does_not_duplicate() {
        let (pool, repo) = setup().await;

        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, '1', 'path', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.insert_or_ignore(&read_chapter(1, 1)).await.unwrap();
        repo.insert_or_ignore(&read_chapter(1, 1)).await.unwrap();

        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_find_ids_by_comic_empty() {
        let (_, repo) = setup().await;
        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert!(ids.is_empty());
    }

    async fn insert_chapters(pool: &sqlx::SqlitePool, ids: &[i64]) {
        for &id in ids {
            sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (?, ?, ?, ?, 0, 1, 0)")
                .bind(id)
                .bind(format!("{}", id))
                .bind(format!("path{}", id))
                .bind(format!("{}", id))
                .execute(pool)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_delete_removes_read_chapter() {
        let (pool, repo) = setup().await;
        insert_chapters(&pool, &[1]).await;

        repo.insert_or_ignore(&read_chapter(1, 1)).await.unwrap();
        assert_eq!(repo.find_ids_by_comic(1).await.unwrap().len(), 1);

        repo.delete_by_comic_and_chapter(1, 1).await.unwrap();
        assert!(repo.find_ids_by_comic(1).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_chapter_does_not_fail() {
        let (_, repo) = setup().await;
        repo.delete_by_comic_and_chapter(1, 999).await.unwrap();
    }

    #[tokio::test]
    async fn test_insert_batch_records_multiple() {
        let (pool, repo) = setup().await;
        insert_chapters(&pool, &[1, 2, 3]).await;

        let count = repo.insert_batch(1, &[1, 2, 3], 1000).await.unwrap();
        assert_eq!(count, 3);

        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn test_insert_batch_ignores_duplicates() {
        let (pool, repo) = setup().await;
        insert_chapters(&pool, &[1, 2]).await;

        repo.insert_batch(1, &[1], 1000).await.unwrap();
        let count = repo.insert_batch(1, &[1, 2], 1000).await.unwrap();

        assert_eq!(count, 1);
        assert_eq!(repo.find_ids_by_comic(1).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_insert_batch_empty_returns_zero() {
        let (_, repo) = setup().await;
        let count = repo.insert_batch(1, &[], 1000).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete_batch_removes_multiple() {
        let (pool, repo) = setup().await;
        insert_chapters(&pool, &[1, 2, 3]).await;
        repo.insert_batch(1, &[1, 2, 3], 1000).await.unwrap();

        let count = repo.delete_batch(1, &[1, 2]).await.unwrap();

        assert_eq!(count, 2);
        let ids = repo.find_ids_by_comic(1).await.unwrap();
        assert_eq!(ids, vec![3]);
    }

    #[tokio::test]
    async fn test_delete_batch_empty_returns_zero() {
        let (_, repo) = setup().await;
        let count = repo.delete_batch(1, &[]).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete_by_comic_removes_all_markers_for_that_comic_only() {
        let (pool, repo) = setup().await;
        insert_comic_directory(&pool, 2, "Outro", "/outro").await;
        insert_chapters(&pool, &[1, 2]).await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (3, '3', 'path3', '3', 0, 2, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.insert_batch(1, &[1, 2], 1000).await.unwrap();
        repo.insert_batch(2, &[3], 1000).await.unwrap();

        let removed = repo.delete_by_comic(1).await.unwrap();

        assert_eq!(removed, 2);
        assert!(repo.find_ids_by_comic(1).await.unwrap().is_empty());
        assert_eq!(repo.find_ids_by_comic(2).await.unwrap(), vec![3]);
    }

    #[tokio::test]
    async fn test_delete_by_comic_without_markers_does_not_fail() {
        let (_, repo) = setup().await;
        let removed = repo.delete_by_comic(999).await.unwrap();
        assert_eq!(removed, 0);
    }

    #[tokio::test]
    async fn test_find_natural_keys_for_chapters_filters_by_comic_and_sort() {
        let (pool, repo) = setup().await;
        insert_comic_directory(&pool, 2, "Outro", "/outro").await;
        insert_chapters(&pool, &[1, 2]).await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (3, '3', 'path3', '3', 0, 2, 0)")
            .execute(&pool)
            .await
            .unwrap();

        repo.insert_batch(1, &[1, 2], 1000).await.unwrap();
        repo.insert_batch(2, &[3], 1000).await.unwrap();

        let keys = repo.find_natural_keys_for_chapters(1, &["1".to_string()]).await.unwrap();
        assert_eq!(keys, vec![("1".to_string(), 1000)]);
    }

    #[tokio::test]
    async fn test_find_natural_keys_for_chapters_empty_list_returns_empty() {
        let (_, repo) = setup().await;
        let keys = repo.find_natural_keys_for_chapters(1, &[]).await.unwrap();
        assert!(keys.is_empty());
    }
}
