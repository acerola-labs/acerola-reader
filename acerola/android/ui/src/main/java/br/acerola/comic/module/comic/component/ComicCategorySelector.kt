package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Chip
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

/** Bookmark of this comic: "None" plus one chip per bookmark (with its color dot), like the desktop. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Comic.Component.ComicCategorySelector(
    selectedCategory: CategoryDto?,
    allCategories: List<CategoryDto>,
    onUpdateMangaCategory: (Long?) -> Unit,
    modifier: Modifier = Modifier,
) {
    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_comic_category),
        description = stringResource(id = R.string.description_comic_category),
        icon = Icons.Rounded.Bookmark,
        modifier = modifier,
        content = {
            FlowRow(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                Acerola.Component.Chip(
                    label = stringResource(id = R.string.label_category_none_selected),
                    selected = selectedCategory == null,
                    onClick = { onUpdateMangaCategory(null) },
                )
                allCategories.forEach { category ->
                    val isSelected = selectedCategory?.id == category.id
                    Acerola.Component.Chip(
                        label = category.name,
                        selected = isSelected,
                        dotColor = Color(category.color),
                        onClick = { onUpdateMangaCategory(if (isSelected) null else category.id) },
                    )
                }
            }
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicCategorySelectorPreview() {
    AcerolaTheme {
        Comic.Component.ComicCategorySelector(
            selectedCategory = null,
            allCategories = emptyList(),
            onUpdateMangaCategory = {},
        )
    }
}
