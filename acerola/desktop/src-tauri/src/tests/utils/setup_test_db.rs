pub async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // archive
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/001_create_chapter_template.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/002_create_comic_directory.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/003_create_chapter_archive.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/004_create_volume_archive.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/archive/005_rename_volume_id_fk_to_volume_fk.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // metadata
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/001_create_comic_metadata.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/002_create_chapter_metadata.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/source/001_create_anilist_source.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/source/003_create_mangadex_source.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/relationship/001_create_genre.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/metadata/relationship/004_create_author.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // category
    sqlx::query(include_str!("../../infra/db/migrations/models/category/001_create_category.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/category/002_create_comic_category.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // history
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/history/001_create_reading_history.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/models/history/002_create_chapter_read.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // views
    sqlx::query(include_str!("../../infra/db/migrations/views/001_create_comic_summary_view.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!("../../infra/db/migrations/views/002_drop_comic_summary_view.sql"))
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!(
        "../../infra/db/migrations/views/003_recreate_comic_summary_view_with_fk_names.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    // sync
    sqlx::query(include_str!(
        "../../infra/db/migrations/models/sync/001_create_sync_history_log.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    pool
}

/// Pool with production seeds applied — used when the test depends on the default templates.
pub async fn setup_test_db_with_seeds() -> sqlx::SqlitePool {
    let pool = setup_test_db().await;

    sqlx::query(include_str!("../../infra/db/migrations/seeds/001_seed_chapter_template.sql"))
        .execute(&pool)
        .await
        .unwrap();

    pool
}

/// Resets `last_modified` of every comic — used to force reprocessing in the incremental scan.
pub async fn reset_comics_last_modified(pool: &sqlx::SqlitePool) {
    sqlx::query("UPDATE comic_directory SET last_modified = 0").execute(pool).await.unwrap();
}

/// Pool with a comic_directory already inserted — used by chapter tests that need the FK.
pub async fn setup_test_db_with_comic() -> sqlx::SqlitePool {
    let pool = setup_test_db().await;

    sqlx::query(
        "INSERT INTO comic_directory (id, name, path, last_modified, external_sync_enabled, hidden)
         VALUES (1, 'Test', '/test', 0, 0, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

/// Inserts a minimal comic_directory (no cover or metadata).
pub async fn insert_comic_directory(pool: &sqlx::SqlitePool, id: i64, name: &str, path: &str) {
    sqlx::query(
        "INSERT INTO comic_directory (id, name, path, last_modified, external_sync_enabled, hidden)
         VALUES (?, ?, ?, 0, 0, 0)",
    )
    .bind(id)
    .bind(name)
    .bind(path)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts a comic_directory with cover and cover version — used by the cover browse tests.
pub async fn insert_comic_directory_with_cover(
    pool: &sqlx::SqlitePool, id: i64, name: &str, path: &str, cover_path: &str, cover_version: i64,
) {
    sqlx::query(
        "INSERT INTO comic_directory (id, name, path, cover, last_modified, external_sync_enabled, hidden)
         VALUES (?, ?, ?, ?, ?, 0, 0)",
    )
    .bind(id)
    .bind(name)
    .bind(path)
    .bind(cover_path)
    .bind(cover_version)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts minimal metadata for a comic, linked by `comic_directory_fk`.
pub async fn insert_comic_metadata(
    pool: &sqlx::SqlitePool, id: i64, comic_directory_fk: i64, title: &str,
) {
    sqlx::query(
        "INSERT INTO comic_metadata (id, comic_directory_fk, title, description, status) VALUES (?, ?, ?, '', '')",
    )
    .bind(id)
    .bind(comic_directory_fk)
    .bind(title)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts a minimal chapter linked to a comic_directory.
pub async fn insert_chapter_archive(pool: &sqlx::SqlitePool, id: i64, comic_directory_fk: i64) {
    sqlx::query(
        "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified)
         VALUES (?, ?, 'path', '1', 0, ?, 0)",
    )
    .bind(id)
    .bind(format!("{id}"))
    .bind(comic_directory_fk)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts a chapter with a checksum — used by file sync tests that compare hashes.
pub async fn insert_chapter_archive_with_checksum(
    pool: &sqlx::SqlitePool, id: i64, comic_directory_fk: i64, chapter: &str, path: &str,
    checksum: &str,
) {
    sqlx::query(
        "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified)
         VALUES (?, ?, ?, '1', 0, ?, ?, 0)",
    )
    .bind(id)
    .bind(chapter)
    .bind(path)
    .bind(checksum)
    .bind(comic_directory_fk)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts a comic's reading progress — unique key per `comic_directory_id`.
pub async fn insert_reading_history(
    pool: &sqlx::SqlitePool, comic_directory_id: i64, chapter_archive_id: i64, last_page: i64,
) {
    sqlx::query(
        "INSERT INTO reading_history (comic_directory_id, chapter_archive_id, last_page, is_completed, updated_at)
         VALUES (?, ?, ?, 0, 1000)",
    )
    .bind(comic_directory_id)
    .bind(chapter_archive_id)
    .bind(last_page)
    .execute(pool)
    .await
    .unwrap();
}

/// Marks a chapter as read — composite key `comic_directory_id` + `chapter_archive_id`.
pub async fn insert_chapter_read(
    pool: &sqlx::SqlitePool, comic_directory_id: i64, chapter_archive_id: i64,
) {
    sqlx::query(
        "INSERT INTO chapter_read (comic_directory_id, chapter_archive_id, created_at) VALUES (?, ?, 1000)",
    )
    .bind(comic_directory_id)
    .bind(chapter_archive_id)
    .execute(pool)
    .await
    .unwrap();
}

/// Inserts a comic with N chapters and metadata — used by the Home sort/filter tests.
pub async fn insert_comic_with_chapters(
    pool: &sqlx::SqlitePool, id: i64, name: &str, title: &str, chapter_count: i64,
) {
    insert_comic_directory(pool, id, name, &format!("/mangas/{}", name.to_lowercase())).await;
    insert_comic_metadata(pool, id, id, title).await;

    for i in 0..chapter_count {
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified)
             VALUES (?, ?, ?, ?, 0, ?, 0)",
        )
        .bind(format!("{id}{i}"))
        .bind(format!("Capítulo {i}"))
        .bind(format!("/mangas/{}/cap{}", name.to_lowercase(), i))
        .bind(format!("{i:03}"))
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// Pool with a comic_directory and volumes already inserted.
pub async fn setup_test_db_with_volumes() -> sqlx::SqlitePool {
    let pool = setup_test_db_with_comic().await;

    sqlx::query(
        "INSERT INTO volume_archive (id, name, path, volume_sort, is_special, comic_directory_fk, last_modified)
         VALUES (1, 'Vol 01', '/test/v1', '1', 0, 1, 0),
                (2, 'Vol 02', '/test/v2', '2', 0, 1, 0)",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}
