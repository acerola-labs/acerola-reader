use sqlx::SqlitePool;

use crate::{
    data::{
        models::archive::archive_template::{ArchiveTemplate, SortType},
        repositories::archive::archive_template_repo::ArchiveTemplateRepository,
    },
    infra::{
        error::ComicError,
        pattern::template_validator::{validate_chapter_template, validate_volume_template},
    },
};

/// Service responsible for CRUD of user-defined naming templates.
/// All pattern validation and parsing stays in the `infra::pattern` layer — this
/// service only orchestrates persistence and the business rules around it (label
/// uniqueness and protecting the default templates from removal).
pub struct ArchiveTemplateService {
    repo: ArchiveTemplateRepository,
}

impl ArchiveTemplateService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { repo: ArchiveTemplateRepository::new(pool) }
    }

    /// Lists every template (system defaults + user-created), with the defaults
    /// always on top to make clear they are the unchangeable reference.
    pub async fn list_templates(&self) -> Result<Vec<ArchiveTemplate>, ComicError> {
        let mut templates = self.repo.base.find_all().await?;
        templates
            .sort_by(|a, b| b.is_default.cmp(&a.is_default).then_with(|| a.label.cmp(&b.label)));
        Ok(templates)
    }

    /// Creates a new user template, validating the pattern with the same rules used
    /// by the scanner (`validate_chapter_template`/`validate_volume_template`) before
    /// persisting, so a broken pattern never gets into the database.
    pub async fn create_template(
        &self, label: String, pattern: String, sort_type: SortType,
    ) -> Result<ArchiveTemplate, ComicError> {
        let label = label.trim().to_string();
        let pattern = pattern.trim().to_string();

        if label.is_empty() {
            return Err(ComicError::InvalidRequest("Template name cannot be empty.".into()));
        }

        match sort_type {
            SortType::Chapter => validate_chapter_template(&pattern),
            SortType::Volume => validate_volume_template(&pattern),
        }
        .map_err(|err| ComicError::InvalidRequest(err.to_string()))?;

        let id = self.repo.base.get_next_id().await?;
        let template =
            ArchiveTemplate { id, label, pattern, sort_type, is_default: false, priority: 0 };

        Ok(self.repo.base.insert(&template).await?)
    }

    /// Removes a user-created template. Default templates (`is_default`) can never be
    /// removed here — this guarantee is what keeps the parser always with a known
    /// fallback, regardless of what the user creates or deletes.
    pub async fn delete_template(&self, id: i64) -> Result<(), ComicError> {
        let templates = self.repo.base.find_all().await?;
        let target =
            templates.iter().find(|template| template.id == id).ok_or(ComicError::NotFound)?;

        if target.is_default {
            return Err(ComicError::InvalidRequest("Default templates cannot be deleted.".into()));
        }

        Ok(self.repo.base.delete(id).await?)
    }
}
