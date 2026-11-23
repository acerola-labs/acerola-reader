package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoStories
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.RadioGroup
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.ChapterPageSizeType
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.PaginationPreference(
    selected: ChapterPageSizeType?,
    onSelect: (ChapterPageSizeType) -> Unit,
    modifier: Modifier = Modifier,
) {
    val options = ChapterPageSizeType.entries
    val selectedIndex = options.indexOf(selected).takeIf { it >= 0 } ?: 0

    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_settings_chapters_per_page),
        description = stringResource(id = R.string.description_settings_chapters_per_page),
        icon = Icons.Default.AutoStories,
        modifier = modifier,
        content = {
            Acerola.Component.RadioGroup(
                selectedIndex = selectedIndex,
                options = options.map { it.key.lowercase() },
                onSelect = { index -> onSelect(options[index]) },
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PaginationPreferencePreview() {
    AcerolaTheme {
        Comic.Component.PaginationPreference(
            selected = null,
            onSelect = {},
        )
    }
}
