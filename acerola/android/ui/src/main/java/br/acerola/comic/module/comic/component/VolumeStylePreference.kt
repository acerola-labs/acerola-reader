package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.LibraryBooks
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.RadioGroup
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.VolumeStylePreference(
    selected: VolumeViewType,
    onSelect: (VolumeViewType) -> Unit,
    modifier: Modifier = Modifier,
) {
    val options = listOf(VolumeViewType.COVER_VOLUME, VolumeViewType.VOLUME)
    val selectedIndex = options.indexOf(selected).takeIf { it >= 0 } ?: 0

    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_settings_volume_style),
        description = stringResource(id = R.string.description_settings_volume_style),
        icon = Icons.AutoMirrored.Filled.LibraryBooks,
        modifier = modifier,
        content = {
            Acerola.Component.RadioGroup(
                selectedIndex = selectedIndex,
                options = options.map { volumeStyleLabel(it) },
                onSelect = { index -> onSelect(options[index]) },
            )
        },
    )
}

@Composable
private fun volumeStyleLabel(viewType: VolumeViewType): String =
    when (viewType) {
        VolumeViewType.COVER_VOLUME -> stringResource(R.string.label_volume_style_cover)
        else -> stringResource(R.string.label_volume_style_normal)
    }

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun VolumeStylePreferencePreview() {
    AcerolaTheme {
        Comic.Component.VolumeStylePreference(
            selected = VolumeViewType.VOLUME,
            onSelect = {},
        )
    }
}
