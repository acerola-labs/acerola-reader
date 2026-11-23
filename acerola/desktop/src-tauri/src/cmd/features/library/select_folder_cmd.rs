use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FsExt;

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<String, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder);
    });

    let path = match rx.await.map_err(|err| err.to_string())? {
        Some(path) => PathBuf::from(path.to_string()),
        None => {
            return Err("No folder selected".to_string());
        },
    };

    tracing::info!("User selected folder: {:?}", path);

    match app.fs_scope().allow_directory(&path, true) {
        Ok(_) => tracing::info!("fs_scope liberado para: {:?}", path),
        Err(err) => tracing::error!("failed to allow fs_scope: {}", err),
    }

    match app.asset_protocol_scope().allow_directory(&path, true) {
        Ok(_) => tracing::info!("asset_protocol_scope liberado para: {:?}", path),
        Err(err) => tracing::error!("failed to allow asset_protocol_scope: {}", err),
    }

    Ok(path.to_string_lossy().to_string())
}
