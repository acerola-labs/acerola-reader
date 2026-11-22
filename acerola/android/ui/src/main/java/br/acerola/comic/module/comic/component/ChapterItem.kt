package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.MenuBook
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.BookmarkRemove
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.MenuBook
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.RadioButtonUnchecked
import androidx.compose.material.icons.filled.Warning
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ActionIcon
import br.acerola.comic.common.ux.component.ActionListItem
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.RoundIcon
import br.acerola.comic.common.ux.component.SyncActionIcon
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.archive.ChapterFileDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Comic.Component.ChapterItem(
    chapterFileDto: ChapterFileDto,
    modifier: Modifier = Modifier,
    onToggleRead: () -> Unit = {},
    isRead: Boolean = false,
    hasConflict: Boolean = false,
    isSelected: Boolean = false,
    isSelectionMode: Boolean = false,
    onLongClick: () -> Unit = {},
    onClick: () -> Unit,
    onSendToPeer: () -> Unit = {},
    sendState: SyncActionVisualState = SyncActionVisualState.IDLE,
) {
    var showDetails by remember { mutableStateOf(value = false) }

    val mainTitle = stringResource(id = R.string.title_chapter_item_chapter_number, chapterFileDto.chapterSort)
    val subtitle = chapterFileDto.name

    val brand = AcerolaExtendedTheme.colors
    // Desktop chapter row: a `rounded-tile bg-surface` row; selected turns `bg-primary-soft`.
    val rowColor = if (isSelectionMode && isSelected) brand.primarySoft else brand.surface
    // The file name only adds something when it is not just the chapter title again.
    val showSubtitle = subtitle.isNotBlank() && !subtitle.equals(mainTitle, ignoreCase = true)

    ProvideSurfaceLevel(AcerolaSurfaceLevel.Surface) {
        Row(
            modifier =
                modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.ExtraSmall)
                    .clip(AcerolaShapes.Tile)
                    .background(rowColor)
                    .combinedClickable(onClick = onClick, onLongClick = onLongClick)
                    .padding(start = SpacingTokens.Large, end = SpacingTokens.Small, top = SpacingTokens.Medium, bottom = SpacingTokens.Medium),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            when {
                sendState != SyncActionVisualState.IDLE ->
                    Acerola.Component.SyncActionIcon(state = sendState) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.Send,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(SizeTokens.IconSmall),
                        )
                    }
                isSelectionMode && isSelected -> Acerola.Component.RoundIcon(icon = Icons.Default.Check, tone = Tone.Primary)
                isSelectionMode -> Acerola.Component.RoundIcon(icon = Icons.Default.RadioButtonUnchecked)
                hasConflict -> Acerola.Component.RoundIcon(icon = Icons.Default.Warning, tone = Tone.Destructive)
                isRead -> Acerola.Component.RoundIcon(icon = Icons.Default.Check, tone = Tone.Success)
                else -> Acerola.Component.RoundIcon(icon = Icons.AutoMirrored.Filled.MenuBook)
            }

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = mainTitle,
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
                    color = if (isRead) brand.mutedForeground else brand.foreground,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (showSubtitle) {
                    Text(
                        text = subtitle,
                        style = MaterialTheme.typography.bodySmall,
                        color = brand.mutedForeground,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            if (hasConflict) {
                Acerola.Component.Badge(text = stringResource(id = R.string.label_comic_status_conflict), tone = Tone.Destructive)
            }
            if (isRead) {
                Acerola.Component.Badge(text = stringResource(id = R.string.label_comic_status_read), tone = Tone.Success)
            }

            Acerola.Component.ActionIcon(
                icon = Icons.Default.MoreVert,
                onClick = { if (isSelectionMode) onClick() else showDetails = true },
                contentDescription = stringResource(id = R.string.description_icon_chapter_more_options),
            )
        }
    }

    if (showDetails) {
        val lastModifiedText =
            remember(chapterFileDto.lastModified) {
                if (chapterFileDto.lastModified > 0) {
                    SimpleDateFormat("dd/MM/yyyy HH:mm", Locale.getDefault()).format(Date(chapterFileDto.lastModified))
                } else {
                    null
                }
            }

        Acerola.Component.AdaptiveSheet(
            onDismissRequest = { showDetails = false },
        ) {
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = SpacingTokens.Huge, vertical = SpacingTokens.Small),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
            ) {
                when {
                    hasConflict -> Acerola.Component.RoundIcon(icon = Icons.Default.Warning, tone = Tone.Destructive)
                    isRead -> Acerola.Component.RoundIcon(icon = Icons.Default.Check, tone = Tone.Success)
                    else -> Acerola.Component.RoundIcon(icon = Icons.AutoMirrored.Filled.MenuBook)
                }

                Column {
                    Text(
                        text = mainTitle,
                        style = MaterialTheme.typography.titleMedium,
                        color = brand.foreground,
                        maxLines = 2,
                    )
                    val volumeName = chapterFileDto.volumeName
                    if (!volumeName.isNullOrBlank()) {
                        Text(text = volumeName, style = MaterialTheme.typography.bodySmall, color = brand.mutedForeground)
                    }
                }
            }

            Column(
                modifier = Modifier.padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Medium),
                verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
            ) {
                Column(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .clip(AcerolaShapes.Tile)
                            .background(brand.surface)
                            .padding(SpacingTokens.Large),
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                ) {
                    DetailRow(
                        label = stringResource(id = R.string.label_chapter_detail_file),
                        value = chapterFileDto.name,
                    )
                    if (lastModifiedText != null) {
                        DetailRow(
                            label = stringResource(id = R.string.label_chapter_detail_last_modified),
                            value = lastModifiedText,
                            isLast = true,
                        )
                    }
                }

                Acerola.Component.ActionListItem(
                    icon = if (isRead) Icons.Default.BookmarkRemove else Icons.Default.Check,
                    title = stringResource(id = if (isRead) R.string.action_mark_as_unread else R.string.action_mark_as_read),
                    tone = if (isRead) Tone.Neutral else Tone.Success,
                    onClick = {
                        showDetails = false
                        onToggleRead()
                    },
                )
                Acerola.Component.ActionListItem(
                    icon = Icons.AutoMirrored.Filled.Send,
                    title = stringResource(id = R.string.action_send_chapters_to_peer),
                    onClick = {
                        showDetails = false
                        onSendToPeer()
                    },
                )
            }

            Spacer(modifier = Modifier.navigationBarsPadding())
        }
    }
}

@Composable
private fun DetailRow(
    label: String,
    value: String,
    isLast: Boolean = false,
) {
    if (value.isBlank()) return
    Column(modifier = Modifier.padding(bottom = if (isLast) 0.dp else SpacingTokens.ExtraSmall)) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = AcerolaExtendedTheme.colors.mutedForeground,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold),
            color = AcerolaExtendedTheme.colors.foreground,
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 1", path = "/path/1", chapterSort = "0001"),
            onClick = {},
        )
    }
}

@Preview(name = "Read - Light", showBackground = true)
@Preview(name = "Read - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemReadPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 1", path = "/path/1", chapterSort = "0001"),
            isRead = true,
            onClick = {},
        )
    }
}

@Preview(name = "Conflict - Light", showBackground = true)
@Preview(name = "Conflict - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemConflictPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 5 (conflito-peer1)", path = "/path/5", chapterSort = "0005"),
            hasConflict = true,
            onClick = {},
        )
    }
}

@Preview(name = "Sending - Light", showBackground = true)
@Preview(name = "Sending - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ChapterItemSendingPreview() {
    AcerolaTheme {
        Comic.Component.ChapterItem(
            chapterFileDto = ChapterFileDto(id = 1L, name = "Capítulo 1", path = "/path/1", chapterSort = "0001"),
            sendState = SyncActionVisualState.LOADING,
            onClick = {},
        )
    }
}
