use serde::{Deserialize, Serialize};
use sqlx::{query::Query, sqlite::SqliteArguments, Sqlite};

use crate::data::repositories::{Bindable, Entity};

/// Raw AniList data for a synced comic (`anilist_source` table).
/// `average_score` is the raw AniList score (0-100 scale) — the UI converts it to /10.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AnilistSource {
    pub id: i64,
    pub anilist_id: i64,
    pub average_score: Option<i64>,
    pub popularity: Option<i64>,
    pub trending: Option<i64>,
    pub cover_image: Option<String>,
    pub banner_image: Option<String>,
    pub comic_metadata_fk: i64,
}

impl Entity for AnilistSource {
    fn columns() -> &'static [&'static str] {
        &[
            "id",
            "anilist_id",
            "average_score",
            "popularity",
            "trending",
            "cover_image",
            "banner_image",
            "comic_metadata_fk",
        ]
    }
    fn table_name() -> &'static str {
        "anilist_source"
    }
    fn id(&self) -> i64 {
        self.id
    }
}

impl Bindable for AnilistSource {
    fn bind_insert<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(self.id)
            .bind(self.anilist_id)
            .bind(self.average_score)
            .bind(self.popularity)
            .bind(self.trending)
            .bind(&self.cover_image)
            .bind(&self.banner_image)
            .bind(self.comic_metadata_fk)
    }

    fn bind_update<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(self.anilist_id)
            .bind(self.average_score)
            .bind(self.popularity)
            .bind(self.trending)
            .bind(&self.cover_image)
            .bind(&self.banner_image)
            .bind(self.comic_metadata_fk)
            .bind(self.id) // WHERE id = ?
    }
}
