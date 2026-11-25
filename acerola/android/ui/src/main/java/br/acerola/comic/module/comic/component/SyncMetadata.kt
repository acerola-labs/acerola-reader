package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Description
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.RoundIcon
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SyncIconButton
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.pattern.metadata.MetadataSource
import br.acerola.comic.ui.R

/**
 * MangaDex and AniList rows of the metadata section (shown while external sync is on), the
 * desktop's rows: logo, title, description and a round sync button; the source that filled this comic
 * gets a "Current source" badge.
 */
@Composable
fun Comic.Component.ExternalMetadataSync(
    remoteInfo: ComicMetadataDto?,
    onSyncMangadexInfo: () -> Unit,
    onSyncAnilistInfo: () -> Unit,
    mangadexInfoState: SyncActionVisualState = SyncActionVisualState.IDLE,
    anilistInfoState: SyncActionVisualState = SyncActionVisualState.IDLE,
    modifier: Modifier = Modifier,
) {
    val syncSource = remoteInfo?.syncSource
    val anyLoading = mangadexInfoState == SyncActionVisualState.LOADING || anilistInfoState == SyncActionVisualState.LOADING

    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
        SourceRow(
            title = stringResource(id = R.string.title_config_sync_mangadex),
            description = stringResource(id = R.string.description_config_sync_mangadex),
            logo = painterResource(id = R.drawable.mangadex_v2),
            isActive = syncSource == MetadataSource.MANGADEX,
            state = mangadexInfoState,
            enabled = !anyLoading,
            onSync = onSyncMangadexInfo,
        )
        SourceRow(
            title = stringResource(id = R.string.title_config_sync_anilist),
            description = stringResource(id = R.string.description_config_sync_anilist),
            logo = painterResource(id = R.drawable.anilist),
            isActive = syncSource == MetadataSource.ANILIST,
            state = anilistInfoState,
            enabled = !anyLoading,
            onSync = onSyncAnilistInfo,
        )
    }
}

/** ComicInfo.xml row: reads the metadata from the file in the comic's own folder. */
@Composable
fun Comic.Component.ComicInfoSync(
    remoteInfo: ComicMetadataDto?,
    onSyncComicInfo: () -> Unit,
    comicInfoState: SyncActionVisualState = SyncActionVisualState.IDLE,
) {
    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_sync_comic_info),
        description = stringResource(id = R.string.description_sync_comic_info),
        leading = { Acerola.Component.RoundIcon(icon = Icons.Rounded.Description) },
        control = {
            Acerola.Component.SyncIconButton(
                state = comicInfoState,
                text = stringResource(id = R.string.action_comic_preferences_read),
                loadingText = stringResource(id = R.string.action_comic_preferences_syncing),
                onClick = onSyncComicInfo,
                enabled = comicInfoState != SyncActionVisualState.LOADING,
            )
        },
        content = activeSourceBadge(isActive = remoteInfo?.syncSource == MetadataSource.COMIC_INFO),
    )
}

@Composable
private fun SourceRow(
    title: String,
    description: String,
    logo: Painter,
    isActive: Boolean,
    state: SyncActionVisualState,
    enabled: Boolean,
    onSync: () -> Unit,
) {
    Acerola.Component.SettingRow(
        title = title,
        description = description,
        leading = { Image(painter = logo, contentDescription = null, modifier = Modifier.size(40.dp)) },
        control = {
            Acerola.Component.SyncIconButton(
                state = state,
                text = stringResource(id = R.string.action_comic_preferences_sync),
                loadingText = stringResource(id = R.string.action_comic_preferences_syncing),
                onClick = onSync,
                enabled = enabled,
            )
        },
        content = activeSourceBadge(isActive = isActive),
    )
}

// The "Current source" badge under the row when this source filled the comic; nothing otherwise.
private fun activeSourceBadge(isActive: Boolean): (@Composable ColumnScope.() -> Unit)? =
    if (isActive) {
        { Acerola.Component.Badge(text = stringResource(id = R.string.label_comic_metadata_source_active), tone = Tone.Success) }
    } else {
        null
    }

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncMetadataPreview() {
    AcerolaTheme {
        Comic.Component.ExternalMetadataSync(
            remoteInfo = null,
            onSyncMangadexInfo = {},
            onSyncAnilistInfo = {},
        )
    }
}
