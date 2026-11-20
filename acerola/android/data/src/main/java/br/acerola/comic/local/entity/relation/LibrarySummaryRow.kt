package br.acerola.comic.local.entity.relation

import androidx.room.ColumnInfo

data class LibrarySummaryRow(
    @ColumnInfo(name = "comic_name")
    val comicName: String,
    @ColumnInfo(name = "chapter_count")
    val chapterCount: Int,
    // Reuses `comic_directory.last_modified` — no new hash. The peer compares it against the
    // version already cached locally to decide whether it needs to fetch a new cover via
    // `acerola/browse-cover/1`.
    @ColumnInfo(name = "cover_version")
    val coverVersion: Long,
)
