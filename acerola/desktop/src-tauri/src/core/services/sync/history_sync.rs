use sqlx::SqlitePool;

use crate::{
    data::{
        models::history::{chapter_read::ChapterRead, reading_history::ReadingHistory},
        repositories::{
            archive::{
                chapter_archive_repo::ChapterRepository, comic_directory_repo::ComicRepository,
            },
            history::{
                chapter_read_repo::ChapterReadRepository,
                reading_history_repo::ReadingHistoryRepository,
            },
        },
    },
    infra::{
        error::ComicError,
        sync::messages::{HistoryEntry, HistoryManifest, ReadMarker},
    },
};

/// Builds and applies reading history manifests between two devices.
///
/// The cross-device identity is the natural key `(comic.name, chapter.chapter_sort)` —
/// **not** the chapter label (`chapter`). The label is free text per device (each one
/// names the file however it wants: "Cap 1", "Capítulo 01", etc.), so it does not work as a
/// key between two independent devices; `chapter_sort` is the normalized value both
/// sides derive from the file name, and it is also what Android uses to resolve the local
/// `chapter_archive_id` after applying a received manifest — using anything else
/// here makes history sync "successfully" but write to the wrong chapter (or to
/// none).
#[derive(Clone)]
pub struct HistorySyncService {
    comic_repo: ComicRepository,
    chapter_repo: ChapterRepository,
    reading_repo: ReadingHistoryRepository,
    read_marker_repo: ChapterReadRepository,
}

impl HistorySyncService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            comic_repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool.clone()),
            reading_repo: ReadingHistoryRepository::new(pool.clone()),
            read_marker_repo: ChapterReadRepository::new(pool),
        }
    }

    /// Builds the full local manifest (reading progress + read markers).
    pub async fn build_manifest(&self) -> Result<HistoryManifest, ComicError> {
        let entries = self
            .reading_repo
            .find_all_payload_ordered()
            .await?
            .into_iter()
            .map(|payload| HistoryEntry {
                comic_name: payload.comic_name,
                chapter: payload.chapter_sort,
                last_page: payload.last_page,
                is_completed: payload.is_completed,
                updated_at: payload.updated_at,
            })
            .collect();

        let read_markers = self
            .read_marker_repo
            .find_all_with_natural_keys()
            .await?
            .into_iter()
            .map(|(comic_name, chapter, created_at)| ReadMarker { comic_name, chapter, created_at })
            .collect();

        Ok(HistoryManifest { entries, read_markers })
    }

    /// Builds the progress entry of ONE single comic, without "read" markers — used by
    /// the individual push (`acerola/sync-history-entry/1`), which exists so the whole library
    /// manifest is not needed just to carry the progress of a comic that just changed.
    /// `None` when the comic does not exist locally or never had saved progress.
    pub async fn build_entry_for_comic(
        &self, comic_name: &str,
    ) -> Result<Option<HistoryEntry>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };
        let Some(progress) = self.reading_repo.find_by_comic_id(comic.id).await? else {
            return Ok(None);
        };
        let Some(chapter) = self.chapter_repo.find_by_id(progress.chapter_archive_id).await? else {
            return Ok(None);
        };

        Ok(Some(HistoryEntry {
            comic_name: comic_name.to_string(),
            chapter: chapter.chapter_sort,
            last_page: progress.last_page,
            is_completed: progress.is_completed,
            updated_at: progress.updated_at,
        }))
    }

    /// Builds the manifest restricted to a subset of chapters of ONE comic — used by the
    /// explicit send of selected chapter(s) to a peer (`acerola/sync-history-entry/1`).
    /// Receives `chapter_archive_id`s (the same unit as the rest of the local UI, e.g.
    /// `markChaptersReadBatch`) and resolves them to `chapter_sort` via [`Self::resolve_chapter_sorts`] —
    /// only the latter is comparable between both devices, so it is what actually travels in
    /// the manifest. IDs that do not exist or belong to another comic are silently
    /// ignored.
    pub async fn build_manifest_for_chapters(
        &self, comic_name: &str, chapter_ids: &[i64],
    ) -> Result<HistoryManifest, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(HistoryManifest { entries: vec![], read_markers: vec![] });
        };

        let chapter_sorts = self.resolve_chapter_sorts(comic_name, chapter_ids).await?;

        let entries = self
            .current_progress_entry_if_selected(comic_name, comic.id, &chapter_sorts)
            .await?
            .into_iter()
            .collect();

        let read_markers = self
            .read_marker_repo
            .find_natural_keys_for_chapters(comic.id, &chapter_sorts)
            .await?
            .into_iter()
            .map(|(chapter, created_at)| ReadMarker {
                comic_name: comic_name.to_string(),
                chapter,
                created_at,
            })
            .collect();

        Ok(HistoryManifest { entries, read_markers })
    }

    /// Progress entry of a comic's "current" chapter (the one in `reading_history`), only if
    /// that chapter is among the selected `chapter_sorts` — `None` on any failing guard
    /// (no saved progress, chapter not found, or chapter not selected).
    /// Extracted from [`Self::build_manifest_for_chapters`] to keep chained guards instead of
    /// nested `if let`.
    async fn current_progress_entry_if_selected(
        &self, comic_name: &str, comic_id: i64, chapter_sorts: &[String],
    ) -> Result<Option<HistoryEntry>, ComicError> {
        let Some(progress) = self.reading_repo.find_by_comic_id(comic_id).await? else {
            return Ok(None);
        };
        let Some(chapter) = self.chapter_repo.find_by_id(progress.chapter_archive_id).await? else {
            return Ok(None);
        };
        if !chapter_sorts.iter().any(|sort| sort == &chapter.chapter_sort) {
            return Ok(None);
        }

        Ok(Some(HistoryEntry {
            comic_name: comic_name.to_string(),
            chapter: chapter.chapter_sort,
            last_page: progress.last_page,
            is_completed: progress.is_completed,
            updated_at: progress.updated_at,
        }))
    }

    /// Resolves local `chapter_archive_id`s to `chapter_sort` (the key comparable across
    /// devices) — used both by [`Self::build_manifest_for_chapters`] and by the
    /// `acerola/sync-history-entry/1` protocol to build the `HistoryEntryRequest` (first message
    /// of the contract, declaring the scope before the manifest itself). IDs that do not exist or
    /// belong to another comic are silently ignored; a nonexistent comic
    /// returns an empty list.
    pub async fn resolve_chapter_sorts(
        &self, comic_name: &str, chapter_ids: &[i64],
    ) -> Result<Vec<String>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(vec![]);
        };

        let mut chapter_sorts = Vec::with_capacity(chapter_ids.len());
        for &chapter_id in chapter_ids {
            let Some(chapter) = self.chapter_repo.find_by_id(chapter_id).await? else {
                continue;
            };
            if chapter.comic_directory_fk != comic.id {
                continue;
            }

            chapter_sorts.push(chapter.chapter_sort);
        }

        Ok(chapter_sorts)
    }

    /// Applies the manifest received from the peer locally: last-write-wins by `updated_at`
    /// for reading progress, union for "read" markers. Only touches comics that
    /// already exist locally — history does not create new comics, that is file sync's
    /// job. Returns how many entries/markers were actually applied — that is what gives the
    /// caller (`acerola/sync-history-entry/1`) a real validation signal, instead of "the
    /// session did not drop" (see `HistoryEntryAck`).
    pub async fn apply_manifest(
        &self, manifest: &HistoryManifest,
    ) -> Result<ApplyManifestStats, ComicError> {
        let mut entries_applied = 0u32;
        let mut markers_applied = 0u32;

        for entry in &manifest.entries {
            let Some(comic) = self.comic_repo.find_by_name(&entry.comic_name).await? else {
                continue;
            };
            let Some(chapter) =
                self.chapter_repo.find_by_comic_and_chapter_sort(comic.id, &entry.chapter).await?
            else {
                continue;
            };

            let local = self.reading_repo.find_by_comic_id(comic.id).await?;
            let should_apply = match &local {
                Some(existing) => entry.updated_at > existing.updated_at,
                None => true,
            };

            if !should_apply {
                continue;
            }

            self.reading_repo
                .upsert(&ReadingHistory {
                    comic_directory_id: comic.id,
                    chapter_archive_id: chapter.id,
                    last_page: entry.last_page,
                    is_completed: entry.is_completed,
                    updated_at: entry.updated_at,
                })
                .await?;

            entries_applied += 1;
        }

        for marker in &manifest.read_markers {
            let Some(comic) = self.comic_repo.find_by_name(&marker.comic_name).await? else {
                continue;
            };
            let Some(chapter) =
                self.chapter_repo.find_by_comic_and_chapter_sort(comic.id, &marker.chapter).await?
            else {
                continue;
            };

            self.read_marker_repo
                .insert_or_ignore(&ChapterRead {
                    comic_directory_id: comic.id,
                    chapter_archive_id: chapter.id,
                    created_at: marker.created_at,
                })
                .await?;

            markers_applied += 1;
        }

        Ok(ApplyManifestStats { entries_applied, markers_applied })
    }

    /// Only checks that the comic exists (without touching progress/read) — used by the
    /// `acerola/sync-history-entry/1` contract to validate BEFORE answering "applied
    /// successfully": without this, an empty manifest (no selected chapter had progress/
    /// marker) gave no error signal even when the peer never had that comic.
    pub async fn comic_exists(&self, comic_name: &str) -> Result<bool, ComicError> {
        Ok(self.comic_repo.find_by_name(comic_name).await?.is_some())
    }
}

/// Result of [`HistorySyncService::apply_manifest`] — how many progress entries and
/// how many "read" markers were actually written locally (not just "the session had no
/// error"). Mirrored on Android by `HistorySyncStats` (`protocol/history/model.rs`), which
/// already had this richer shape — this type aligns Desktop with the same pattern.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyManifestStats {
    pub entries_applied: u32,
    pub markers_applied: u32,
}

#[cfg(test)]
mod tests {
    use super::HistorySyncService;
    use crate::{
        data::{
            models::history::reading_history::ReadingHistory,
            repositories::history::reading_history_repo::ReadingHistoryRepository,
        },
        infra::sync::messages::{HistoryEntry, HistoryManifest, ReadMarker},
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    async fn setup() -> (sqlx::SqlitePool, HistorySyncService) {
        let pool = setup_test_db_with_comic().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', 'p', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        (pool.clone(), HistorySyncService::new(pool))
    }

    #[tokio::test]
    async fn build_manifest_reflects_local_history() {
        let (pool, service) = setup().await;
        ReadingHistoryRepository::new(pool)
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 5,
                is_completed: false,
                updated_at: 1000,
            })
            .await
            .unwrap();

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].comic_name, "Test");
        // Loads chapter_sort ("1"), not the label ("Cap 1") — it is the cross-device key.
        assert_eq!(manifest.entries[0].chapter, "1");
    }

    #[tokio::test]
    async fn apply_manifest_ignores_older_entry() {
        let (pool, service) = setup().await;
        ReadingHistoryRepository::new(pool)
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 20,
                is_completed: false,
                updated_at: 5000,
            })
            .await
            .unwrap();

        let peer_manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Test".into(),
                chapter: "1".into(),
                last_page: 1,
                is_completed: false,
                updated_at: 100,
            }],
            read_markers: vec![],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied.entries_applied, 0);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries[0].last_page, 20);
    }

    #[tokio::test]
    async fn apply_manifest_applies_newer_entry() {
        let (pool, service) = setup().await;
        ReadingHistoryRepository::new(pool)
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 1,
                is_completed: false,
                updated_at: 100,
            })
            .await
            .unwrap();

        let peer_manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Test".into(),
                chapter: "1".into(),
                last_page: 30,
                is_completed: true,
                updated_at: 9000,
            }],
            read_markers: vec![],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied.entries_applied, 1);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.entries[0].last_page, 30);
        assert!(manifest.entries[0].is_completed);
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_includes_progress_only_when_selected() {
        let (pool, service) = setup().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        ReadingHistoryRepository::new(pool.clone())
            .upsert(&ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 5,
                is_completed: false,
                updated_at: 1000,
            })
            .await
            .unwrap();

        let not_selected = service.build_manifest_for_chapters("Test", &[2]).await.unwrap();
        assert!(not_selected.entries.is_empty());

        let selected = service.build_manifest_for_chapters("Test", &[1]).await.unwrap();
        assert_eq!(selected.entries.len(), 1);
        assert_eq!(selected.entries[0].chapter, "1");
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_only_includes_selected_read_markers() {
        use crate::data::repositories::history::chapter_read_repo::ChapterReadRepository;

        let (pool, service) = setup().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let read_repo = ChapterReadRepository::new(pool.clone());
        read_repo.insert_batch(1, &[1, 2], 500).await.unwrap();

        let manifest = service.build_manifest_for_chapters("Test", &[1]).await.unwrap();

        assert_eq!(manifest.read_markers.len(), 1);
        assert_eq!(manifest.read_markers[0].chapter, "1");
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_ignores_id_belonging_to_another_comic() {
        use crate::{
            data::repositories::history::chapter_read_repo::ChapterReadRepository,
            tests::utils::setup_test_db::insert_comic_directory,
        };

        let (pool, service) = setup().await;
        insert_comic_directory(&pool, 2, "Outro", "/outro").await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, '1', 'p', '1', 0, 2, 0)")
            .execute(&pool)
            .await
            .unwrap();
        ChapterReadRepository::new(pool.clone()).insert_batch(2, &[2], 500).await.unwrap();

        let manifest = service.build_manifest_for_chapters("Test", &[2]).await.unwrap();

        assert!(manifest.read_markers.is_empty());
    }

    #[tokio::test]
    async fn build_manifest_for_chapters_unknown_comic_returns_empty_manifest() {
        let (_, service) = setup().await;

        let manifest = service.build_manifest_for_chapters("Nao Existe", &[1]).await.unwrap();

        assert!(manifest.entries.is_empty());
        assert!(manifest.read_markers.is_empty());
    }

    #[tokio::test]
    async fn apply_manifest_ignores_comic_not_present_locally() {
        let (_, service) = setup().await;

        let peer_manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Nao Existe Aqui".into(),
                chapter: "1".into(),
                last_page: 1,
                is_completed: false,
                updated_at: 100,
            }],
            read_markers: vec![],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied.entries_applied, 0);
    }

    #[tokio::test]
    async fn apply_manifest_merges_read_markers() {
        let (pool, service) = setup().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let peer_manifest = HistoryManifest {
            entries: vec![],
            read_markers: vec![
                ReadMarker { comic_name: "Test".into(), chapter: "1".into(), created_at: 1 },
                ReadMarker { comic_name: "Test".into(), chapter: "2".into(), created_at: 2 },
            ],
        };

        let applied = service.apply_manifest(&peer_manifest).await.unwrap();
        assert_eq!(applied.markers_applied, 2);

        let manifest = service.build_manifest().await.unwrap();
        assert_eq!(manifest.read_markers.len(), 2);
    }

    #[tokio::test]
    async fn resolve_chapter_sorts_ignores_unknown_id_and_unknown_comic() {
        let (_, service) = setup().await;

        assert_eq!(
            service.resolve_chapter_sorts("Test", &[1, 999]).await.unwrap(),
            vec!["1".to_string()]
        );
        assert!(service.resolve_chapter_sorts("Nao Existe", &[1]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn comic_exists_reflects_local_presence() {
        let (_, service) = setup().await;

        assert!(service.comic_exists("Test").await.unwrap());
        assert!(!service.comic_exists("Nao Existe Aqui").await.unwrap());
    }
}
