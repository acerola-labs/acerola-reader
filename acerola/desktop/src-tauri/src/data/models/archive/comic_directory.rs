use serde::{Deserialize, Serialize};
use sqlx::{query::Query, sqlite::SqliteArguments, Sqlite};

use crate::data::repositories::{Bindable, Entity};

/// Contract with the generic [`crate::data::repositories::Repository`].
impl Entity for ComicDirectory {
    fn columns() -> &'static [&'static str] {
        &[
            "id",
            "name",
            "path",
            "cover",
            "banner",
            "last_modified",
            "archive_template_fk",
            "external_sync_enabled",
            "hidden",
        ]
    }
    fn table_name() -> &'static str {
        "comic_directory"
    }
    fn id(&self) -> i64 {
        self.id
    }
}

/// Ensures the code can deserialize the SQL row into the object
impl Bindable for ComicDirectory {
    fn bind_insert<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(self.id)
            .bind(&self.name)
            .bind(&self.path)
            .bind(&self.cover)
            .bind(&self.banner)
            .bind(self.last_modified)
            .bind(self.archive_template_fk)
            .bind(self.external_sync_enabled)
            .bind(self.hidden)
    }

    fn bind_update<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(&self.name)
            .bind(&self.path)
            .bind(&self.cover)
            .bind(&self.banner)
            .bind(self.last_modified)
            .bind(self.archive_template_fk)
            .bind(self.external_sync_enabled)
            .bind(self.hidden)
            .bind(self.id) // <- id for WHERE id = ?
    }
}

/// Comic directory managed by the application.                                                                                                                                                                                                                                      ///
/// Equivalent to a JPA `@Entity` — maps to the `comic_directory` table.
/// Migration in `src-tauri/migrations/archive`.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ComicDirectory {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub cover: Option<String>,
    pub banner: Option<String>,
    pub last_modified: i64,
    pub archive_template_fk: Option<i64>,
    pub external_sync_enabled: bool,
    pub hidden: bool,
}
