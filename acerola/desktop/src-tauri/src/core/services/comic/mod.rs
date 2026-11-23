use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use crate::{
    core::services::archive::{comic_scanner_engine::ComicScannerService, cover_extractor},
    data::{
        models::archive::{comic_directory::ComicDirectory, volume_archive::VolumeArchive},
        repositories::archive::{
            chapter_archive_repo::{ChapterRepository, ChapterSortCriteria},
            comic_directory_repo::ComicRepository,
            volume_archive_repo::VolumeRepository,
        },
    },
    infra::error::ComicError,
};

/// Service for bulk and individual comic operations.
pub struct ComicService {
    repo: ComicRepository,
    chapter_repo: ChapterRepository,
    volume_repo: VolumeRepository,
    pool: SqlitePool,
}

impl ComicService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool.clone()),
            volume_repo: VolumeRepository::new(pool.clone()),
            pool,
        }
    }

    /// Rescans a single comic on demand from its current path on disk.
    pub async fn rescan_comic(&self, id: i64) -> Result<(), ComicError> {
        let comic = self
            .repo
            .find_by_id(id)
            .await
            .map_err(ComicError::from)?
            .ok_or(ComicError::NotFound)?;
        let scanner = ComicScannerService::new(PathBuf::from(&comic.path), self.pool.clone());

        scanner.rescan_comic(comic, |_| {}, |_| {}).await
    }

    /// Updates the visibility status of a specific comic.
    pub async fn update_hidden_status(
        &self, id: i64, hidden: bool,
    ) -> Result<ComicDirectory, ComicError> {
        self.repo.update_hidden_status(id, hidden).await.map_err(ComicError::from)
    }

    /// Updates the external sync status of a specific comic.
    pub async fn update_external_sync_enabled(
        &self, id: i64, external_sync_enabled: bool,
    ) -> Result<ComicDirectory, ComicError> {
        self.repo
            .update_external_sync_enabled(id, external_sync_enabled)
            .await
            .map_err(ComicError::from)
    }

    /// Updates the visibility status of multiple comics in batch.
    pub async fn update_hidden_status_batch(
        &self, ids: &[i64], hidden: bool,
    ) -> Result<usize, ComicError> {
        self.repo.update_hidden_status_batch(ids, hidden).await.map_err(ComicError::from)
    }

    /// Deletes multiple comics: the physical folder on disk (recursively) and every
    /// related row in the database, including the `comic_directory` record itself.
    ///
    /// Removal from disk is best-effort, per comic — an already-missing or locked folder
    /// does not prevent deleting the others nor the database record (same stance as
    /// `MetadataService::clear_comic_metadata`).
    ///
    /// Child tables are cleaned manually, in dependency order (children before
    /// parents), because the FK cascade declared in the migrations does not run in production —
    /// the pool never enables `PRAGMA foreign_keys` (same reason documented in
    /// `VolumeRepository::delete_by_comic`). Without this cleanup, orphan rows stay tied
    /// to the deleted comic's `id` — and since that `id` is a path hash
    /// (`path_hash`), a new comic created later at the same path would silently
    /// inherit the old comic's garbage (chapters, history, metadata).
    pub async fn delete_batch(&self, ids: &[i64]) -> Result<usize, ComicError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

        // 1. Collect the comics' physical paths before removing the database records.
        let select_sql = format!("SELECT path FROM comic_directory WHERE id IN ({})", placeholders);
        let mut query = sqlx::query_scalar::<_, String>(&select_sql);
        for &id in ids {
            query = query.bind(id);
        }
        let paths: Vec<String> = query.fetch_all(&self.pool).await.unwrap_or_default();

        // 2. Batch-clean every child table and the main records inside a single atomic
        // transaction, avoiding deadlocks and dozens of individual transactions.
        let mut tx = self.pool.begin().await?;

        let child_delete_queries = [
            format!("DELETE FROM reading_history WHERE comic_directory_id IN ({})", placeholders),
            format!("DELETE FROM chapter_read WHERE comic_directory_id IN ({})", placeholders),
            format!("DELETE FROM comic_metadata WHERE comic_directory_fk IN ({})", placeholders),
            format!("DELETE FROM chapter_archive WHERE comic_directory_fk IN ({})", placeholders),
            format!("DELETE FROM volume_archive WHERE comic_directory_fk IN ({})", placeholders),
            format!("DELETE FROM comic_category WHERE comic_directory_fk IN ({})", placeholders),
        ];

        for sql in child_delete_queries {
            let mut q = sqlx::query(&sql);
            for &id in ids {
                q = q.bind(id);
            }
            q.execute(&mut *tx).await?;
        }

        let comic_delete_sql =
            format!("DELETE FROM comic_directory WHERE id IN ({})", placeholders);
        let mut q = sqlx::query(&comic_delete_sql);
        for &id in ids {
            q = q.bind(id);
        }
        let rows_affected = q.execute(&mut *tx).await?.rows_affected() as usize;

        tx.commit().await?;

        // 3. Offload disk removal to dedicated blocking threads (spawn_blocking),
        // avoiding stalling Tokio worker threads or the main IPC thread on Windows.
        let mut handles = Vec::with_capacity(paths.len());
        for path_str in paths {
            handles.push(tokio::task::spawn_blocking(move || {
                let path = Path::new(&path_str);
                if path.exists() {
                    tracing::info!(path = %path_str, "Removing comic folder from disk");
                    if let Err(error) = std::fs::remove_dir_all(path) {
                        tracing::warn!(
                            path = %path_str,
                            error = %error,
                            "Failed to remove comic folder from disk while deleting comic"
                        );
                    } else {
                        tracing::info!(path = %path_str, "Comic folder removed from disk");
                    }
                }
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }

        Ok(rows_affected)
    }

    /// Generates a comic's cover from the first page of its first chapter
    /// (following reading order, including inside volumes), replacing
    /// any cover already set.
    pub async fn regenerate_cover(&self, id: i64) -> Result<ComicDirectory, ComicError> {
        let comic = self
            .repo
            .find_by_id(id)
            .await
            .map_err(ComicError::from)?
            .ok_or(ComicError::NotFound)?;

        let chapters = self
            .chapter_repo
            .get_chapters_by_directory(id, 1, 0, ChapterSortCriteria::NumberAsc)
            .await
            .map_err(ComicError::from)?;

        let first_chapter = chapters.into_iter().next().ok_or_else(|| {
            ComicError::SystemFailure("Comic has no chapters to generate a cover from.".to_string())
        })?;

        let cover_path =
            extract_and_persist_cover(PathBuf::from(first_chapter.path), Path::new(&comic.path))
                .await?;

        self.repo.update_cover(id, &cover_path).await.map_err(ComicError::from)
    }

    /// Generates the cover of each volume of the comic from the first page of its first
    /// chapter, replacing any cover already set. Volumes without chapters are skipped.
    pub async fn regenerate_volume_covers(
        &self, id: i64,
    ) -> Result<Vec<VolumeArchive>, ComicError> {
        self.repo.find_by_id(id).await.map_err(ComicError::from)?.ok_or(ComicError::NotFound)?;

        let volumes = self.volume_repo.find_by_comic(id).await.map_err(ComicError::from)?;

        let mut updated = Vec::new();
        for volume in volumes {
            let chapters = self
                .chapter_repo
                .get_chapters_by_volume(id, volume.id, 1, 0, ChapterSortCriteria::NumberAsc)
                .await
                .map_err(ComicError::from)?;

            let Some(first_chapter) = chapters.into_iter().next() else {
                continue;
            };

            let cover_path = extract_and_persist_cover(
                PathBuf::from(first_chapter.path),
                Path::new(&volume.path),
            )
            .await?;

            let saved = self
                .volume_repo
                .update_cover(volume.id, &cover_path)
                .await
                .map_err(ComicError::from)?;
            updated.push(saved);
        }

        Ok(updated)
    }
}

/// Extracts the first page of the given chapter file (on a blocking thread) and
/// persists it as `cover.*` inside `target_dir`.
async fn extract_and_persist_cover(
    chapter_file: PathBuf, target_dir: &Path,
) -> Result<String, ComicError> {
    let (bytes, format) =
        tokio::task::spawn_blocking(move || cover_extractor::extract_first_page(&chapter_file))
            .await
            .map_err(|error| ComicError::SystemFailure(error.to_string()))??;

    cover_extractor::persist_cover(target_dir, bytes, format).await
}

#[cfg(test)]
mod tests {
    use super::ComicService;
    use crate::tests::utils::setup_test_db::{
        insert_chapter_archive, insert_chapter_read, insert_comic_directory,
        insert_reading_history, setup_test_db,
    };

    /// Proves the very reason for this change: deleting a comic had to delete the physical
    /// folder, not just the database row (it used to be only `DELETE FROM comic_directory`, the
    /// file on disk was left untouched).
    #[tokio::test]
    async fn delete_batch_removes_the_physical_folder_from_disk() {
        let pool = setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let comic_dir = temp_dir.path().join("Quadrinho Para Excluir");
        tokio::fs::create_dir_all(&comic_dir).await.unwrap();
        tokio::fs::write(comic_dir.join("Cap 1.cbz"), b"fake cbz").await.unwrap();

        insert_comic_directory(&pool, 1, "Quadrinho Para Excluir", &comic_dir.to_string_lossy())
            .await;

        let service = ComicService::new(pool);
        let deleted = service.delete_batch(&[1]).await.unwrap();

        assert_eq!(deleted, 1);
        assert!(!comic_dir.exists(), "physical folder should have been removed");
    }

    /// An already-missing folder (user deleted it manually, or outdated path) must not
    /// prevent deleting the database record — same best-effort stance as
    /// `MetadataService::clear_comic_metadata`.
    #[tokio::test]
    async fn delete_batch_still_removes_db_row_when_folder_is_already_gone() {
        let pool = setup_test_db().await;
        insert_comic_directory(&pool, 1, "Quadrinho Fantasma", "/caminho/que/nao/existe").await;

        let service = ComicService::new(pool.clone());
        let deleted = service.delete_batch(&[1]).await.unwrap();

        assert_eq!(deleted, 1);
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comic_directory").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 0);
    }

    /// Regression for the orphan row bug: since the FK cascade does not run in production,
    /// deleting a comic without manually cleaning the child tables left chapter/history/read
    /// rows tied to the old `id` — and since that `id` is a path hash, a new comic created
    /// later at the same path would silently inherit that garbage.
    #[tokio::test]
    async fn delete_batch_cleans_up_orphaned_child_rows() {
        let pool = setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let comic_dir = temp_dir.path().join("Quadrinho Com Historico");
        tokio::fs::create_dir_all(&comic_dir).await.unwrap();

        insert_comic_directory(&pool, 1, "Quadrinho Com Historico", &comic_dir.to_string_lossy())
            .await;
        insert_chapter_archive(&pool, 1, 1).await;
        insert_reading_history(&pool, 1, 1, 5).await;
        insert_chapter_read(&pool, 1, 1).await;

        let service = ComicService::new(pool.clone());
        service.delete_batch(&[1]).await.unwrap();

        let chapters: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chapter_archive").fetch_one(&pool).await.unwrap();
        let history: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM reading_history").fetch_one(&pool).await.unwrap();
        let read: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chapter_read").fetch_one(&pool).await.unwrap();

        assert_eq!(chapters.0, 0, "orphan chapter was not cleaned up");
        assert_eq!(history.0, 0, "orphan reading history was not cleaned up");
        assert_eq!(read.0, 0, "orphan read marker was not cleaned up");
    }
}
