package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.mapper.LanguageMapper
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.ui.R

@Composable
fun Acerola.Component.LanguagePicker(
    selectedLanguage: String?,
    onLanguageSelected: (String) -> Unit,
    trigger: @Composable (onClick: () -> Unit) -> Unit,
) {
    var showSheet by remember { mutableStateOf(false) }

    trigger { showSheet = true }

    if (showSheet) {
        val configuration = LocalConfiguration.current
        val maxHeight = (configuration.screenHeightDp * 0.55f).dp

        Acerola.Component.AdaptiveSheet(
            onDismissRequest = { showSheet = false },
            isScrollable = false,
        ) {
            Acerola.Component.SheetHeader(title = stringResource(id = R.string.title_settings_metadata_language))

            LazyColumn(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .heightIn(max = maxHeight),
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 32.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                items(LanguageMapper.getAllCodes()) { code ->
                    Acerola.Component.OptionRow(
                        label = stringResource(id = LanguageMapper.getLabelRes(code)),
                        selected = code == selectedLanguage,
                        onClick = {
                            onLanguageSelected(code)
                            showSheet = false
                        },
                    )
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun LanguagePickerPreview() {
    AcerolaTheme {
        Acerola.Component.LanguagePicker(
            selectedLanguage = "pt-BR",
            onLanguageSelected = {},
            trigger = { onClick -> Acerola.Component.Button(text = "Language", onClick = onClick) },
        )
    }
}
