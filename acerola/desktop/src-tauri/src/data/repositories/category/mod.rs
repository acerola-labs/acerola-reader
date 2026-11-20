use sqlx::SqlitePool;

use crate::{
    data::{
        models::category::{category::Category, comic_category::ComicCategory},
        repositories::Repository,
    },
    infra::error::DbError,
};

/// Repository that manages categories (bookmarks) and their relation to comics.
pub struct CategoryRepository {
    pub comic_category_base: Repository<ComicCategory>,
    pub base: Repository<Category>,
    pool: SqlitePool,
}

impl CategoryRepository {
    /// Creates the repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            base: Repository::new(pool.clone()),
            comic_category_base: Repository::new(pool.clone()),
            pool,
        }
    }

    /// Removes a comic's association with its current category (if any).
    pub async fn remove_category_from_comic(&self, comic_id: i64) -> Result<(), DbError> {
        sqlx::query("DELETE FROM comic_category WHERE comic_directory_fk = ?")
            .bind(comic_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Fetches the category associated with a comic, if any.
    pub async fn get_comic_category(&self, comic_id: i64) -> Result<Option<Category>, DbError> {
        let result = sqlx::query_as::<_, Category>(
            "SELECT c.id, c.name, c.color FROM category c
             INNER JOIN comic_category cc ON c.id = cc.category_id
             WHERE cc.comic_directory_fk = ?",
        )
        .bind(comic_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Fetches every category associated with each comic as a HashMap (comic_directory_fk -> Category).
    pub async fn get_all_comic_bookmarks(
        &self,
    ) -> Result<std::collections::HashMap<i64, Category>, DbError> {
        let rows = sqlx::query_as::<_, (i64, Option<i64>, String, i64)>(
            "SELECT cc.comic_directory_fk, c.id, c.name, c.color FROM category c
             INNER JOIN comic_category cc ON c.id = cc.category_id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map = std::collections::HashMap::new();
        for (comic_directory_fk, id, name, color) in rows {
            map.insert(comic_directory_fk, Category { id, name, color });
        }
        Ok(map)
    }
}
