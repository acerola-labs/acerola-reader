use sqlx::SqlitePool;
use tauri::State;

use crate::{
    cmd::events::shared::ErrorPayload,
    core::services::archive::archive_template_service::ArchiveTemplateService,
    data::models::archive::archive_template::{ArchiveTemplate, SortType},
};

/// Tauri command that lists every naming template (default + user-defined).
#[tauri::command]
pub async fn get_archive_templates(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<ArchiveTemplate>, ErrorPayload> {
    let service = ArchiveTemplateService::new(pool.inner().clone());
    service.list_templates().await.map_err(|error| ErrorPayload::from(&error))
}

/// Tauri command that creates a user-defined naming template.
#[tauri::command]
pub async fn create_archive_template(
    label: String, pattern: String, sort_type: SortType, pool: State<'_, SqlitePool>,
) -> Result<ArchiveTemplate, ErrorPayload> {
    let service = ArchiveTemplateService::new(pool.inner().clone());
    service
        .create_template(label, pattern, sort_type)
        .await
        .map_err(|error| ErrorPayload::from(&error))
}

/// Tauri command that removes a user-created naming template.
/// Default templates are rejected by the service and are never removed.
#[tauri::command]
pub async fn delete_archive_template(
    id: i64, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let service = ArchiveTemplateService::new(pool.inner().clone());
    service.delete_template(id).await.map_err(|error| ErrorPayload::from(&error))
}
