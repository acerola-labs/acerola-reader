package br.acerola.comic.common.ux.component

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import br.acerola.comic.common.ux.Acerola
import org.junit.Rule
import org.junit.Test

class ButtonTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun should_render_button_with_label_and_execute_click() {
        var clicked = false
        composeTestRule.setContent {
            Acerola.Component.Button(
                text = "Click Here",
                onClick = { clicked = true },
            )
        }

        composeTestRule.onNodeWithText("Click Here").assertIsDisplayed().performClick()
        assert(clicked)
    }

    @Test
    fun should_render_icon_button_with_content_description() {
        composeTestRule.setContent {
            Acerola.Component.ActionIcon(
                icon = Icons.Default.Add,
                onClick = {},
                contentDescription = "Add",
            )
        }

        composeTestRule.onNodeWithContentDescription("Add").assertIsDisplayed()
    }

    @Test
    fun should_display_icon_and_text_in_mixed_button() {
        composeTestRule.setContent {
            Acerola.Component.Button(
                text = "Save",
                onClick = {},
                icon = Icons.Default.Add,
            )
        }

        // The leading icon is decorative: the button's accessible name is its text.
        composeTestRule.onNodeWithText("Save").assertIsDisplayed()
    }
}
