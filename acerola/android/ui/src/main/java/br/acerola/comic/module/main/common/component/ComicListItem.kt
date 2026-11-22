package br.acerola.comic.module.main.common.component

import android.content.res.Configuration
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.rounded.AutoStories
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ActionIcon
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.BookmarkRibbon
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.component.ImageCard
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.archive.ComicDirectoryDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.pattern.metadata.MetadataSource
import br.acerola.comic.ui.R
import coil.compose.rememberAsyncImagePainter
import coil.request.ImageRequest
import coil.size.Size
import coil.size.SizeResolver

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Main.Common.Component.ComicListItem(
    comic: ComicDto,
    subtitle: String? = null,
    chapterCount: Int = 0,
    isCompleted: Boolean = false,
    conflictCount: Int = 0,
    isSelected: Boolean = false,
    isSelectionMode: Boolean = false,
    onLongClick: (() -> Unit)? = null,
    onPlayClick: (() -> Unit)? = null,
    onShowActions: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    val context = LocalContext.current
    val density = LocalDensity.current

    val coverUri = comic.directory.coverUri ?: comic.directory.bannerUri
    val title = comic.remoteInfo?.title ?: comic.directory.name

    val imageSize = with(receiver = density) { Size(width = 80.dp.toPx().toInt(), height = SizeTokens.ComicGridMinSize.toPx().toInt()) }

    val placeholderPainter =
        rememberAsyncImagePainter(
            model =
                ImageRequest
                    .Builder(context)
                    .data(data = R.raw.placeholder_comic)
                    .size(resolver = SizeResolver(imageSize))
                    .build(),
        )

    val coverPainter =
        rememberAsyncImagePainter(
            placeholder = placeholderPainter,
            fallback = placeholderPainter,
            error = placeholderPainter,
            model =
                ImageRequest
                    .Builder(context)
                    .data(data = coverUri)
                    .memoryCacheKey("${coverUri}_${comic.directory.lastModified}")
                    .diskCacheKey("${coverUri}_${comic.directory.lastModified}")
                    .size(resolver = SizeResolver(imageSize))
                    .build(),
        )

    val categoryColor = comic.category?.color
    val score =
        comic.remoteInfo
            ?.sources
            ?.anilist
            ?.averageScore
            ?.let { it / 10f }

    // Brand list row: `rounded-tile bg-surface`, cover on the left, info, then the actions.
    ProvideSurfaceLevel(AcerolaSurfaceLevel.Surface) {
        Row(
            modifier =
                modifier
                    .fillMaxWidth()
                    .height(height = 128.dp)
                    .clip(AcerolaShapes.Tile)
                    .background(AcerolaExtendedTheme.colors.surface)
                    .combinedClickable(onClick = onClick, onLongClick = onLongClick)
                    .padding(all = SpacingTokens.Small),
        ) {
            // Cover Box (72dp width)
            Box(
                modifier =
                    Modifier
                        .width(width = 72.dp)
                        .fillMaxHeight(),
            ) {
                Acerola.Component.ImageCard(
                    onClick = onClick,
                    onLongClick = onLongClick,
                    image = coverPainter,
                    shape = AcerolaShapes.Inner,
                    modifier = Modifier.fillMaxSize(),
                )

                // Scrim only at the bottom for the source logo visibility
                Box(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .height(SpacingTokens.Giant)
                            .align(Alignment.BottomCenter)
                            .clip(AcerolaShapes.Inner.copy(topStart = CornerSize(0.dp), topEnd = CornerSize(0.dp)))
                            .background(
                                Brush.verticalGradient(
                                    colors = listOf(Color.Transparent, AcerolaColors.Scrim.copy(alpha = 0.5f)),
                                ),
                            ),
                )

                // Bottom Right: Source Logo Overlay on Image
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .padding(bottom = SpacingTokens.ExtraSmall, end = SpacingTokens.ExtraSmall),
                    contentAlignment = Alignment.BottomEnd,
                ) {
                    val sourceIcon =
                        when (comic.remoteInfo?.syncSource) {
                            MetadataSource.MANGADEX -> R.drawable.mangadex_v2
                            MetadataSource.ANILIST -> R.drawable.anilist
                            else -> null
                        }
                    if (sourceIcon != null) {
                        Icon(
                            painter = painterResource(id = sourceIcon),
                            contentDescription = null,
                            tint = Color.Unspecified,
                            modifier = Modifier.size(SizeTokens.IconExtraSmall),
                        )
                    }
                }

                if (categoryColor != null) {
                    BookmarkRibbon(
                        color = Color(categoryColor),
                        modifier =
                            Modifier
                                .align(Alignment.TopStart)
                                .padding(start = SpacingTokens.Small)
                                .width(SpacingTokens.Medium)
                                .height(SpacingTokens.ExtraLarge),
                    )
                }

                if (comic.directory.hidden) {
                    Icon(
                        imageVector = Icons.Rounded.VisibilityOff,
                        contentDescription = null,
                        tint = AcerolaColors.OnScrim,
                        modifier =
                            Modifier
                                .align(Alignment.BottomStart)
                                .padding(bottom = SpacingTokens.ExtraSmall, start = SpacingTokens.ExtraSmall)
                                .size(SizeTokens.IconExtraSmall)
                                .background(
                                    color = AcerolaColors.Scrim.copy(alpha = 0.5f),
                                    shape = AcerolaShapes.Inner,
                                ).padding(2.dp),
                    )
                }

                if (isSelectionMode) {
                    if (isSelected) {
                        Box(
                            modifier =
                                Modifier
                                    .fillMaxSize()
                                    .clip(AcerolaShapes.Inner)
                                    .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.3f)),
                            contentAlignment = Alignment.Center,
                        ) {
                            Box(
                                modifier =
                                    Modifier
                                        .size(38.dp)
                                        .background(
                                            color = MaterialTheme.colorScheme.primary,
                                            shape = CircleShape,
                                        ),
                                contentAlignment = Alignment.Center,
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Check,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onPrimary,
                                    modifier = Modifier.size(24.dp),
                                )
                            }
                        }
                    } else {
                        Box(
                            modifier =
                                Modifier
                                    .fillMaxSize()
                                    .clip(AcerolaShapes.Inner)
                                    .background(AcerolaColors.Scrim.copy(alpha = 0.5f)),
                            contentAlignment = Alignment.Center,
                        ) {
                            Box(
                                modifier =
                                    Modifier
                                        .size(32.dp)
                                        .background(color = AcerolaColors.OnScrim.copy(alpha = 0.2f), shape = CircleShape),
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.width(width = SpacingTokens.Medium))

            // Info Column
            Column(
                modifier =
                    Modifier
                        .fillMaxHeight()
                        .weight(weight = 1f),
                verticalArrangement = Arrangement.Center,
            ) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    color = AcerolaExtendedTheme.colors.foreground,
                )

                if (subtitle != null) {
                    Text(
                        text = subtitle,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        color = AcerolaExtendedTheme.colors.mutedForeground,
                    )
                }

                Spacer(modifier = Modifier.height(SpacingTokens.Small))

                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                    // Completed status
                    if (isCompleted) {
                        Acerola.Component.Badge(text = stringResource(id = R.string.label_comic_status_read), tone = Tone.Success)
                    }

                    // Rating
                    if (score != null) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                            Icon(
                                imageVector = Icons.Rounded.Star,
                                contentDescription = null,
                                tint = AcerolaExtendedTheme.colors.warning,
                                modifier = Modifier.size(SpacingTokens.MediumLarge),
                            )
                            Text(
                                text = score.toString(),
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                                color = AcerolaExtendedTheme.colors.mutedForeground,
                            )
                        }
                    }

                    // Chapter Count
                    if (chapterCount > 0) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                            Icon(
                                imageVector = Icons.Rounded.AutoStories,
                                contentDescription = null,
                                tint = AcerolaExtendedTheme.colors.mutedForeground,
                                modifier = Modifier.size(SpacingTokens.MediumLarge),
                            )
                            Text(
                                text = chapterCount.toString(),
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                                color = AcerolaExtendedTheme.colors.mutedForeground,
                            )
                        }
                    }

                    // Pending sync conflicts — same recipe as Rating/Chapter Count (icon +
                    // text with no filled chip): `error` alone over the row background already passes
                    // in every theme (verified), a background behind the icon didn't pass in
                    // Catppuccin Light.
                    if (conflictCount > 0) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                            Icon(
                                imageVector = Icons.Default.Warning,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.error,
                                modifier = Modifier.size(SpacingTokens.MediumLarge),
                            )
                            Text(
                                text = stringResource(id = R.string.label_comic_status_conflict),
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                                color = AcerolaExtendedTheme.colors.mutedForeground,
                            )
                        }
                    }
                }
            }

            if (onPlayClick != null) {
                Acerola.Component.IconButton(
                    icon = Icons.Default.PlayArrow,
                    contentDescription = stringResource(id = R.string.description_icon_continue_reading),
                    onClick = onPlayClick,
                    variant = ButtonVariant.Secondary,
                    modifier = Modifier.align(Alignment.CenterVertically).padding(end = SpacingTokens.Small),
                )
            }

            if (onShowActions != null) {
                Acerola.Component.ActionIcon(
                    icon = Icons.Default.MoreVert,
                    onClick = onShowActions,
                    contentDescription = stringResource(id = R.string.description_icon_chapter_more_options),
                    modifier = Modifier.align(Alignment.CenterVertically),
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicListItemPreview() {
    AcerolaTheme {
        Main.Common.Component.ComicListItem(
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
            onClick = {},
        )
    }
}

@Preview(name = "Conflict - Light", showBackground = true)
@Preview(name = "Conflict - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ComicListItemConflictPreview() {
    AcerolaTheme {
        Main.Common.Component.ComicListItem(
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
            chapterCount = 170,
            conflictCount = 2,
            onClick = {},
        )
    }
}
