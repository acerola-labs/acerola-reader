package br.acerola.comic.module.main.home.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.Chip
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SheetHeader
import br.acerola.comic.common.ux.component.Switch
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.config.preference.types.ComicSortType
import br.acerola.comic.config.preference.types.HomeSortPreference
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.home.state.FilterSettings
import br.acerola.comic.pattern.metadata.MetadataSource
import br.acerola.comic.ui.R

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun Main.Home.Component.HomeFilterSheet(
    sortSettings: HomeSortPreference,
    filterSettings: FilterSettings,
    categories: List<CategoryDto>,
    onSortChange: (HomeSortPreference) -> Unit,
    onFilterChange: (FilterSettings) -> Unit,
    onDismiss: () -> Unit,
) {
    // Same panel as the desktop's filter sheet: sort as chips (tap the active one again to flip
    // the direction), the "show hidden" switch row, then bookmark and source chips.
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        isScrollable = false,
    ) {
        Acerola.Component.SheetHeader(title = stringResource(id = R.string.title_home_filter_sheet))

        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(start = SpacingTokens.Huge, end = SpacingTokens.Huge, top = SpacingTokens.Small, bottom = SpacingTokens.Giant),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraLarge),
        ) {
            FilterGroup(title = stringResource(id = R.string.subtitle_home_sort)) {
                ComicSortType.entries.forEach { type ->
                    val label =
                        when (type) {
                            ComicSortType.TITLE -> stringResource(id = R.string.label_sort_title)
                            ComicSortType.CHAPTER_COUNT -> stringResource(id = R.string.label_sort_chapter_count)
                            ComicSortType.LAST_UPDATE -> stringResource(id = R.string.label_sort_last_update)
                        }
                    val selected = sortSettings.type == type
                    Acerola.Component.Chip(
                        label = label,
                        selected = selected,
                        icon =
                            if (!selected) {
                                null
                            } else if (sortSettings.direction == SortDirection.ASCENDING) {
                                Icons.Default.ArrowUpward
                            } else {
                                Icons.Default.ArrowDownward
                            },
                        onClick = {
                            if (selected) {
                                val flipped =
                                    if (sortSettings.direction == SortDirection.ASCENDING) SortDirection.DESCENDING else SortDirection.ASCENDING
                                onSortChange(sortSettings.copy(direction = flipped))
                            } else {
                                onSortChange(sortSettings.copy(type = type))
                            }
                        },
                    )
                }
            }

            ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) {
                Acerola.Component.SettingRow(
                    title = stringResource(id = R.string.label_home_filter_show_hidden),
                    icon = Icons.Default.VisibilityOff,
                    onClick = { onFilterChange(filterSettings.copy(showHidden = !filterSettings.showHidden)) },
                    control = {
                        Acerola.Component.Switch(
                            checked = filterSettings.showHidden,
                            onCheckedChange = { onFilterChange(filterSettings.copy(showHidden = it)) },
                        )
                    },
                )
            }

            FilterGroup(title = stringResource(id = R.string.subtitle_home_filter_categories)) {
                Acerola.Component.Chip(
                    label = stringResource(id = R.string.label_filter_all),
                    selected = filterSettings.bookmarkCategoryId == null,
                    onClick = { onFilterChange(filterSettings.copy(bookmarkCategoryId = null)) },
                )
                categories.forEach { category ->
                    Acerola.Component.Chip(
                        label = category.name,
                        selected = filterSettings.bookmarkCategoryId == category.id,
                        dotColor = Color(category.color),
                        onClick = { onFilterChange(filterSettings.copy(bookmarkCategoryId = category.id)) },
                    )
                }
            }

            FilterGroup(title = stringResource(id = R.string.subtitle_home_filter_sources)) {
                Acerola.Component.Chip(
                    label = stringResource(id = R.string.label_filter_all),
                    selected = filterSettings.metadataSource == null,
                    onClick = { onFilterChange(filterSettings.copy(metadataSource = null)) },
                )
                MetadataSource.entries.forEach { source ->
                    val label =
                        when (source) {
                            MetadataSource.MANGADEX -> stringResource(id = R.string.label_source_mangadex)
                            MetadataSource.ANILIST -> stringResource(id = R.string.label_source_anilist)
                            MetadataSource.COMIC_INFO -> stringResource(id = R.string.label_source_comic_info)
                        }
                    Acerola.Component.Chip(
                        label = label,
                        selected = filterSettings.metadataSource == source.displayName,
                        onClick = { onFilterChange(filterSettings.copy(metadataSource = source.displayName)) },
                    )
                }
                Acerola.Component.Chip(
                    label = stringResource(id = R.string.label_filter_none),
                    selected = filterSettings.metadataSource == "NONE",
                    onClick = { onFilterChange(filterSettings.copy(metadataSource = "NONE")) },
                )
            }
        }
    }
}

// A labelled chip group of the filter panel: `font-black` label over the chips.
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun FilterGroup(
    title: String,
    chips: @Composable () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
            color = AcerolaExtendedTheme.colors.foreground,
        )
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            chips()
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HomeFilterSheetPreview() {
    AcerolaTheme {
        Main.Home.Component.HomeFilterSheet(
            sortSettings =
                HomeSortPreference(
                    type = ComicSortType.TITLE,
                    direction = SortDirection.ASCENDING,
                ),
            filterSettings = FilterSettings(),
            categories = emptyList(),
            onSortChange = {},
            onFilterChange = {},
            onDismiss = {},
        )
    }
}
