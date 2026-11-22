package br.acerola.comic.module.comic.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.Chip
import br.acerola.comic.common.ux.component.SheetHeader
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.config.preference.types.ChapterSortPreferenceData
import br.acerola.comic.config.preference.types.ChapterSortType
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

@Composable
fun Comic.Component.ChapterSortSheet(
    sortSettings: ChapterSortPreferenceData,
    onSortChange: (ChapterSortPreferenceData) -> Unit,
    onDismiss: () -> Unit,
) {
    // Same as the home filter's sort: one chip per order, tap the active one again to flip it.
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
    ) {
        Acerola.Component.SheetHeader(title = stringResource(id = R.string.title_chapter_sort_sheet))

        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(start = SpacingTokens.Huge, end = SpacingTokens.Huge, top = SpacingTokens.Small, bottom = SpacingTokens.Giant),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            ChapterSortType.entries.forEach { type ->
                val selected = sortSettings.type == type
                Acerola.Component.Chip(
                    label =
                        when (type) {
                            ChapterSortType.NUMBER -> stringResource(id = R.string.label_sort_number)
                            ChapterSortType.LAST_UPDATE -> stringResource(id = R.string.label_sort_last_update)
                        },
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
    }
}
