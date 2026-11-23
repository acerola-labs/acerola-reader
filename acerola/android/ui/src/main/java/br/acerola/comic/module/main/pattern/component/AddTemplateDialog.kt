package br.acerola.comic.module.main.pattern.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.SegmentedControl
import br.acerola.comic.common.ux.component.SheetHeader
import br.acerola.comic.common.ux.component.TextField
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.component.colors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import br.acerola.comic.util.sort.SortType

/** Create / edit a template, the desktop's template form: kind as chip tabs, name and pattern pills, the macro help. */
@Composable
fun Main.Pattern.Component.AddTemplateDialog(
    onDismiss: () -> Unit,
    onConfirm: (String, String, SortType) -> Unit,
    initialLabel: String = "",
    initialPattern: String = "",
    initialType: SortType = SortType.CHAPTER,
    isEditMode: Boolean = false,
) {
    var label by remember { mutableStateOf(initialLabel) }
    var pattern by remember { mutableStateOf(initialPattern) }
    var type by remember { mutableStateOf(initialType) }
    val brand = AcerolaExtendedTheme.colors

    Acerola.Component.AdaptiveSheet(onDismissRequest = onDismiss) {
        Acerola.Component.SheetHeader(
            title = stringResource(id = if (isEditMode) R.string.title_dialog_edit_template else R.string.title_dialog_new_template),
        )

        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.Huge, vertical = SpacingTokens.Small),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            Text(
                text = stringResource(id = R.string.label_template_applies_to),
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
                color = brand.foreground,
            )
            Acerola.Component.SegmentedControl(
                options =
                    SortType.entries.map {
                        stringResource(id = if (it == SortType.CHAPTER) R.string.label_sort_type_chapter else R.string.label_sort_type_volume)
                    },
                selectedIndex = SortType.entries.indexOf(type),
                onSelect = { type = SortType.entries[it] },
            )

            Acerola.Component.TextField(
                value = label,
                onValueChange = { label = it },
                placeholder = stringResource(id = R.string.label_template_label),
            )
            Acerola.Component.TextField(
                value = pattern,
                onValueChange = { pattern = it },
                placeholder = stringResource(id = R.string.placeholder_template_pattern),
            )

            val info = Tone.Info.colors()
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clip(AcerolaShapes.Tile)
                        .background(info.container)
                        .padding(SpacingTokens.Large),
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
            ) {
                Icon(Icons.Default.Info, contentDescription = null, tint = info.content, modifier = Modifier.size(SizeTokens.IconSmall))
                Text(
                    text = stringResource(id = R.string.description_template_macros),
                    style = MaterialTheme.typography.bodySmall,
                    color = info.content,
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth().padding(top = SpacingTokens.Small, bottom = SpacingTokens.Large),
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                Acerola.Component.Button(
                    text = stringResource(id = R.string.action_cancel),
                    onClick = onDismiss,
                    variant = ButtonVariant.Secondary,
                    modifier = Modifier.weight(1f),
                )
                Acerola.Component.Button(
                    text = stringResource(id = if (isEditMode) R.string.action_save else R.string.action_add),
                    onClick = { if (label.isNotBlank() && pattern.isNotBlank()) onConfirm(label, pattern, type) },
                    variant = ButtonVariant.Contrast,
                    enabled = label.isNotBlank() && pattern.isNotBlank(),
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AddTemplateDialogPreview() {
    AcerolaTheme {
        Main.Pattern.Component.AddTemplateDialog(
            onDismiss = {},
            onConfirm = { _, _, _ -> },
        )
    }
}
