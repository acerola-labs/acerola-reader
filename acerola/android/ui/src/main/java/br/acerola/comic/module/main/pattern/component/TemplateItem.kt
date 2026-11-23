package br.acerola.comic.module.main.pattern.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.rounded.Code
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ActionIcon
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.archive.ArchiveTemplateDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import br.acerola.comic.util.sort.SortType

/** One template, the desktop's template row: name and pattern; default ones carry a "Default" badge, the rest edit/remove. */
@Composable
fun Main.Pattern.Component.TemplateItem(
    template: ArchiveTemplateDto,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    Acerola.Component.SettingRow(
        title = template.label,
        description = template.pattern,
        icon = Icons.Rounded.Code,
        control = {
            if (template.isDefault) {
                Acerola.Component.Badge(text = stringResource(id = R.string.label_system_template), tone = Tone.Primary)
            } else {
                Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall)) {
                    Acerola.Component.ActionIcon(
                        icon = Icons.Default.Edit,
                        onClick = onEdit,
                        contentDescription = stringResource(id = R.string.description_icon_edit_template),
                    )
                    Acerola.Component.ActionIcon(
                        icon = Icons.Default.Delete,
                        onClick = onDelete,
                        contentDescription = stringResource(id = R.string.description_icon_delete_template),
                        variant = ButtonVariant.Destructive,
                    )
                }
            }
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TemplateItemPreview() {
    AcerolaTheme {
        Main.Pattern.Component.TemplateItem(
            template =
                ArchiveTemplateDto(
                    id = 1L,
                    label = "Default",
                    pattern = "{title} - {chapter}",
                    type = SortType.CHAPTER,
                    isDefault = true,
                ),
            onEdit = {},
            onDelete = {},
        )
    }
}
