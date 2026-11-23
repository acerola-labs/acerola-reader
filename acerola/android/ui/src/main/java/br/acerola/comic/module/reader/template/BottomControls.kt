package br.acerola.comic.module.reader.template

import android.content.res.Configuration
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.automirrored.filled.NavigateBefore
import androidx.compose.material.icons.automirrored.filled.NavigateNext
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.component.ProgressBar
import br.acerola.comic.common.ux.modifier.float
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.reader.Reader
import br.acerola.comic.ui.R

private const val TAG = "ReaderBottomControls"

@Composable
fun Reader.Template.BottomControls(
    pageCount: Int,
    currentPage: Int,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    onNextChapterClick: () -> Unit,
    onPreviousChapterClick: () -> Unit,
    isChapterRead: Boolean = false,
    hasNextChapter: Boolean = false,
    hasPreviousChapter: Boolean = false,
    enableNavigation: Boolean = true,
    isLoading: Boolean = false,
) {
    AcerolaLogger.d(
        TAG,
        "Recomposed: isChapterRead=$isChapterRead, hasNextChapter=$hasNextChapter, page=$currentPage/$pageCount",
        LogSource.UI,
    )

    // Solid floating sheet (`rounded-sheet bg-card shadow-float`) over the page, no glass.
    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(SpacingTokens.Large),
        contentAlignment = Alignment.Center,
    ) {
        ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) {
            Column(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .float(AcerolaShapes.Sheet)
                        .clip(AcerolaShapes.Sheet)
                        .background(AcerolaExtendedTheme.colors.card)
                        .animateContentSize()
                        .padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Large),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
            ) {
                PageIndicator(currentPage = currentPage, pageCount = pageCount)

                ChapterNavigation(
                    isLoading = isLoading,
                    enableNavigation = enableNavigation,
                    currentPage = currentPage,
                    pageCount = pageCount,
                    hasPreviousChapter = hasPreviousChapter,
                    hasNextChapter = hasNextChapter,
                    isChapterRead = isChapterRead,
                    onPreviousChapterClick = onPreviousChapterClick,
                    onPrevClick = onPrevClick,
                    onNextClick = onNextClick,
                    onNextChapterClick = onNextChapterClick,
                )
            }
        }
    }
}

@Composable
private fun PageIndicator(
    currentPage: Int,
    pageCount: Int,
) {
    Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.Bottom,
        ) {
            Text(
                text = stringResource(id = R.string.label_reader_pages_prefix),
                style = MaterialTheme.typography.labelMedium,
                color = AcerolaExtendedTheme.colors.mutedForeground,
            )
            Text(
                text = "${currentPage + 1} / $pageCount",
                style = MaterialTheme.typography.titleMedium,
                color = AcerolaExtendedTheme.colors.foreground,
            )
        }

        val progress = if (pageCount > 0) (currentPage + 1).toFloat() / pageCount.toFloat() else 0f
        Acerola.Component.ProgressBar(progress = progress)
    }
}

@Composable
private fun ChapterNavigation(
    isLoading: Boolean,
    enableNavigation: Boolean,
    currentPage: Int,
    pageCount: Int,
    hasPreviousChapter: Boolean,
    hasNextChapter: Boolean,
    isChapterRead: Boolean,
    onPreviousChapterClick: () -> Unit,
    onPrevClick: () -> Unit,
    onNextClick: () -> Unit,
    onNextChapterClick: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            if (hasPreviousChapter) {
                Acerola.Component.Button(
                    text = stringResource(id = R.string.label_reader_previous_chapter),
                    onClick = onPreviousChapterClick,
                    variant = ButtonVariant.Secondary,
                    icon = Icons.AutoMirrored.Filled.NavigateBefore,
                    enabled = !isLoading,
                )
            }
        }

        if (enableNavigation) {
            Row(
                modifier = Modifier.padding(horizontal = SpacingTokens.Small),
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                Acerola.Component.IconButton(
                    icon = Icons.AutoMirrored.Filled.KeyboardArrowLeft,
                    contentDescription = stringResource(id = R.string.description_icon_pagination_previous),
                    onClick = onPrevClick,
                    enabled = currentPage > 0,
                )
                Acerola.Component.IconButton(
                    icon = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                    contentDescription = stringResource(id = R.string.description_icon_pagination_next),
                    onClick = onNextClick,
                    enabled = currentPage < pageCount - 1,
                )
            }
        }

        Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterEnd) {
            if (hasNextChapter) {
                Acerola.Component.Button(
                    text = stringResource(id = R.string.label_reader_next_chapter),
                    onClick = onNextChapterClick,
                    variant = ButtonVariant.Primary,
                    trailingIcon = Icons.AutoMirrored.Filled.NavigateNext,
                    enabled = !isLoading,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun BottomControlsPreview() {
    AcerolaTheme {
        Reader.Template.BottomControls(
            pageCount = 10,
            currentPage = 1,
            onPrevClick = {},
            onNextClick = {},
            onNextChapterClick = {},
            onPreviousChapterClick = {},
        )
    }
}
