package br.acerola.comic.module.comic.template

import android.content.res.Configuration
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.BookmarkRibbon
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.modifier.lift
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.dto.history.ReadingHistoryDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.pattern.metadata.ComicStatus
import br.acerola.comic.ui.R
import coil.compose.AsyncImage
import coil.compose.rememberAsyncImagePainter
import coil.request.ImageRequest

@Composable
fun Comic.Template.Header(
    comic: ComicDto,
    history: ReadingHistoryDto?,
    onContinueClick: (Long?, Int) -> Unit,
) {
    val scrollState = rememberScrollState()

    var isExpanded by remember { mutableStateOf(value = false) }

    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(color = MaterialTheme.colorScheme.background),
    ) {
        Box(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .height(height = SizeTokens.ComicHeaderHeight),
        ) {
            val context = LocalContext.current
            val bannerModel = comic.directory.bannerUri ?: comic.directory.coverUri

            val placeholderPainter =
                rememberAsyncImagePainter(
                    model =
                        ImageRequest
                            .Builder(context)
                            .data(data = R.raw.placeholder_comic)
                            .build(),
                )

            AsyncImage(
                contentDescription = null,
                contentScale = ContentScale.Crop,
                model =
                    ImageRequest
                        .Builder(context = context)
                        .data(data = bannerModel)
                        .memoryCacheKey("${bannerModel}_${comic.directory.lastModified}")
                        .diskCacheKey("${bannerModel}_${comic.directory.lastModified}")
                        .crossfade(enable = true)
                        .build(),
                placeholder = placeholderPainter,
                error = placeholderPainter,
                fallback = placeholderPainter,
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .height(height = SizeTokens.ComicHeaderBannerHeight)
                        .align(Alignment.TopCenter),
            )

            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .height(height = SizeTokens.ComicHeaderBannerHeight)
                        .background(
                            Brush.verticalGradient(colors = listOf(Color.Transparent, MaterialTheme.colorScheme.background)),
                        ),
            )

            Column(
                modifier =
                    Modifier
                        .align(Alignment.BottomCenter)
                        .padding(horizontal = SpacingTokens.ExtraLarge),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.Bottom,
                ) {
                    Box {
                        AsyncImage(
                            contentDescription = stringResource(id = R.string.comic_header_cover_description),
                            contentScale = ContentScale.Crop,
                            model =
                                ImageRequest
                                    .Builder(context = context)
                                    .data(data = comic.directory.coverUri)
                                    .memoryCacheKey("${comic.directory.coverUri}_${comic.directory.lastModified}")
                                    .diskCacheKey("${comic.directory.coverUri}_${comic.directory.lastModified}")
                                    .crossfade(enable = true)
                                    .build(),
                            placeholder = placeholderPainter,
                            error = placeholderPainter,
                            fallback = placeholderPainter,
                            modifier =
                                Modifier
                                    .padding(top = SpacingTokens.ExtraSmall)
                                    .width(width = SizeTokens.ComicHeaderCoverWidth)
                                    .height(height = SizeTokens.ComicHeaderCoverHeight)
                                    .clip(shape = AcerolaShapes.Tile)
                                    .background(color = AcerolaExtendedTheme.colors.surface),
                        )

                        val categoryColor = comic.category?.color
                        if (categoryColor != null) {
                            BookmarkRibbon(
                                color = Color(categoryColor),
                                modifier =
                                    Modifier
                                        .align(Alignment.TopStart)
                                        .padding(start = SpacingTokens.Medium)
                                        .width(SpacingTokens.ExtraLarge)
                                        .height(SpacingTokens.Giant),
                            )
                        }

                        if (comic.directory.hidden) {
                            Icon(
                                imageVector = Icons.Rounded.VisibilityOff,
                                contentDescription = stringResource(id = R.string.description_icon_comic_hidden),
                                tint = AcerolaColors.OnScrim,
                                modifier =
                                    Modifier
                                        .align(Alignment.BottomStart)
                                        .padding(SpacingTokens.Small)
                                        .size(SizeTokens.IconSmall)
                                        .background(
                                            color = AcerolaColors.Scrim.copy(alpha = 0.5f),
                                            shape = AcerolaShapes.Inner,
                                        ).padding(2.dp),
                            )
                        }
                    }

                    Spacer(modifier = Modifier.width(width = SpacingTokens.Large))

                    Column(
                        verticalArrangement = Arrangement.Bottom,
                        modifier =
                            Modifier
                                .height(height = SizeTokens.ComicHeaderInfoHeight)
                                .weight(weight = 1f),
                    ) {
                        Text(
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                            text = comic.remoteInfo?.title ?: comic.directory.name,
                            style =
                                MaterialTheme.typography.headlineMedium.copy(color = AcerolaExtendedTheme.colors.foreground),
                        )

                        Text(
                            color = AcerolaExtendedTheme.colors.mutedForeground,
                            style = MaterialTheme.typography.bodyLarge,
                            text = comic.remoteInfo?.authors?.name ?: stringResource(id = R.string.comic_header_unknown),
                        )

                        Spacer(modifier = Modifier.height(height = SpacingTokens.Small))

                        Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                            val status = ComicStatus.fromRawValue(comic.remoteInfo?.status)
                            Acerola.Component.Badge(text = stringResource(id = status.stringRes), tone = Tone.Primary)
                            comic.remoteInfo?.syncSource?.let { source ->
                                Acerola.Component.Badge(text = source.displayName)
                            }
                        }
                    }
                }
            }
        }

        Row(
            horizontalArrangement = Arrangement.spacedBy(space = SpacingTokens.Small),
            modifier =
                Modifier
                    .fillMaxWidth()
                    .horizontalScroll(scrollState)
                    .padding(
                        horizontal = SpacingTokens.ExtraLarge,
                        vertical = SpacingTokens.Large,
                    ),
        ) {
            comic.remoteInfo?.genre?.forEach { genre ->
                Acerola.Component.Badge(text = genre.name)
            }
        }

        // Synopsis card (`rounded-card bg-card p-5`), tap to unfold.
        Surface(
            shape = AcerolaShapes.Card,
            color = AcerolaExtendedTheme.colors.card,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.Large)
                    .lift(AcerolaShapes.Card)
                    .animateContentSize()
                    .clickable {
                        isExpanded = !isExpanded
                    },
        ) {
            Column(
                modifier = Modifier.padding(SpacingTokens.ExtraLarge),
            ) {
                Text(
                    text = stringResource(id = R.string.comic_header_synopsis_title),
                    color = AcerolaExtendedTheme.colors.foreground,
                    style = MaterialTheme.typography.titleMedium,
                )

                Spacer(modifier = Modifier.height(height = SpacingTokens.Small))

                Text(
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = if (isExpanded) Int.MAX_VALUE else 3,
                    color = AcerolaExtendedTheme.colors.mutedForeground,
                    lineHeight = MaterialTheme.typography.bodyLarge.lineHeight,
                    text = comic.remoteInfo?.description ?: stringResource(id = R.string.comic_header_no_description),
                )

                Spacer(modifier = Modifier.height(height = SpacingTokens.Small))

                Icon(
                    imageVector = if (isExpanded) Icons.Default.KeyboardArrowUp else Icons.Default.KeyboardArrowDown,
                    contentDescription = null,
                    tint = AcerolaExtendedTheme.colors.mutedForeground,
                    modifier = Modifier.align(Alignment.CenterHorizontally),
                )
            }
        }

        Spacer(modifier = Modifier.height(height = SpacingTokens.Medium))

        Acerola.Component.Button(
            modifier = Modifier.fillMaxWidth().padding(horizontal = SpacingTokens.Large),
            icon = Icons.Default.PlayArrow,
            onClick = {
                if (history != null) {
                    onContinueClick(history.chapterArchiveId, history.lastPage)
                } else {
                    onContinueClick(-1L, 0)
                }
            },
            text =
                when {
                    history != null -> stringResource(id = R.string.label_comic_action_continue)
                    else -> stringResource(id = R.string.label_comic_action_start)
                },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HeaderPreview() {
    AcerolaTheme {
        Comic.Template.Header(
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
            history = null,
            onContinueClick = { _, _ -> },
        )
    }
}
