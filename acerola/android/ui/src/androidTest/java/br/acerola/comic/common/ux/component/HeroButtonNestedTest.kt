package br.acerola.comic.common.ux.component

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Sync
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import org.junit.Rule
import org.junit.Test

class HeroButtonNestedTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun should_display_nested_content_when_provided() {
        composeTestRule.setContent {
            AcerolaTheme {
                Acerola.Component.HeroButton(
                    title = "Main Item",
                    icon = Icons.Default.Settings,
                    bottomContent = {
                        Acerola.Component.HeroNestedButton(
                            title = "Nested Item",
                            icon = Icons.Default.Sync,
                            onClick = {},
                        )
                    },
                )
            }
        }

        composeTestRule.onNodeWithText("Main Item").assertIsDisplayed()
        composeTestRule.onNodeWithText("Nested Item").assertIsDisplayed()
    }
}
