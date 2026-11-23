package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ActionIcon
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.TextField
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

// The color palette belongs to this screen only: `Category.color` (Room) is an unvalidated Int —
// `CategoryManager.createCategory` persists whatever the UI sends, without checking against this list.
val categoryColors =
    listOf(
        0xFFF44336,
        0xFFE91E63,
        0xFF9C27B0,
        0xFF673AB7,
        0xFF3F51B5,
        0xFF2196F3,
        0xFF03A9F4,
        0xFF00BCD4,
        0xFF009688,
        0xFF4CAF50,
        0xFF8BC34A,
        0xFFCDDC39,
        0xFFFFEB3B,
        0xFFFFC107,
        0xFFFF9800,
        0xFFFF5722,
        0xFF795548,
        0xFF9E9E9E,
        0xFF607D8B,
    )

/**
 * Bookmark manager, the same as the desktop's `AcerolaBookmarkManager`: the form is always on
 * screen (name field + "Create" in contrast, then the color swatches), and each bookmark is a
 * surface row with its color dot and a remove button. Empty: the round-icon empty state.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Main.Config.Component.GlobalCategoryManager(
    categories: List<CategoryDto>,
    onCreateCategory: (String, Int) -> Unit,
    onDeleteCategory: (Long) -> Unit,
    modifier: Modifier = Modifier,
) {
    var name by remember { mutableStateOf("") }
    var selectedColor by remember { mutableLongStateOf(categoryColors.first()) }

    fun create() {
        val trimmed = name.trim()
        if (trimmed.isEmpty()) return
        onCreateCategory(trimmed, selectedColor.toInt())
        name = ""
    }

    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            Acerola.Component.TextField(
                value = name,
                onValueChange = { name = it },
                placeholder = stringResource(id = R.string.placeholder_category_name),
                modifier = Modifier.weight(1f),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                keyboardActions = KeyboardActions(onDone = { create() }),
            )
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_add_category),
                onClick = { create() },
                variant = ButtonVariant.Contrast,
                icon = Icons.Rounded.Add,
                enabled = name.isNotBlank(),
            )
        }

        FlowRow(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            categoryColors.forEachIndexed { index, color ->
                val selected = selectedColor == color
                val optionLabel = stringResource(id = R.string.label_category_color_option, index + 1)
                Box(
                    contentAlignment = Alignment.Center,
                    modifier =
                        Modifier
                            .size(32.dp)
                            .clip(AcerolaShapes.Full)
                            .background(Color(color))
                            .semantics {
                                contentDescription = optionLabel
                                this.selected = selected
                            }.clickable { selectedColor = color },
                ) {
                    if (selected) {
                        Icon(
                            imageVector = Icons.Rounded.Check,
                            contentDescription = null,
                            tint = AcerolaColors.OnScrim,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }
            }
        }

        if (categories.isEmpty()) {
            Acerola.Component.EmptyState(
                title = stringResource(id = R.string.label_category_empty),
                hint = stringResource(id = R.string.label_category_empty_hint),
                icon = Icons.Rounded.Bookmark,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )
        } else {
            categories.forEach { category ->
                CategoryRow(category = category, onDelete = { onDeleteCategory(category.id) })
            }
        }
    }
}

@Composable
private fun CategoryRow(
    category: CategoryDto,
    onDelete: () -> Unit,
) {
    val brand = AcerolaExtendedTheme.colors
    ProvideSurfaceLevel(AcerolaSurfaceLevel.Surface) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(AcerolaShapes.Tile)
                    .background(brand.surface)
                    .padding(start = SpacingTokens.Large, end = SpacingTokens.Small, top = SpacingTokens.Small, bottom = SpacingTokens.Small),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            Box(
                modifier =
                    Modifier
                        .size(12.dp)
                        .clip(AcerolaShapes.Full)
                        .background(Color(category.color)),
            )
            Text(
                text = category.name,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Black),
                color = brand.foreground,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
            Acerola.Component.ActionIcon(
                icon = Icons.Rounded.Delete,
                onClick = onDelete,
                contentDescription = stringResource(id = R.string.action_delete_category),
                variant = ButtonVariant.Destructive,
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun GlobalCategoryManagerPreview() {
    AcerolaTheme {
        Main.Config.Component.GlobalCategoryManager(
            categories = emptyList(),
            onCreateCategory = { _, _ -> },
            onDeleteCategory = {},
            modifier = Modifier.padding(SpacingTokens.Large),
        )
    }
}
