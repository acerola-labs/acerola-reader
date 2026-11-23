use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    cmd::events::shared::ErrorPayload,
    core::services::archive::comic_scanner_engine::ComicScannerService,
};

/// Processes every folder found on disk without comparing against the database.
/// Already indexed folders are skipped via INSERT OR IGNORE.
#[tauri::command]
pub async fn refresh_library<R: Runtime>(
    path: String, app: AppHandle<R>, pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let root = PathBuf::from(&path);
    let pool = pool.inner().clone();

    tokio::spawn(async move {
        let service = ComicScannerService::new(root.clone(), pool);

        match service
            .refresh_library(
                root,
                |progress_path| {
                    if let Err(error) = app.emit("scan:progress", progress_path) {
                        tracing::error!("Failed to emit scan:progress: {}", error);
                    }
                },
                |converting_message| {
                    if let Err(error) = app.emit("scan:converting", converting_message) {
                        tracing::error!("Failed to emit scan:converting: {}", error);
                    }
                },
            )
            .await
        {
            Ok(_) => {
                if let Err(error) = app.emit("scan:complete", ()) {
                    tracing::error!("Failed to emit scan:complete: {}", error);
                }
            },
            Err(error) => {
                if let Err(emit_error) = app.emit("scan:error", ErrorPayload::from(&error)) {
                    tracing::error!("Failed to emit scan:error: {}", emit_error);
                }
            },
        }
    });

    Ok(())
}

/// Processes only new or modified folders based on `last_modified`.
/// Removes from the database the folders that no longer exist on disk.
#[tauri::command]
pub async fn incremental_scan<R: Runtime>(
    path: String, app: AppHandle<R>, pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let root = PathBuf::from(&path);
    let pool = pool.inner().clone();

    tokio::spawn(async move {
        let service = ComicScannerService::new(root.clone(), pool);

        match service
            .incremental_scan(
                root,
                |progress_path| {
                    if let Err(error) = app.emit("scan:progress", progress_path) {
                        tracing::error!("Failed to emit scan:progress: {}", error);
                    }
                },
                |converting_message| {
                    if let Err(error) = app.emit("scan:converting", converting_message) {
                        tracing::error!("Failed to emit scan:converting: {}", error);
                    }
                },
            )
            .await
        {
            Ok(_) => {
                if let Err(error) = app.emit("scan:complete", ()) {
                    tracing::error!("Failed to emit scan:complete: {}", error);
                }
            },
            Err(error) => {
                if let Err(emit_error) = app.emit("scan:error", ErrorPayload::from(&error)) {
                    tracing::error!("Failed to emit scan:error: {}", emit_error);
                }
            },
        }
    });

    Ok(())
}

/// Fully refreshes every folder and rescans the chapters of each comic.
/// It is the heaviest operation — use it when the database may be in an inconsistent state.
#[tauri::command]
pub async fn rebuild_library<R: Runtime>(
    path: String, app: AppHandle<R>, pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let root = PathBuf::from(&path);
    let pool = pool.inner().clone();

    tokio::spawn(async move {
        let service = ComicScannerService::new(root.clone(), pool);

        match service
            .rebuild_library(
                root,
                |progress_path| {
                    if let Err(error) = app.emit("scan:progress", progress_path) {
                        tracing::error!("Failed to emit scan:progress: {}", error);
                    }
                },
                |converting_message| {
                    if let Err(error) = app.emit("scan:converting", converting_message) {
                        tracing::error!("Failed to emit scan:converting: {}", error);
                    }
                },
            )
            .await
        {
            Ok(_) => {
                if let Err(error) = app.emit("scan:complete", ()) {
                    tracing::error!("Failed to emit scan:complete: {}", error);
                }
            },
            Err(error) => {
                if let Err(emit_error) = app.emit("scan:error", ErrorPayload::from(&error)) {
                    tracing::error!("Failed to emit scan:error: {}", emit_error);
                }
            },
        }
    });

    Ok(())
}
