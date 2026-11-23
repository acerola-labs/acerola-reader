package br.acerola.comic.module.comic.template

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.module.comic.Comic
import org.junit.Rule
import org.junit.Test

class HeaderTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun `Header_should_display_remote_title_when_available`() {
        val comic =
            ComicDto(
                directory =
                    ComicDirectoryDto(
                        id = 1L,
                        name = "Folder",
                        path = "",
                        coverUri = null,
                        bannerUri = null,
                        lastModified = 0L,
                        archiveTemplateFk = null,
                    ),
                remoteInfo =
                    ComicMetadataDto(
                        title = "Fantastic Comic",
                        description = "Some random synopsis",
                        status = "Ongoing",
                    ),
            )

        composeTestRule.setContent {
            AcerolaTheme {
                Comic.Template.Header(
                    comic = comic,
                    history = null,
                    onContinueClick = { _, _ -> },
                )
            }
        }

        // Checks that the remote title is shown with priority
        composeTestRule.onNodeWithText("Fantastic Comic").assertIsDisplayed()

        // Checks that the synopsis shows up
        composeTestRule.onNodeWithText("Some random synopsis").assertIsDisplayed()
    }
}
