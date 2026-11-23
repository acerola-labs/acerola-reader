package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Code
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.TemplateManager(
    onManageTemplates: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_chapter_naming_templates),
        description = stringResource(id = R.string.description_template_config_activity),
        icon = Icons.Default.Code,
        modifier = modifier,
        content = {
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_config_open),
                onClick = onManageTemplates,
                variant = ButtonVariant.Secondary,
                trailingIcon = Icons.AutoMirrored.Filled.KeyboardArrowRight,
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TemplateManagerPreview() {
    AcerolaTheme {
        Main.Config.Component.TemplateManager(
            onManageTemplates = {},
        )
    }
}
