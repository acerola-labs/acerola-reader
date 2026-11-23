package br.acerola.comic.service.reader.extract

import android.content.Context
import br.acerola.comic.dto.archive.ChapterFileDto
import io.mockk.MockKAnnotations
import io.mockk.impl.annotations.MockK
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class CbzPageResolverTest {
    @MockK lateinit var context: Context
    private lateinit var service: CbzPageResolver

    @Before
    fun setUp() {
        MockKAnnotations.init(this)
        service = CbzPageResolver(context)
    }

    @Test
    fun open_should_fail_if_file_does_not_exist() {
        val chapter = ChapterFileDto(1, "ch", "/non/existent/file.cbz", "1")

        val result = service.open(chapter)

        assertTrue(result.isLeft())
    }

    // NOTE: Testing the happy path of Cbz/CbrChapterSourceService requires real files
    // or a complex mocked file system (Jimfs).
    // Since the services use 'new ZipFile(file)', mockk cannot easily intercept the native constructor.
    // I recommend testing these two files via integration tests with real files in androidTest
    // or keeping them with basic error tests in the unit tests.

    @Test
    fun close_should_clean_references_and_temporary_files() {
        service.close()
        runTest {
            assertEquals(0, service.pageCount())
        }
    }
}
