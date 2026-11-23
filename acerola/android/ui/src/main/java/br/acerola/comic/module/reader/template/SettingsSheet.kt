package br.acerola.comic.module.reader.template

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoStories
import androidx.compose.material.icons.filled.VerticalAlignBottom
import androidx.compose.material.icons.filled.ViewHeadline
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.OptionRow
import br.acerola.comic.common.ux.component.SheetHeader
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.config.preference.types.ReadingMode
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.ui.R

@Composable
fun Reader.Template.SettingsSheet(
    onDismissRequest: () -> Unit,
    currentMode: ReadingMode,
    onModeSelected: (ReadingMode) -> Unit,
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismissRequest,
        isScrollable = false,
    ) {
        Acerola.Component.SheetHeader(
            title = stringResource(id = R.string.label_reader_config_title),
            lead = stringResource(id = R.string.label_reader_layout_title),
        )

        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(start = SpacingTokens.Large, end = SpacingTokens.Large, top = SpacingTokens.Small, bottom = SpacingTokens.Giant),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall),
        ) {
            Acerola.Component.OptionRow(
                label = stringResource(id = R.string.label_reader_mode_horizontal),
                icon = Icons.Default.AutoStories,
                selected = currentMode == ReadingMode.HORIZONTAL,
                onClick = { onModeSelected(ReadingMode.HORIZONTAL) },
            )
            Acerola.Component.OptionRow(
                label = stringResource(id = R.string.label_reader_mode_vertical),
                icon = Icons.Default.ViewHeadline,
                selected = currentMode == ReadingMode.VERTICAL,
                onClick = { onModeSelected(ReadingMode.VERTICAL) },
            )
            Acerola.Component.OptionRow(
                label = stringResource(id = R.string.label_reader_mode_webtoon),
                icon = Icons.Default.VerticalAlignBottom,
                selected = currentMode == ReadingMode.WEBTOON,
                onClick = { onModeSelected(ReadingMode.WEBTOON) },
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SettingsSheetPreview() {
    AcerolaTheme {
        Reader.Template.SettingsSheet(
            onDismissRequest = {},
            currentMode = ReadingMode.VERTICAL,
            onModeSelected = {},
        )
    }
}
