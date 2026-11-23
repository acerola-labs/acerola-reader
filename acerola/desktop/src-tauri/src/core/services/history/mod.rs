use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::SqlitePool;

use crate::{
    cmd::events::history::ReadingHistoryPayload,
    data::{
        models::history::{chapter_read::ChapterRead, reading_history::ReadingHistory},
        repositories::history::{
            chapter_read_repo::ChapterReadRepository,
            reading_history_repo::ReadingHistoryRepository,
        },
    },
    infra::error::DbError,
};

#[derive(Clone)]
pub struct HistoryService {
    reading_repo: ReadingHistoryRepository,
    chapter_repo: ChapterReadRepository,
}

impl HistoryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            reading_repo: ReadingHistoryRepository::new(pool.clone()),
            chapter_repo: ChapterReadRepository::new(pool),
        }
    }

    /// Updates a comic's reading progress. If `is_completed`, also
    /// records the chapter as read in the `chapter_read` table.
    pub async fn update_progress(
        &self, comic_directory_id: i64, chapter_archive_id: i64, last_page: i64, is_completed: bool,
    ) -> Result<ReadingHistory, DbError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        let history = ReadingHistory {
            comic_directory_id,
            chapter_archive_id,
            last_page,
            is_completed,
            updated_at: now,
        };

        let result = self.reading_repo.upsert(&history).await?;

        if is_completed {
            let chapter_read_entry =
                ChapterRead { comic_directory_id, chapter_archive_id, created_at: now };
            self.chapter_repo.insert_or_ignore(&chapter_read_entry).await?;
        }

        Ok(result)
    }

    /// Returns a comic's enriched reading payload.
    pub async fn find_by_comic(
        &self, comic_directory_id: i64,
    ) -> Result<Option<ReadingHistoryPayload>, DbError> {
        self.reading_repo.find_payload_by_comic_id(comic_directory_id).await
    }

    /// Returns the whole enriched history, most recent first.
    pub async fn find_all(&self) -> Result<Vec<ReadingHistoryPayload>, DbError> {
        self.reading_repo.find_all_payload_ordered().await
    }

    /// Deletes the whole reading history.
    pub async fn clear(&self) -> Result<(), DbError> {
        self.reading_repo.delete_all().await
    }

    /// Returns the IDs of a comic's read chapters.
    pub async fn find_read_chapters(&self, comic_directory_id: i64) -> Result<Vec<i64>, DbError> {
        self.chapter_repo.find_ids_by_comic(comic_directory_id).await
    }

    /// Explicitly marks a chapter as read — writes both to `chapter_read` (checkmark in the
    /// chapter list) and to `reading_history` (the same table `update_progress` uses),
    /// otherwise the comic never shows up/updates on the History page until the user actually
    /// opens the reader. `last_page` keeps the already-saved value if the existing entry is
    /// already for this same chapter (real reading progress does not regress); falls back to `0`
    /// when there is no previous entry or it points to another chapter — marking as read from the
    /// list has no real page associated.
    pub async fn mark_chapter_read(
        &self, comic_directory_id: i64, chapter_archive_id: i64,
    ) -> Result<(), DbError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let chapter_read_entry =
            ChapterRead { comic_directory_id, chapter_archive_id, created_at: now };
        self.chapter_repo.insert_or_ignore(&chapter_read_entry).await?;

        let last_page = self
            .reading_repo
            .find_by_comic_id(comic_directory_id)
            .await?
            .filter(|existing| existing.chapter_archive_id == chapter_archive_id)
            .map(|existing| existing.last_page)
            .unwrap_or(0);

        self.reading_repo
            .upsert(&ReadingHistory {
                comic_directory_id,
                chapter_archive_id,
                last_page,
                is_completed: true,
                updated_at: now,
            })
            .await?;

        Ok(())
    }

    /// Unmarks a chapter as read.
    pub async fn unmark_chapter_read(
        &self, comic_directory_id: i64, chapter_archive_id: i64,
    ) -> Result<(), DbError> {
        self.chapter_repo.delete_by_comic_and_chapter(comic_directory_id, chapter_archive_id).await
    }

    /// Marks multiple chapters as read in batch — same care as [`Self::mark_chapter_read`]
    /// with `reading_history`, otherwise the comic never shows up/updates on the History page.
    /// Since a batch can mark several chapters at once without a clear notion of "which one is
    /// the current", it uses the last item of `chapter_ids` (order decided by the caller, e.g.
    /// multi-selection on screen) as the chapter that now represents this comic's progress.
    pub async fn mark_chapters_read_batch(
        &self, comic_directory_id: i64, chapter_ids: &[i64],
    ) -> Result<usize, DbError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let marked = self.chapter_repo.insert_batch(comic_directory_id, chapter_ids, now).await?;

        if let Some(&chapter_archive_id) = chapter_ids.last() {
            let last_page = self
                .reading_repo
                .find_by_comic_id(comic_directory_id)
                .await?
                .filter(|existing| existing.chapter_archive_id == chapter_archive_id)
                .map(|existing| existing.last_page)
                .unwrap_or(0);

            self.reading_repo
                .upsert(&ReadingHistory {
                    comic_directory_id,
                    chapter_archive_id,
                    last_page,
                    is_completed: true,
                    updated_at: now,
                })
                .await?;
        }

        Ok(marked)
    }

    /// Unmarks multiple chapters as read in batch.
    pub async fn unmark_chapters_read_batch(
        &self, comic_directory_id: i64, chapter_ids: &[i64],
    ) -> Result<usize, DbError> {
        self.chapter_repo.delete_batch(comic_directory_id, chapter_ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::HistoryService;
    use crate::tests::utils::setup_test_db::{insert_chapter_archive, setup_test_db_with_comic};

    async fn setup() -> (sqlx::SqlitePool, HistoryService) {
        let pool = setup_test_db_with_comic().await;
        let service = HistoryService::new(pool.clone());
        (pool, service)
    }

    #[tokio::test]
    async fn test_updates_reading_progress() {
        let (pool, service) = setup().await;

        insert_chapter_archive(&pool, 1, 1).await;

        let result = service.update_progress(1, 1, 10, false).await.unwrap();

        assert_eq!(result.comic_directory_id, 1);
        assert_eq!(result.chapter_archive_id, 1);
        assert_eq!(result.last_page, 10);
        assert!(!result.is_completed);
    }

    #[tokio::test]
    async fn test_marks_and_unmarks_chapter_as_read() {
        let (pool, service) = setup().await;

        insert_chapter_archive(&pool, 1, 1).await;

        service.mark_chapter_read(1, 1).await.unwrap();
        assert_eq!(service.find_read_chapters(1).await.unwrap(), vec![1]);

        service.unmark_chapter_read(1, 1).await.unwrap();
        assert!(service.find_read_chapters(1).await.unwrap().is_empty());
    }

    /// Regression: marking a chapter as read from the list (without going through the reader)
    /// had to update `reading_history` too, otherwise the comic never showed up/updated on the
    /// History page.
    #[tokio::test]
    async fn mark_chapter_read_also_updates_reading_history() {
        let (pool, service) = setup().await;
        insert_chapter_archive(&pool, 1, 1).await;

        service.mark_chapter_read(1, 1).await.unwrap();

        let history = service.reading_repo.find_by_comic_id(1).await.unwrap().unwrap();
        assert_eq!(history.chapter_archive_id, 1);
        assert!(history.is_completed);
        assert_eq!(history.last_page, 0);
    }

    /// If the chapter marked as read is already the one `reading_history` points to (the user
    /// was actually reading and reached page 42 before marking it manually), the real
    /// progress must not regress to page 0.
    #[tokio::test]
    async fn mark_chapter_read_preserves_last_page_of_the_same_chapter() {
        let (pool, service) = setup().await;
        insert_chapter_archive(&pool, 1, 1).await;
        service.update_progress(1, 1, 42, false).await.unwrap();

        service.mark_chapter_read(1, 1).await.unwrap();

        let history = service.reading_repo.find_by_comic_id(1).await.unwrap().unwrap();
        assert_eq!(history.last_page, 42);
        assert!(history.is_completed);
    }

    /// Same regression as [`mark_chapter_read_also_updates_reading_history`], for the path of
    /// marking several chapters at once — uses the last id of the list as the chapter that now
    /// represents the comic's progress.
    #[tokio::test]
    async fn mark_chapters_read_batch_also_updates_reading_history() {
        let (pool, service) = setup().await;
        for id in 1..=3 {
            insert_chapter_archive(&pool, id, 1).await;
        }

        service.mark_chapters_read_batch(1, &[1, 2, 3]).await.unwrap();

        let history = service.reading_repo.find_by_comic_id(1).await.unwrap().unwrap();
        assert_eq!(history.chapter_archive_id, 3);
        assert!(history.is_completed);
    }

    #[tokio::test]
    async fn test_marks_and_unmarks_chapters_in_batch() {
        let (pool, service) = setup().await;

        for id in 1..=3 {
            insert_chapter_archive(&pool, id, 1).await;
        }

        let marked = service.mark_chapters_read_batch(1, &[1, 2, 3]).await.unwrap();
        assert_eq!(marked, 3);
        assert_eq!(service.find_read_chapters(1).await.unwrap().len(), 3);

        let unmarked = service.unmark_chapters_read_batch(1, &[1, 2]).await.unwrap();
        assert_eq!(unmarked, 2);
        assert_eq!(service.find_read_chapters(1).await.unwrap(), vec![3]);
    }

    #[tokio::test]
    async fn test_clears_history() {
        let (pool, service) = setup().await;

        insert_chapter_archive(&pool, 1, 1).await;

        service.update_progress(1, 1, 10, false).await.unwrap();

        let before = service.reading_repo.find_all_ordered().await.unwrap();
        assert_eq!(before.len(), 1);

        service.clear().await.unwrap();

        let after = service.reading_repo.find_all_ordered().await.unwrap();
        assert_eq!(after.len(), 0);
    }
}
