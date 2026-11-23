use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::SqlitePool;

use crate::{
    core::services::{archive::path_guard::path_hash, metadata::find_comic_info_path},
    data::{
        models::archive::{
            chapter_archive::{is_special_name, ChapterArchive},
            comic_directory::ComicDirectory,
        },
        repositories::archive::{
            chapter_archive_repo::ChapterRepository, comic_directory_repo::ComicRepository,
        },
    },
    infra::{
        error::{ComicError, DbError},
        sync::messages::{
            ComicSummaryEntry, FileChapterInfo, FileComicInfo, FileExtraInfo, FileManifest,
            EXTRA_KIND_BANNER, EXTRA_KIND_COMIC_INFO, EXTRA_KIND_COVER,
        },
    },
};

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

/// Replaces characters invalid in Windows/Unix folder names with `_`.
fn sanitize_folder_name(name: &str) -> String {
    name.chars()
        .map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

/// Restricts a manifest (already scoped to ONE comic, via `build_manifest_for_comic`) to the
/// requested chapter labels — an empty list means "no filter" (whole comic),
/// preserving the full sync behavior (`sync_comic`/"Sync with device").
/// Extra items (cover/banner/`ComicInfo.xml`) are never filtered here: they are metadata of
/// the comic as a whole, not of a specific chapter, so they keep going along with
/// any scoped session (useful to display the newly created comic on the destination).
pub fn restrict_manifest_to_chapters(
    mut manifest: FileManifest, chapters: &[String],
) -> FileManifest {
    if chapters.is_empty() {
        return manifest;
    }
    for comic in &mut manifest.comics {
        comic.chapters.retain(|chapter| chapters.iter().any(|wanted| wanted == &chapter.chapter));
    }
    manifest
}

/// Builds and applies file manifests (the actual `.cbz`/`.cbr`) between two devices.
///
/// Unlike history sync, this one CAN create new comics locally — it is the
/// mechanism that actually "carries the library" from one device to the other. A new comic
/// (no local entry yet) is written directly to `<library_root>/<sanitized name>/` —
/// the same place the regular library scanner would use if the user had added it
/// manually, with no intermediate `synced/` folder.
#[derive(Clone)]
pub struct FileSyncService {
    comic_repo: ComicRepository,
    chapter_repo: ChapterRepository,
    /// Resolved on demand (never stored as a fixed `PathBuf`) because `FileSyncService`
    /// is built only once, in a background task, at app boot
    /// (`bios::network::setup_network`) — if we stored an already-resolved `PathBuf` here,
    /// any `library_path` change made by the user after boot (in
    /// `settings.json`) would be ignored until the next restart, and sync would keep
    /// writing to the old destination (that was exactly the bug: files going to the fallback
    /// `<app_data>/library` even with the right path already configured).
    library_root: Arc<dyn Fn() -> PathBuf + Send + Sync>,
}

impl FileSyncService {
    pub fn new(
        pool: SqlitePool, library_root: impl Fn() -> PathBuf + Send + Sync + 'static,
    ) -> Self {
        Self {
            comic_repo: ComicRepository::new(pool.clone()),
            chapter_repo: ChapterRepository::new(pool),
            library_root: Arc::new(library_root),
        }
    }

    /// Builds the full local manifest, with file size read from disk on demand
    /// (only runs when a file sync session starts, not on every scan).
    pub async fn build_manifest(&self) -> Result<FileManifest, ComicError> {
        let rows = self.chapter_repo.find_all_with_comic_name().await?;
        let mut by_comic: HashMap<String, Vec<FileChapterInfo>> = HashMap::new();

        for row in rows {
            let size = tokio::fs::metadata(&row.path).await.map(|meta| meta.len()).unwrap_or(0);
            let file_name = Path::new(&row.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&row.chapter)
                .to_string();

            by_comic.entry(row.comic_name).or_default().push(FileChapterInfo {
                chapter: row.chapter,
                file_name,
                checksum: row.checksum,
                size,
            });
        }

        // Cover/banner/ComicInfo.xml live in `comic_directory`, not in `chapter_archive` —
        // the whole comic row is needed (not just the name) to build all three, so it
        // iterates `comic_directory` instead of only the names already collected in `by_comic` above
        // (it also covers a comic with a cover but no indexed chapter yet).
        let all_comics = self.comic_repo.base.find_all().await?;
        let mut comics = Vec::with_capacity(all_comics.len());

        for comic in &all_comics {
            let chapters = by_comic.remove(&comic.name).unwrap_or_default();
            let (cover, banner, comic_info) = self.build_comic_extras(comic).await;

            if chapters.is_empty() && cover.is_none() && banner.is_none() && comic_info.is_none() {
                continue;
            }

            comics.push(FileComicInfo {
                comic_name: comic.name.clone(),
                chapters,
                cover,
                banner,
                comic_info,
            });
        }

        // Defensive: a chapter whose `comic_name` matched no `comic_directory` row
        // should not exist (the FK guarantees consistency), but do not silently drop it
        // if it happens.
        comics.extend(by_comic.into_iter().map(|(comic_name, chapters)| FileComicInfo {
            comic_name,
            chapters,
            ..Default::default()
        }));

        Ok(FileManifest { comics })
    }

    /// Reads and hashes (if present) a comic's local cover, banner and `ComicInfo.xml` —
    /// small files (thumbnail/xml), unlike chapters, so it is fine to recompute the
    /// checksum on every `build_manifest()` instead of relying on a cached column.
    async fn build_comic_extras(
        &self, comic: &ComicDirectory,
    ) -> (Option<FileExtraInfo>, Option<FileExtraInfo>, Option<FileExtraInfo>) {
        let cover = Self::hash_extra_file(comic.cover.as_deref().map(Path::new)).await;
        let banner = Self::hash_extra_file(comic.banner.as_deref().map(Path::new)).await;
        let comic_info_path = find_comic_info_path(Path::new(&comic.path));
        let comic_info = Self::hash_extra_file(comic_info_path.as_deref()).await;

        (cover, banner, comic_info)
    }

    async fn hash_extra_file(path: Option<&Path>) -> Option<FileExtraInfo> {
        let path = path?;
        let bytes = tokio::fs::read(path).await.ok()?;
        let file_name = path.file_name()?.to_str()?.to_string();

        Some(FileExtraInfo { file_name, checksum: sha256_hex(&bytes), size: bytes.len() as u64 })
    }

    /// Library summary (title + chapter count + cover version) via
    /// `ChapterRepository::get_library_summary` — pure SQL, no disk access. Only used by
    /// `acerola/browse-library/1` (`LibraryBrowseInbound`); `build_manifest()` is still
    /// what file transfer (`sync-files`/`sync-comic`) uses, since that phase
    /// really needs each chapter's bytes/checksum.
    pub async fn get_library_summary(&self) -> Result<Vec<ComicSummaryEntry>, ComicError> {
        let rows = self.chapter_repo.get_library_summary().await?;
        Ok(rows
            .into_iter()
            .map(|row| ComicSummaryEntry {
                comic_name: row.comic_name,
                chapter_count: row.chapter_count as u32,
                cover_version: row.cover_version,
            })
            .collect())
    }

    /// Same manifest as `build_manifest()`, but filtered to a single comic — used by the
    /// individual sync (`acerola/sync-comic/1`) to avoid exchanging the whole library when only
    /// one `comic_name` was requested. Reuses the full query instead of a dedicated lookup
    /// in the repository: personal libraries are small enough for this not to matter, and
    /// it avoids duplicating the `FileChapterInfo` building logic (size on disk, file
    /// name) in two places.
    pub async fn build_manifest_for_comic(
        &self, comic_name: &str,
    ) -> Result<FileManifest, ComicError> {
        let mut manifest = self.build_manifest().await?;
        manifest.comics.retain(|comic| comic.comic_name == comic_name);
        Ok(manifest)
    }

    /// Resolves local `chapter_archive_id`s (the unit the multi-selection UI already uses,
    /// e.g. `markChaptersReadBatch`/`HistoryEntryScope`) to chapter LABELS
    /// (`FileChapterInfo.chapter`) — unlike `HistorySyncService::resolve_chapter_sorts`,
    /// which resolves to `chapter_sort`: file sync already uses the label as the
    /// cross-device key (see the contract documented in `persist_received_chapter`), so that is
    /// the key that must travel in `ComicSyncRequest.chapters`. IDs that do not exist or that
    /// belong to another comic are silently ignored, same stance as
    /// `resolve_chapter_sorts`.
    pub async fn resolve_chapter_labels(
        &self, comic_name: &str, chapter_ids: &[i64],
    ) -> Result<Vec<String>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(Vec::new());
        };

        let mut labels = Vec::with_capacity(chapter_ids.len());
        for &chapter_id in chapter_ids {
            let Some(chapter) = self.chapter_repo.find_by_id(chapter_id).await? else {
                continue;
            };
            if chapter.comic_directory_fk != comic.id {
                continue;
            }
            labels.push(chapter.chapter);
        }
        Ok(labels)
    }

    /// Computes what I want, comparing the peer's manifest against the local database: chapters
    /// that are missing or have a different checksum (file outdated/corrupted locally), plus
    /// missing or outdated extra items (cover/banner/ComicInfo.xml). Returns
    /// `(wanted, wanted_extras, conflicts)` — chapters and extras are independent concepts,
    /// so separate vectors instead of mixing everything in a single list with a discriminator.
    /// `conflicts` is a subset of `wanted`: chapters that already EXIST locally but with a
    /// checksum different from the peer's (both sides really diverged, it is not just "I don't have
    /// it yet") — they are still pulled normally (without changing the current overwrite
    /// behavior), they are just counted separately so the session can report that a
    /// real conflict happened, instead of treating it as just another silent update.
    pub async fn diff_wanted(
        &self, peer_manifest: &FileManifest,
    ) -> Result<(Vec<(String, String)>, Vec<(String, String)>, Vec<(String, String)>), ComicError>
    {
        let mut wanted = Vec::new();
        let mut wanted_extras = Vec::new();
        let mut conflicts = Vec::new();

        for comic in &peer_manifest.comics {
            let local_comic = self.comic_repo.find_by_name(&comic.comic_name).await?;

            for chapter in &comic.chapters {
                let local_chapter = match &local_comic {
                    Some(existing) => {
                        self.chapter_repo
                            .find_by_comic_and_chapter(existing.id, &chapter.chapter)
                            .await?
                    },
                    None => None,
                };

                let already_have = local_chapter.as_ref().is_some_and(|local| {
                    local.checksum.is_some() && local.checksum == chapter.checksum
                });

                if already_have {
                    continue;
                }

                // A local chapter with an already-recorded checksum that differs from what the peer
                // has is a real conflict — unlike "I never had this chapter".
                if local_chapter.as_ref().is_some_and(|local| local.checksum.is_some()) {
                    conflicts.push((comic.comic_name.clone(), chapter.chapter.clone()));
                }

                wanted.push((comic.comic_name.clone(), chapter.chapter.clone()));
            }

            for (kind, peer_extra) in [
                (EXTRA_KIND_COVER, &comic.cover),
                (EXTRA_KIND_BANNER, &comic.banner),
                (EXTRA_KIND_COMIC_INFO, &comic.comic_info),
            ] {
                let Some(peer_extra) = peer_extra else { continue };

                let local_checksum = match &local_comic {
                    Some(existing) => {
                        self.local_extra_info(existing, kind).await.map(|info| info.checksum)
                    },
                    None => None,
                };

                if local_checksum.as_deref() != Some(peer_extra.checksum.as_str()) {
                    wanted_extras.push((comic.comic_name.clone(), kind.to_string()));
                }
            }
        }

        Ok((wanted, wanted_extras, conflicts))
    }

    async fn local_extra_info(&self, comic: &ComicDirectory, kind: &str) -> Option<FileExtraInfo> {
        match kind {
            EXTRA_KIND_COVER => Self::hash_extra_file(comic.cover.as_deref().map(Path::new)).await,
            EXTRA_KIND_BANNER => {
                Self::hash_extra_file(comic.banner.as_deref().map(Path::new)).await
            },
            EXTRA_KIND_COMIC_INFO => {
                let path = find_comic_info_path(Path::new(&comic.path));
                Self::hash_extra_file(path.as_deref()).await
            },
            _ => None,
        }
    }

    /// Resolves the local path + metadata of an existing chapter, to send to the peer.
    pub async fn resolve_local_file(
        &self, comic_name: &str, chapter: &str,
    ) -> Result<Option<(PathBuf, u64, Option<String>, String)>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };
        let Some(chapter_row) =
            self.chapter_repo.find_by_comic_and_chapter(comic.id, chapter).await?
        else {
            return Ok(None);
        };

        let path = PathBuf::from(&chapter_row.path);
        let size = tokio::fs::metadata(&path).await.map(|meta| meta.len()).unwrap_or(0);
        let file_name =
            path.file_name().and_then(|name| name.to_str()).unwrap_or(chapter).to_string();

        Ok(Some((path, size, chapter_row.checksum, file_name)))
    }

    /// Resolves path + metadata of an existing local extra item (cover/banner/ComicInfo.xml),
    /// to send to the peer — same return shape as `resolve_local_file`. The
    /// checksum does not come from a cached column (there is none for extras): it is recomputed
    /// here, the same way as in `build_comic_extras`/`local_extra_info`.
    pub async fn resolve_local_extra(
        &self, comic_name: &str, kind: &str,
    ) -> Result<Option<(PathBuf, u64, Option<String>, String)>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };

        let path = match kind {
            EXTRA_KIND_COVER => comic.cover.map(PathBuf::from),
            EXTRA_KIND_BANNER => comic.banner.map(PathBuf::from),
            EXTRA_KIND_COMIC_INFO => find_comic_info_path(Path::new(&comic.path)),
            _ => None,
        };
        let Some(path) = path else {
            return Ok(None);
        };

        let Ok(bytes) = tokio::fs::read(&path).await else {
            return Ok(None);
        };
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or(kind).to_string();

        Ok(Some((path, bytes.len() as u64, Some(sha256_hex(&bytes)), file_name)))
    }

    /// Local cover (version + bytes) of `comic_name`, used by `acerola/browse-cover/1`. `None`
    /// covers both "comic does not exist" and "exists but has no saved cover" — both cases
    /// result in the same answer to the peer, there is no need to tell them apart.
    pub async fn get_local_cover(
        &self, comic_name: &str,
    ) -> Result<Option<(i64, Vec<u8>)>, ComicError> {
        let Some(comic) = self.comic_repo.find_by_name(comic_name).await? else {
            return Ok(None);
        };
        let Some(cover_path) = comic.cover else {
            return Ok(None);
        };

        let bytes = match tokio::fs::read(&cover_path).await {
            Ok(bytes) => bytes,
            Err(_) => return Ok(None),
        };

        Ok(Some((comic.last_modified, bytes)))
    }

    /// Resolves a comic's `ComicDirectory` by name, creating it (via
    /// `create_synced_comic`) if it does not exist locally yet, and ensures the folder exists
    /// on disk. Public so whoever receives bytes (`transfer::receive_files`) can write the
    /// temporary file ALREADY inside that folder before knowing the download finished —
    /// avoids relying on a separate staging folder (`rename` would have to be guaranteed on the
    /// same volume anyway) and leaves no extra folder visible in the library.
    pub async fn resolve_comic_dir(&self, comic_name: &str) -> Result<ComicDirectory, ComicError> {
        let comic = match self.comic_repo.find_by_name(comic_name).await? {
            Some(existing) => existing,
            None => self.create_synced_comic(comic_name).await?,
        };

        let comic_dir = PathBuf::from(&comic.path);
        tokio::fs::create_dir_all(&comic_dir).await.map_err(ComicError::Io)?;

        Ok(comic)
    }

    /// Moves the already received (and verified) file from `temp_path` to its final destination
    /// inside the library, creating the comic in the database if it does not exist yet, and indexes
    /// the chapter reusing the scanner's same insert/update pattern.
    ///
    /// **Chapter label contract** (the same one any peer, including Android, must follow
    /// for the received `chapter` to match what this desktop already produces in the local
    /// scan via `Path::file_stem` — see `chapter_scanner_engine.rs::scan_chapter`):
    /// the displayed label is the file name **without the last extension** — everything before
    /// the last `.`, except when that `.` is the first character of the name (a dotfile with no
    /// real extension, e.g. `.gitignore`, which stays as is). `"Ch. 1.cbz"` → `"Ch. 1"`;
    /// `"archive.tar.gz"` → `"archive.tar"` (only the LAST extension is removed).
    ///
    /// That is why we ignore the `chapter` that came in the `FileHeader` to build the label — it is
    /// only the natural key the peer used to decide WHICH chapter to send (via
    /// `resolve_local_file`/`find_by_comic_and_chapter`), it does not necessarily come in that
    /// format. A peer with this contract broken (that was the case: Android sends the raw
    /// `DocumentsContract` `DISPLAY_NAME`, with the extension, without going through a `file_stem`
    /// equivalent) should not pollute the local library — we recompute it here from the
    /// file's real `file_name`, which is always reliable (it is literally the name of the file
    /// we just wrote to disk).
    pub async fn persist_received_chapter(
        &self, comic_name: &str, chapter: &str, file_name: &str, temp_path: &Path, checksum: String,
    ) -> Result<(), ComicError> {
        let comic = self.resolve_comic_dir(comic_name).await?;
        let comic_dir = PathBuf::from(&comic.path);
        let dest_path = comic_dir.join(file_name);
        tokio::fs::rename(temp_path, &dest_path).await.map_err(ComicError::Io)?;

        let display_chapter =
            Path::new(file_name).file_stem().and_then(|stem| stem.to_str()).unwrap_or(chapter);

        // An existing `(comic_directory_fk, chapter)` covers a resend (sync interrupted and
        // resumed, or the same chapter requested twice) — in that case it must be an UPDATE
        // on the existing row, not a new INSERT: its `id` is the `path_hash` of the OLD path,
        // which only matches the `path_hash(&dest_path)` computed below if the file name did not
        // change between both attempts. Without this, the INSERT hits the UNIQUE constraint, and
        // since the file was already renamed on disk (line above), the database row silently
        // becomes stale (old path/checksum) even with the new content already
        // written — that was exactly the bug: `Err(DbError::UniqueViolation) => Ok(())`
        // discarded the conflict instead of propagating the new data to the existing row.
        let existing =
            self.chapter_repo.find_by_comic_and_chapter(comic.id, display_chapter).await?;

        // The old path is only removed after the new record is in place — never before,
        // so an error midway leaves neither a file nor a database row behind.
        if let Some(existing) = &existing {
            if existing.path != dest_path.to_string_lossy() {
                tokio::fs::remove_file(&existing.path).await.ok();
            }
        }

        let chapter_row = ChapterArchive {
            id: existing.as_ref().map(|c| c.id).unwrap_or_else(|| path_hash(&dest_path)),
            chapter: display_chapter.to_string(),
            path: dest_path.to_string_lossy().to_string(),
            chapter_sort: ChapterArchive::fallback_sort(display_chapter, 0),
            is_special: is_special_name(display_chapter),
            checksum: Some(checksum),
            comic_directory_fk: comic.id,
            volume_fk: existing.as_ref().and_then(|c| c.volume_fk),
            last_modified: now_secs(),
        };

        if existing.is_some() {
            self.chapter_repo.base.update(&chapter_row).await.map(|_| ()).map_err(ComicError::from)
        } else {
            match self.chapter_repo.base.insert(&chapter_row).await {
                // A genuine race (two concurrent inserts for the same pair at the same
                // instant) can still collide here even after `find_by_comic_and_chapter`
                // found nothing — window between the SELECT and the INSERT without a transaction. It is
                // still an acceptable no-op in this very rare case: the other session already wrote
                // equivalent data (same chapter, same transfer).
                Ok(_) | Err(DbError::UniqueViolation) => Ok(()),
                Err(error) => Err(ComicError::from(error)),
            }
        }
    }

    /// Persists a received extra item (cover/banner/ComicInfo.xml) — counterpart of
    /// `persist_received_chapter` for items that are not chapters. Returns the already updated
    /// `ComicDirectory` (with the new cover/banner path, when applicable), so the caller in
    /// `transfer.rs` knows the `id` without a second query and decides whether to trigger
    /// metadata reprocessing of `ComicInfo.xml`.
    pub async fn persist_received_extra(
        &self, comic_name: &str, kind: &str, file_name: &str, temp_path: &Path,
    ) -> Result<ComicDirectory, ComicError> {
        let comic = self.resolve_comic_dir(comic_name).await?;
        let comic_dir = PathBuf::from(&comic.path);

        match kind {
            EXTRA_KIND_COVER => {
                self.persist_artwork_extra(&comic, &comic_dir, "cover", file_name, temp_path).await
            },
            EXTRA_KIND_BANNER => {
                self.persist_artwork_extra(&comic, &comic_dir, "banner", file_name, temp_path).await
            },
            EXTRA_KIND_COMIC_INFO => {
                let dest_path = comic_dir.join("ComicInfo.xml");

                // A pre-existing `ComicInfo.xml` with different casing (the guard that
                // validates this name is case-insensitive, but the filesystem may have more than one
                // entry on disk on case-sensitive systems) must go away before writing the
                // canonical name — otherwise both coexist and the next `find_comic_info_path`
                // becomes ambiguous about which one is the "real" one.
                if let Some(existing_path) = find_comic_info_path(&comic_dir) {
                    if existing_path != dest_path {
                        let _ = tokio::fs::remove_file(&existing_path).await;
                    }
                }

                tokio::fs::rename(temp_path, &dest_path).await.map_err(ComicError::Io)?;
                Ok(comic)
            },
            _ => Err(ComicError::SystemFailure(format!("unknown extra kind: {kind}"))),
        }
    }

    /// Renames `temp_path` to `<stem>.<extension>` at the root of the comic directory, cleans
    /// up any orphan `<stem>.<other extension>` (same cleanup
    /// `cover_extractor::persist_cover` already does for the cover, replicated here instead of
    /// generalizing that public function — avoids touching a contract already used by the manual
    /// metadata flow), and updates the matching column (`cover`/`banner`) + `last_modified`.
    async fn persist_artwork_extra(
        &self, comic: &ComicDirectory, comic_dir: &Path, stem: &str, file_name: &str,
        temp_path: &Path,
    ) -> Result<ComicDirectory, ComicError> {
        let extension = Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg")
            .to_ascii_lowercase();
        let dest_path = comic_dir.join(format!("{stem}.{extension}"));

        for stale_extension in ["jpg", "jpeg", "png"] {
            if stale_extension == extension {
                continue;
            }
            let _ =
                tokio::fs::remove_file(comic_dir.join(format!("{stem}.{stale_extension}"))).await;
        }

        tokio::fs::rename(temp_path, &dest_path).await.map_err(ComicError::Io)?;

        let mut updated = comic.clone();
        let dest_str = Some(dest_path.to_string_lossy().to_string());
        match stem {
            "cover" => updated.cover = dest_str,
            "banner" => updated.banner = dest_str,
            _ => {},
        }
        updated.last_modified = now_secs();

        self.comic_repo.base.update(&updated).await.map_err(ComicError::from)
    }

    /// Creates a new `comic_directory` under `<library_root>/<name>` — used when a
    /// received chapter belongs to a comic that does not exist locally yet. Same
    /// path the regular library scanner would use for a manually added comic
    /// — with no intermediate `synced/` folder.
    async fn create_synced_comic(&self, comic_name: &str) -> Result<ComicDirectory, ComicError> {
        let comic_dir = self.library_root().join(sanitize_folder_name(comic_name));

        let comic = ComicDirectory {
            id: path_hash(&comic_dir),
            name: comic_name.to_string(),
            path: comic_dir.to_string_lossy().to_string(),
            cover: None,
            banner: None,
            last_modified: now_secs(),
            archive_template_fk: None,
            external_sync_enabled: true,
            hidden: false,
        };

        match self.comic_repo.base.insert(&comic).await {
            Ok(saved) => Ok(saved),
            Err(DbError::UniqueViolation) => {
                self.comic_repo.find_by_name(comic_name).await?.ok_or(ComicError::NotFound)
            },
            Err(error) => Err(ComicError::from(error)),
        }
    }

    /// Resolved on every call — never cached — to always reflect the current `library_path`
    /// in `settings.json`, even if the user changed it after boot.
    pub fn library_root(&self) -> PathBuf {
        (self.library_root)()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::FileSyncService;
    use crate::infra::sync::messages::{FileChapterInfo, FileComicInfo, FileManifest};

    async fn setup() -> (sqlx::SqlitePool, tempfile::TempDir, FileSyncService) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db_with_comic().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', 'p/Cap 1.cbz', '1', 0, 'abc', 1, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool.clone(), move || root.clone());
        (pool, temp_dir, service)
    }

    #[tokio::test]
    async fn build_manifest_for_comic_scopes_to_a_single_comic() {
        let (_, _dir, service) = setup().await;

        let manifest = service.build_manifest_for_comic("Test").await.unwrap();
        assert_eq!(manifest.comics.len(), 1);
        assert_eq!(manifest.comics[0].comic_name, "Test");

        let empty = service.build_manifest_for_comic("Nao Existe").await.unwrap();
        assert!(empty.comics.is_empty());
    }

    #[tokio::test]
    async fn diff_wanted_ignores_chapter_with_same_checksum() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Test".into(),
                chapters: vec![FileChapterInfo {
                    chapter: "Cap 1".into(),
                    file_name: "Cap 1.cbz".into(),
                    checksum: Some("abc".into()),
                    size: 10,
                }],
                ..Default::default()
            }],
        };

        let (wanted, wanted_extras, conflicts) = service.diff_wanted(&peer_manifest).await.unwrap();
        assert!(wanted.is_empty());
        assert!(wanted_extras.is_empty());
        assert!(conflicts.is_empty());
    }

    #[tokio::test]
    async fn diff_wanted_wants_chapter_with_different_checksum() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Test".into(),
                chapters: vec![FileChapterInfo {
                    chapter: "Cap 1".into(),
                    file_name: "Cap 1.cbz".into(),
                    checksum: Some("different".into()),
                    size: 10,
                }],
                ..Default::default()
            }],
        };

        let (wanted, _, conflicts) = service.diff_wanted(&peer_manifest).await.unwrap();
        assert_eq!(wanted, vec![("Test".to_string(), "Cap 1".to_string())]);
        assert_eq!(
            conflicts,
            vec![("Test".to_string(), "Cap 1".to_string())],
            "a local checksum different from the peer's is a real conflict, not just an update"
        );
    }

    #[tokio::test]
    async fn diff_wanted_wants_comic_not_present_locally() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Novo Quadrinho".into(),
                chapters: vec![FileChapterInfo {
                    chapter: "Cap 1".into(),
                    file_name: "Cap 1.cbz".into(),
                    checksum: Some("x".into()),
                    size: 10,
                }],
                ..Default::default()
            }],
        };

        let (wanted, _, conflicts) = service.diff_wanted(&peer_manifest).await.unwrap();
        assert_eq!(wanted, vec![("Novo Quadrinho".to_string(), "Cap 1".to_string())]);
        assert!(conflicts.is_empty(), "a comic never seen locally is not a conflict");
    }

    #[tokio::test]
    async fn diff_wanted_wants_extra_missing_locally() {
        let (_, _dir, service) = setup().await;

        let peer_manifest = FileManifest {
            comics: vec![FileComicInfo {
                comic_name: "Test".into(),
                chapters: vec![],
                cover: Some(crate::infra::sync::messages::FileExtraInfo {
                    file_name: "cover.jpg".into(),
                    checksum: "cover-hash".into(),
                    size: 100,
                }),
                ..Default::default()
            }],
        };

        let (_, wanted_extras, _) = service.diff_wanted(&peer_manifest).await.unwrap();
        assert_eq!(wanted_extras, vec![("Test".to_string(), "cover".to_string())]);
    }

    #[tokio::test]
    async fn persist_received_chapter_creates_new_comic_and_indexes_chapter() {
        let (pool, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming.tmp");
        tokio::fs::write(&temp_path, b"fake cbz bytes").await.unwrap();

        service
            .persist_received_chapter(
                "Quadrinho Recebido",
                "Cap 1",
                "Cap 1.cbz",
                &temp_path,
                "checksum123".to_string(),
            )
            .await
            .unwrap();

        let comic: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM comic_directory WHERE name = 'Quadrinho Recebido'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(comic.0, 1);

        let chapter: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM chapter_archive WHERE chapter = 'Cap 1' AND checksum = 'checksum123'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(chapter.0, 1);

        assert!(!temp_path.exists(), "temp file should have been moved, not copied");
    }

    /// Locks the chapter label contract: even if the peer sends a dirty `chapter` (with the
    /// extension, exactly the bug reproduced with Android — raw `DocumentsContract`
    /// `DISPLAY_NAME`), the desktop never stores the extension. The persisted label is always
    /// recomputed from the file's real `file_name`, via `file_stem`.
    #[tokio::test]
    async fn persist_received_chapter_ignores_extension_in_dirty_chapter_from_peer() {
        let (pool, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming.tmp");
        tokio::fs::write(&temp_path, b"fake cbz bytes").await.unwrap();

        service
            .persist_received_chapter(
                "Mato Seihei no Slave",
                "Ch. 1.cbz", // <- dirty label, exactly as it came in the real Android log
                "Ch. 1.cbz",
                &temp_path,
                "checksum456".to_string(),
            )
            .await
            .unwrap();

        let chapter: (String, String) = sqlx::query_as(
            "SELECT chapter, chapter_sort FROM chapter_archive WHERE comic_directory_fk = (SELECT id FROM comic_directory WHERE name = 'Mato Seihei no Slave')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(chapter.0, "Ch. 1", "chapter must not carry the file extension");
        assert_eq!(chapter.1, "1");
    }

    /// Regression for the bug reported on 2026-08-22: syncing "Omoide Emanon" resending an
    /// already-indexed chapter (retry of a previous interrupted session) collided on
    /// `UNIQUE(comic_directory_fk, chapter)` and the conflict was silently discarded
    /// (`Err(DbError::UniqueViolation) => Ok(())`), leaving the old checksum/path in the database
    /// even with the new file already overwritten on disk. Receiving the same
    /// `(comic_name, chapter)` again must UPDATE the existing row, not discard it.
    #[tokio::test]
    async fn persist_received_chapter_updates_existing_row_on_resend_instead_of_dropping_it() {
        let (pool, dir, service) = setup().await;

        let first_temp = dir.path().join("first.tmp");
        tokio::fs::write(&first_temp, b"conteudo antigo").await.unwrap();
        service
            .persist_received_chapter(
                "Omoide Emanon",
                "Ch. 1",
                "Ch. 1.cbz",
                &first_temp,
                "checksum-antigo".to_string(),
            )
            .await
            .unwrap();

        let second_temp = dir.path().join("second.tmp");
        tokio::fs::write(&second_temp, b"conteudo novo, reenviado").await.unwrap();
        service
            .persist_received_chapter(
                "Omoide Emanon",
                "Ch. 1",
                "Ch. 1.cbz",
                &second_temp,
                "checksum-novo".to_string(),
            )
            .await
            .unwrap();

        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT checksum FROM chapter_archive WHERE comic_directory_fk = (SELECT id FROM comic_directory WHERE name = 'Omoide Emanon') AND chapter = 'Ch. 1'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows.len(), 1, "a resend must not create a second row for the same chapter");
        assert_eq!(rows[0].0, "checksum-novo", "the resend must overwrite the old checksum");
    }

    #[tokio::test]
    async fn persist_received_extra_cover_moves_file_and_updates_comic_directory() {
        let (pool, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming-cover.tmp");
        tokio::fs::write(&temp_path, b"fake jpeg bytes").await.unwrap();

        let comic = service
            .persist_received_extra("Quadrinho Com Capa", "cover", "cover.jpg", &temp_path)
            .await
            .unwrap();

        assert!(comic.cover.as_deref().is_some_and(|path| path.ends_with("cover.jpg")));
        assert!(tokio::fs::metadata(comic.cover.as_deref().unwrap()).await.is_ok());
        assert!(!temp_path.exists(), "temporary file should have been moved, not copied");

        let row: (Option<String>,) =
            sqlx::query_as("SELECT cover FROM comic_directory WHERE name = 'Quadrinho Com Capa'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(row.0.as_deref().is_some_and(|path| path.ends_with("cover.jpg")));
    }

    /// Changing the cover extension (e.g. `cover.png` -> `cover.jpg`) must not leave both
    /// coexisting — same orphan cleanup `cover_extractor::persist_cover` already guarantees
    /// for the manual flow, replicated in `persist_artwork_extra`.
    #[tokio::test]
    async fn persist_received_extra_cover_removes_stale_extension() {
        let (_, dir, service) = setup().await;

        let first_temp = dir.path().join("first-cover.tmp");
        tokio::fs::write(&first_temp, b"png bytes").await.unwrap();
        service
            .persist_received_extra("Quadrinho Capa Trocada", "cover", "cover.png", &first_temp)
            .await
            .unwrap();

        let comic_dir = service.library_root().join("Quadrinho Capa Trocada");
        assert!(tokio::fs::metadata(comic_dir.join("cover.png")).await.is_ok());

        let second_temp = dir.path().join("second-cover.tmp");
        tokio::fs::write(&second_temp, b"jpg bytes").await.unwrap();
        let comic = service
            .persist_received_extra("Quadrinho Capa Trocada", "cover", "cover.jpg", &second_temp)
            .await
            .unwrap();

        assert!(comic.cover.as_deref().is_some_and(|path| path.ends_with("cover.jpg")));
        assert!(tokio::fs::metadata(comic_dir.join("cover.jpg")).await.is_ok());
        assert!(
            tokio::fs::metadata(comic_dir.join("cover.png")).await.is_err(),
            "orphan cover.png should have been removed"
        );
    }

    #[tokio::test]
    async fn persist_received_extra_comic_info_writes_canonical_filename() {
        let (_, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming-comicinfo.tmp");
        tokio::fs::write(&temp_path, b"<ComicInfo><Title>X</Title></ComicInfo>").await.unwrap();

        let comic = service
            .persist_received_extra("Quadrinho Com Info", "comic_info", "ComicInfo.xml", &temp_path)
            .await
            .unwrap();

        let dest = PathBuf::from(&comic.path).join("ComicInfo.xml");
        assert!(tokio::fs::metadata(&dest).await.is_ok());
    }

    #[tokio::test]
    async fn resolve_local_extra_round_trips_persisted_cover() {
        let (_, dir, service) = setup().await;

        let temp_path = dir.path().join("incoming-cover.tmp");
        tokio::fs::write(&temp_path, b"fake jpeg bytes").await.unwrap();
        service
            .persist_received_extra("Quadrinho Resolve", "cover", "cover.jpg", &temp_path)
            .await
            .unwrap();

        let resolved = service.resolve_local_extra("Quadrinho Resolve", "cover").await.unwrap();
        let (path, size, checksum, file_name) = resolved.expect("cover should have been found");
        assert!(path.ends_with("cover.jpg"));
        assert_eq!(size, b"fake jpeg bytes".len() as u64);
        assert!(checksum.is_some());
        assert_eq!(file_name, "cover.jpg");
    }

    #[tokio::test]
    async fn resolve_local_extra_returns_none_for_unknown_comic() {
        let (_, _dir, service) = setup().await;

        let resolved = service.resolve_local_extra("Nao Existe", "cover").await.unwrap();
        assert!(resolved.is_none());
    }
}
