package br.acerola.comic.module.comic.component

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.comic.Comic
import org.junit.Rule
import org.junit.Test

class FileSyncActionsTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun should_display_file_sync_options() {
        composeTestRule.setContent {
            AcerolaTheme {
                Comic.Component.FileSyncActions(onSyncChapters = {}, onRescanCover = {})
            }
        }

        composeTestRule.onNodeWithText("Sincronizar capítulos locais").assertIsDisplayed()
        composeTestRule.onNodeWithText("Buscar capa e banner").assertIsDisplayed()
    }

    @Test
    fun should_call_onSyncChapters_when_clicking_its_button() {
        var clicked = false
        composeTestRule.setContent {
            AcerolaTheme {
                Comic.Component.FileSyncActions(onSyncChapters = { clicked = true }, onRescanCover = {})
            }
        }

        composeTestRule.onNodeWithContentDescription("Sincronizar").performClick()
        assert(clicked)
    }
}
