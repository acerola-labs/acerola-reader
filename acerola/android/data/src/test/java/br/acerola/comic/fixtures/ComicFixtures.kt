package br.acerola.comic.fixtures

import br.acerola.comic.dto.archive.ChapterFileDto

/**
 * Reusable fixtures for comic tests.
 * Follow the AAA pattern and write test descriptions in English.
 */
object ComicFixtures {
    fun createChapterFileDto(
        id: Long = 1L,
        name: String = "Capítulo 1",
        path: String = "/path/to/cap1",
        chapterSort: String = "0001",
    ) = ChapterFileDto(
        id = id,
        name = name,
        path = path,
        chapterSort = chapterSort,
    )

    fun createChapterList(count: Int = 3): List<ChapterFileDto> =
        (1..count).map {
            createChapterFileDto(
                id = it.toLong(),
                name = "Capítulo $it",
                chapterSort = it.toString().padStart(4, '0'),
            )
        }
}
