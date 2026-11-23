package br.acerola.comic.module.main.common.component

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.platform.app.InstrumentationRegistry
import br.acerola.comic.fixtures.ComicFixtures
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import org.junit.Rule
import org.junit.Test

class ComicActionsSheetTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    private val context = InstrumentationRegistry.getInstrumentation().targetContext

    @Test
    fun should_display_main_actions_when_opening_sheet() {
        val comic = ComicFixtures.createMangaUiState().comic

        composeTestRule.setContent {
            Main.Common.Component.ComicActionsSheet(
                comic = comic,
                categories = emptyList(),
                onHide = {},
                onDelete = {},
                onClearMetadata = {},
                onBookmark = {},
                onDismiss = {},
            )
        }

        composeTestRule.onNodeWithText(context.getString(R.string.action_bookmark)).assertIsDisplayed()
        composeTestRule.onNodeWithText(context.getString(R.string.action_hide)).assertIsDisplayed()
        composeTestRule.onNodeWithText(context.getString(R.string.action_delete)).assertIsDisplayed()
    }

    @Test
    fun should_display_confirmation_dialog_when_clicking_hide() {
        composeTestRule.setContent {
            Main.Common.Component.ComicActionsSheet(
                comic = ComicFixtures.createMangaUiState().comic,
                categories = emptyList(),
                onHide = {},
                onDelete = {},
                onClearMetadata = {},
                onBookmark = {},
                onDismiss = {},
            )
        }

        composeTestRule.onNodeWithText(context.getString(R.string.action_hide)).performClick()

        // Checks that the hide dialog title shows up
        composeTestRule.onNodeWithText(context.getString(R.string.dialog_hide_title)).assertIsDisplayed()
        composeTestRule.onNodeWithText(context.getString(R.string.action_cancel)).assertIsDisplayed()
    }

    @Test
    fun should_display_confirmation_dialog_when_clicking_delete() {
        composeTestRule.setContent {
            Main.Common.Component.ComicActionsSheet(
                comic = ComicFixtures.createMangaUiState().comic,
                categories = emptyList(),
                onHide = {},
                onDelete = {},
                onClearMetadata = {},
                onBookmark = {},
                onDismiss = {},
            )
        }

        composeTestRule.onNodeWithText(context.getString(R.string.action_delete)).performClick()

        // Checks that the delete dialog title shows up
        composeTestRule.onNodeWithText(context.getString(R.string.dialog_delete_title)).assertIsDisplayed()
    }
}
