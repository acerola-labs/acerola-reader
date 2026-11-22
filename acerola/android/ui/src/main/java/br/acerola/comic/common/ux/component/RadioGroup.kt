package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens

/** One of N options of a setting, as brand chips: the selected one is `bg-foreground`. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Acerola.Component.RadioGroup(
    selectedIndex: Int,
    options: List<String>,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    FlowRow(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
    ) {
        options.forEachIndexed { index, label ->
            Acerola.Component.Chip(
                label = label,
                selected = selectedIndex == index,
                onClick = { onSelect(index) },
                modifier =
                    Modifier
                        .testTag("radio_button_$label")
                        .semantics { role = Role.RadioButton },
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun RadioGroupPreview() {
    AcerolaTheme {
        Acerola.Component.RadioGroup(
            selectedIndex = 0,
            options = listOf("Option 1", "Option 2"),
            onSelect = {},
        )
    }
}
