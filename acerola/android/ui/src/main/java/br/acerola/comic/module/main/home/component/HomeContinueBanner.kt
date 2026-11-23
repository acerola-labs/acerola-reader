package br.acerola.comic.module.main.home.component

import android.content.res.Configuration
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.modifier.lift
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.history.ReadingHistoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R
import coil.compose.AsyncImage
import coil.request.ImageRequest
import kotlin.math.roundToInt

private val BannerExpandedHeightPortrait = 280.dp
private val BannerExpandedHeightLandscape = 180.dp
private val BannerCollapsedHeight = 72.dp
private val DragThreshold = 120f

@Composable
fun Main.Home.Component.HomeContinueBanner(
    comic: ComicDto,
    history: ReadingHistoryDto,
    isExpanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    onContinueClick: () -> Unit,
    onComicClick: () -> Unit,
    modifier: Modifier = Modifier,
    isLandscape: Boolean = false,
) {
    val context = LocalContext.current
    var dragOffset by remember { mutableFloatStateOf(0f) }

    val coverUri = comic.directory.coverUri ?: comic.remoteInfo?.cover?.url
    val bannerUri = comic.directory.bannerUri ?: comic.remoteInfo?.banner?.url ?: coverUri
    val title = comic.remoteInfo?.title ?: comic.directory.name

    val expandedHeight = if (isLandscape) BannerExpandedHeightLandscape else BannerExpandedHeightPortrait
    val targetHeight = if (isExpanded) expandedHeight.value else 88f
    val animatedHeight by animateFloatAsState(
        targetValue = targetHeight,
        animationSpec =
            spring(
                dampingRatio = Spring.DampingRatioMediumBouncy,
                stiffness = Spring.StiffnessLow,
            ),
        label = "banner_height",
    )

    // Brand card (`rounded-card bg-card shadow-lift`); expanded it shows the banner art under a scrim.
    androidx.compose.material3.Surface(
        shape = AcerolaShapes.Card,
        color = AcerolaExtendedTheme.colors.card,
        modifier =
            modifier
                .fillMaxWidth()
                .height(animatedHeight.dp)
                .lift(AcerolaShapes.Card)
                .clip(AcerolaShapes.Card)
                .clickable {
                    if (!isExpanded && !isLandscape) {
                        onExpandedChange(true)
                        return@clickable
                    }
                    onComicClick()
                }.then(
                    if (!isLandscape) {
                        Modifier.pointerInput(isExpanded) {
                            detectVerticalDragGestures(
                                onDragEnd = {
                                    when {
                                        isExpanded && dragOffset < -DragThreshold -> onExpandedChange(false)
                                        !isExpanded && dragOffset > DragThreshold -> onExpandedChange(true)
                                    }
                                    dragOffset = 0f
                                },
                                onVerticalDrag = { _, dragAmount ->
                                    dragOffset =
                                        if (isExpanded) {
                                            (dragOffset + dragAmount).coerceIn(-400f, 0f)
                                        } else {
                                            (dragOffset + dragAmount).coerceIn(0f, 400f)
                                        }
                                },
                            )
                        }
                    } else {
                        Modifier
                    },
                ),
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            if (isExpanded && bannerUri != null) {
                AsyncImage(
                    model =
                        ImageRequest
                            .Builder(context)
                            .data(bannerUri)
                            .memoryCacheKey("${bannerUri}_${comic.directory.lastModified}")
                            .diskCacheKey("${bannerUri}_${comic.directory.lastModified}")
                            .crossfade(true)
                            .build(),
                    contentDescription = null,
                    contentScale = ContentScale.Crop,
                    modifier = Modifier.matchParentSize(),
                )

                Box(
                    modifier =
                        Modifier
                            .matchParentSize()
                            .background(
                                Brush.verticalGradient(
                                    colors =
                                        listOf(
                                            Color.Transparent,
                                            AcerolaColors.Scrim.copy(alpha = 0.4f),
                                            AcerolaColors.Scrim.copy(alpha = 0.8f),
                                        ),
                                ),
                            ),
                )
            }

            if (!isExpanded) {
                Row(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    if (coverUri != null) {
                        AsyncImage(
                            model =
                                ImageRequest
                                    .Builder(context)
                                    .data(coverUri)
                                    .memoryCacheKey("${coverUri}_${comic.directory.lastModified}")
                                    .diskCacheKey("${coverUri}_${comic.directory.lastModified}")
                                    .crossfade(true)
                                    .build(),
                            contentDescription = null,
                            contentScale = ContentScale.Crop,
                            modifier =
                                Modifier
                                    .size(64.dp)
                                    .clip(AcerolaShapes.Inner),
                        )
                        Spacer(modifier = Modifier.width(16.dp))
                    }

                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.Center,
                    ) {
                        Text(
                            text = stringResource(R.string.label_banner_continue_reading),
                            style = MaterialTheme.typography.labelMedium,
                            color = AcerolaExtendedTheme.colors.mutedForeground,
                        )
                        Spacer(modifier = Modifier.height(2.dp))
                        Text(
                            text = title,
                            style = MaterialTheme.typography.titleMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            color = AcerolaExtendedTheme.colors.foreground,
                        )
                        Text(
                            text = stringResource(R.string.label_banner_chapter_progress, history.chapterSort),
                            style = MaterialTheme.typography.bodySmall,
                            color = AcerolaExtendedTheme.colors.mutedForeground,
                            maxLines = 1,
                        )
                    }

                    Spacer(modifier = Modifier.width(8.dp))

                    Acerola.Component.IconButton(
                        icon = Icons.Filled.PlayArrow,
                        contentDescription = stringResource(R.string.label_banner_continue),
                        onClick = onContinueClick,
                        variant = ButtonVariant.Primary,
                        size = 48.dp,
                    )
                }
            } else {
                Column(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .height(expandedHeight)
                            .padding(if (isLandscape) SpacingTokens.Medium else SpacingTokens.Large)
                            .offset { IntOffset(0, (dragOffset * 0.5f).roundToInt()) },
                    verticalArrangement = Arrangement.Bottom,
                ) {
                    Text(
                        text = title,
                        style =
                            MaterialTheme.typography.headlineSmall.copy(
                                color = if (bannerUri != null) AcerolaColors.OnScrim else AcerolaExtendedTheme.colors.foreground,
                            ),
                        maxLines = if (isLandscape) 1 else 2,
                        overflow = TextOverflow.Ellipsis,
                    )

                    Spacer(modifier = Modifier.height(if (isLandscape) SpacingTokens.ExtraSmall else SpacingTokens.Small))

                    Text(
                        text = stringResource(R.string.label_banner_chapter_progress, history.chapterSort),
                        style =
                            MaterialTheme.typography.bodyLarge.copy(
                                color =
                                    if (bannerUri !=
                                        null
                                    ) {
                                        AcerolaColors.OnScrim.copy(alpha = 0.8f)
                                    } else {
                                        AcerolaExtendedTheme.colors.mutedForeground
                                    },
                            ),
                        maxLines = 1,
                    )

                    Spacer(modifier = Modifier.height(if (isLandscape) SpacingTokens.Medium else SpacingTokens.Large))

                    Acerola.Component.Button(
                        text = stringResource(R.string.label_banner_continue),
                        onClick = onContinueClick,
                        icon = Icons.Filled.PlayArrow,
                        modifier = Modifier.fillMaxWidth().height(48.dp),
                    )
                    if (!isLandscape) {
                        Spacer(modifier = Modifier.height(16.dp))
                    }
                }
            }

            if (!isLandscape) {
                Box(
                    modifier =
                        Modifier
                            .align(Alignment.BottomCenter)
                            .padding(bottom = 12.dp)
                            .clip(AcerolaShapes.Full)
                            .background(
                                if (isExpanded && bannerUri != null) {
                                    AcerolaColors.OnScrim.copy(alpha = 0.4f)
                                } else {
                                    AcerolaExtendedTheme.colors.mutedForeground.copy(alpha = 0.4f)
                                },
                            ).size(width = 36.dp, height = 4.dp),
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HomeContinueBannerPreview() {
    AcerolaTheme {
        Main.Home.Component.HomeContinueBanner(
            comic =
                ComicDto(
                    directory =
                        ComicDirectoryDto(
                            id = 1L,
                            name = "Sample Comic",
                            path = "/path",
                            coverUri = null,
                            bannerUri = null,
                            lastModified = 0L,
                            archiveTemplateFk = null,
                        ),
                    category = null,
                    remoteInfo = null,
                ),
            history =
                ReadingHistoryDto(
                    comicDirectoryId = 1L,
                    chapterArchiveId = 10L,
                    chapterSort = "0001",
                    lastPage = 5,
                    isCompleted = false,
                    updatedAt = 123456L,
                ),
            isExpanded = true,
            onExpandedChange = {},
            onContinueClick = {},
            onComicClick = {},
        )
    }
}
