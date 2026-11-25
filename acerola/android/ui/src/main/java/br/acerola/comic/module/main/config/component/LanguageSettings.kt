package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Translate
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.mapper.LanguageMapper
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.component.LanguagePicker
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.type.Language
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.LanguageSettings(
    selectedLanguage: String?,
    onLanguageSelected: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val resolvedLanguage = selectedLanguage ?: Language.PT_BR.code
    val languageLabel = stringResource(id = LanguageMapper.getLabelRes(resolvedLanguage))

    Acerola.Component.LanguagePicker(
        selectedLanguage = resolvedLanguage,
        onLanguageSelected = onLanguageSelected,
        // The chosen language is the row's description, like the folder row shows the folder.
        trigger = { onClick ->
            val title = stringResource(id = R.string.title_settings_metadata_language)
            Acerola.Component.SettingRow(
                title = title,
                description = languageLabel,
                icon = Icons.Filled.Translate,
                modifier = modifier,
                control = {
                    Acerola.Component.IconButton(
                        icon = Icons.Filled.ExpandMore,
                        contentDescription = title,
                        onClick = onClick,
                    )
                },
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun LanguageSettingsPreview() {
    AcerolaTheme {
        Main.Config.Component.LanguageSettings(
            selectedLanguage = "pt-BR",
            onLanguageSelected = {},
        )
    }
}
