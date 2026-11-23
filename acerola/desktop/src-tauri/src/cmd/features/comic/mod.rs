use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    cmd::events::{shared::ErrorPayload, summary::ComicSummaryPayload},
    core::services::{
        comic::ComicService,
        summary::{ChapterCacheService, HomeService},
    },
    data::repositories::views::SortCriteria,
    infra::error::ComicError,
};

/// Tauri command that fetches comics with specific sorting and filters.
#[tauri::command]
pub async fn get_comic_summary_sorted<R: Runtime>(
    sort_by: String, sort_order: String, show_hidden: Option<bool>,
    metadata_source: Option<String>, app: AppHandle<R>, pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let pool = pool.inner().clone();

    // Parse the sort criteria
    let criteria = match (sort_by.as_str(), sort_order.as_str()) {
        ("title", "asc") => SortCriteria::TitleAsc,
        ("title", "desc") => SortCriteria::TitleDesc,
        ("chapterCount", "asc") => SortCriteria::ChapterCountAsc,
        ("chapterCount", "desc") => SortCriteria::ChapterCountDesc,
        ("lastUpdated", "asc") => SortCriteria::LastUpdatedAsc,
        ("lastUpdated", "desc") => SortCriteria::LastUpdatedDesc,
        _ => SortCriteria::TitleAsc,
    };

    let show_hidden = show_hidden.unwrap_or(false);

    tokio::spawn(async move {
        let service = HomeService::new(pool);

        match service.get_all_sorted(criteria, show_hidden, metadata_source).await {
            Ok((comics, counts, meta_map, bookmark_map)) => app
                .emit(
                    "home:data",
                    ComicSummaryPayload::from(comics, counts, meta_map, bookmark_map),
                )
                .unwrap(),
            Err(err) => app.emit("home:error", ErrorPayload::from(&err)).unwrap(),
        }
    });

    Ok(())
}

/// Converts IDs received as strings into `i64` — `comic_directory` IDs are path hashes
/// (`path_hash`) that span almost the whole `i64` range, well above JavaScript's
/// `Number.MAX_SAFE_INTEGER` (2^53). Receiving them directly as `Vec<i64>` forces the
/// frontend to convert a string to `Number` before sending, rounding the value and hitting
/// an `id` that does not exist (that was exactly the bug: "0 comic(s) deleted" and no
/// matching row in the database). Same fix already applied to
/// `clear_comics_metadata_batch`.
fn parse_ids(ids: Vec<String>) -> Result<Vec<i64>, ErrorPayload> {
    ids.into_iter()
        .map(|id| {
            id.parse::<i64>().map_err(|err| {
                ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {err}")))
            })
        })
        .collect()
}

/// Tauri command that updates the visibility of multiple comics.
#[tauri::command]
pub async fn update_comics_visibility(
    ids: Vec<String>, hidden: bool, pool: State<'_, SqlitePool>,
) -> Result<usize, ErrorPayload> {
    let ids = parse_ids(ids)?;
    let service = ComicService::new(pool.inner().clone());
    service
        .update_hidden_status_batch(&ids, hidden)
        .await
        .map_err(|error| ErrorPayload::from(&error))
}

/// Tauri command that deletes multiple comics.
#[tauri::command]
pub async fn delete_comics(
    ids: Vec<String>, pool: State<'_, SqlitePool>,
) -> Result<usize, ErrorPayload> {
    let ids = parse_ids(ids)?;
    let service = ComicService::new(pool.inner().clone());
    service.delete_batch(&ids).await.map_err(|error| ErrorPayload::from(&error))
}

/// Tauri command that updates a comic's external sync status.
#[tauri::command]
pub async fn toggle_comic_external_sync(
    id: String, enabled: bool, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    service
        .update_external_sync_enabled(parsed_id, enabled)
        .await
        .map(|_| ())
        .map_err(|error| ErrorPayload::from(&error))
}

/// Tauri command that rescans a single comic on demand (light sync).
#[tauri::command]
pub async fn rescan_comic(
    id: String, pool: State<'_, SqlitePool>, chapter_cache: State<'_, ChapterCacheService>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    let result = service.rescan_comic(parsed_id).await.map_err(|error| ErrorPayload::from(&error));

    // INFO: A rescan may add/remove/rename chapter files — the
    // get_comic_chapters cache (chapter_cache.rs) must forget this
    // comic so it does not keep serving the old list.
    chapter_cache.invalidate_comic(parsed_id);

    result
}

/// Tauri command that generates a comic's cover from the first chapter's page.
#[tauri::command]
pub async fn regenerate_comic_cover(
    id: String, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    service
        .regenerate_cover(parsed_id)
        .await
        .map(|_| ())
        .map_err(|error| ErrorPayload::from(&error))
}

/// Tauri command that generates the cover of every volume of a comic from the page
/// of each volume's first chapter.
#[tauri::command]
pub async fn regenerate_volume_covers(
    id: String, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    service
        .regenerate_volume_covers(parsed_id)
        .await
        .map(|_| ())
        .map_err(|error| ErrorPayload::from(&error))
}
