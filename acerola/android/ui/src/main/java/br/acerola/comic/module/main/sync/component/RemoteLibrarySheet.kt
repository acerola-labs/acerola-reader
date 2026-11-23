package br.acerola.comic.module.main.sync.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoStories
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.RoundIcon
import br.acerola.comic.common.ux.component.SheetHeader
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.component.colors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.module.main.Main
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.ui.R
import coil.compose.AsyncImage

/**
 * Lists a peer's library (name + chapter count only, via `acerola/browse-library/1`) and lets
 * the user pick a comic to pull with [br.acerola.comic.module.main.sync.state.SyncAction.SyncComic]
 * — the "pull" half of individual comic sync, complementing the push entry points on Home and
 * Comic Detail. Same states as the desktop's remote library dialog: skeleton rows while loading,
 * a `destructive-soft` notice on error, the empty state, then one surface row per comic.
 */
@Composable
fun Main.Sync.Component.RemoteLibrarySheet(
    peerDisplayName: String,
    comics: List<ComicSummary>,
    isLoading: Boolean,
    errorMessage: String?,
    errorType: SyncProtocolError? = null,
    onSelectComic: (comicName: String) -> Unit,
    onDismiss: () -> Unit,
    /** Local path (`file://...`) of the cover already downloaded via `acerola/browse-cover/1`, if
     *  any — `null` shows the default icon instead of an image. */
    coverPathFor: (comicName: String) -> String? = { null },
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        isScrollable = false,
    ) {
        Acerola.Component.SheetHeader(title = stringResource(id = R.string.title_sync_remote_library, peerDisplayName))

        val listPadding = PaddingValues(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small)
        when {
            isLoading -> {
                val loadingLabel = stringResource(id = R.string.label_sync_remote_library_loading)
                Column(
                    modifier =
                        Modifier
                            .padding(listPadding)
                            .semantics { contentDescription = loadingLabel },
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                ) {
                    repeat(4) {
                        Box(
                            modifier =
                                Modifier
                                    .fillMaxWidth()
                                    .height(72.dp)
                                    .clip(AcerolaShapes.Tile)
                                    .background(AcerolaExtendedTheme.colors.surface),
                        )
                    }
                }
            }

            errorMessage != null -> {
                val tone = Tone.Destructive.colors()
                Row(
                    modifier =
                        Modifier
                            .padding(listPadding)
                            .fillMaxWidth()
                            .clip(AcerolaShapes.Card)
                            .background(tone.container)
                            .padding(SpacingTokens.ExtraLarge),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
                ) {
                    Acerola.Component.RoundIcon(icon = Icons.Rounded.CloudOff, tone = Tone.Destructive)
                    Text(
                        text = errorType?.uiMessage?.asString() ?: stringResource(id = R.string.error_sync_remote_library_failed, errorMessage),
                        style = MaterialTheme.typography.bodyMedium,
                        color = tone.content,
                    )
                }
            }

            comics.isEmpty() ->
                Acerola.Component.EmptyState(
                    title = stringResource(id = R.string.label_sync_remote_library_empty),
                    icon = Icons.Rounded.AutoStories,
                    modifier = Modifier.fillMaxWidth().padding(horizontal = SpacingTokens.Huge),
                )

            else ->
                LazyColumn(
                    contentPadding = listPadding,
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                ) {
                    items(items = comics, key = { it.comicName }) { comic ->
                        val coverPath = coverPathFor(comic.comicName)
                        Acerola.Component.HeroButton(
                            title = comic.comicName,
                            description = stringResource(id = R.string.label_sync_remote_library_chapter_count, comic.chapterCount),
                            onClick = { onSelectComic(comic.comicName) },
                            icon = {
                                if (coverPath != null) {
                                    AsyncImage(
                                        model = coverPath,
                                        contentDescription = null,
                                        contentScale = ContentScale.Crop,
                                        modifier =
                                            Modifier
                                                .width(40.dp)
                                                .height(56.dp)
                                                .clip(AcerolaShapes.Inner)
                                                .background(AcerolaExtendedTheme.colors.card),
                                    )
                                } else {
                                    Acerola.Component.RoundIcon(icon = Icons.Rounded.AutoStories)
                                }
                            },
                        )
                    }
                }
        }

        Spacer(modifier = Modifier.navigationBarsPadding().padding(bottom = SpacingTokens.Large).size(0.dp))
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun RemoteLibrarySheetPreview() {
    AcerolaTheme {
        Main.Sync.Component.RemoteLibrarySheet(
            peerDisplayName = "Pixel 8",
            comics =
                listOf(
                    ComicSummary(comicName = "Berserk", chapterCount = 42, coverVersion = 0),
                    ComicSummary(comicName = "Solo Leveling", chapterCount = 12, coverVersion = 0),
                ),
            isLoading = false,
            errorMessage = null,
            onSelectComic = {},
            onDismiss = {},
        )
    }
}
