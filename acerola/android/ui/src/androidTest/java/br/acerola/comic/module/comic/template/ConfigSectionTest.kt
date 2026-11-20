package br.acerola.comic.module.comic.template

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import br.acerola.comic.fixtures.ComicFixtures
import br.acerola.comic.module.comic.Comic
import org.junit.Rule
import org.junit.Test

class ConfigSectionTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun should_display_all_comic_config_sections() {
        val uiState = ComicFixtures.createMangaUiState()

        composeTestRule.setContent {
            LazyColumn {
                Comic.Template.configSection(
                    scope = this,
                    uiState = uiState,
                    // "Leitura" (Reading) collapses by default (see Acerola.Component.AccordionCard) — expanded
                    // so the test can inspect the content. "Sincronização" (Sync) is always flat/open.
                    expandedCategories = setOf("reading"),
                    onToggleCategory = {},
                    onAction = {},
                    onSyncAction = {},
                )
            }
        }

        // Titles of the 3 categories (Reading/Sync/Advanced) are
        // Acerola.Component.AccordionCard headers, exact text from strings.xml.
        composeTestRule.onNodeWithText("Leitura", ignoreCase = true).assertIsDisplayed()
        composeTestRule.onNodeWithText("Sincronização", ignoreCase = true).assertIsDisplayed()

        // MangaDex is shown inside ComicExternalSyncToggle's expandable content, in the
        // "Sincronização" (Sync) category (always flat) — the fixture already has externalSyncEnabled = true.
        composeTestRule.onNodeWithText("MangaDex", ignoreCase = true).assertIsDisplayed()
    }
}
