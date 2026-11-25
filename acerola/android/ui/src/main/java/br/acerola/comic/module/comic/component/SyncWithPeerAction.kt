package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudDownload
import androidx.compose.material.icons.rounded.CloudUpload
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SyncIconButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

/**
 * Single entry point for `acerola/sync-comic/1`, shared by the Comic Detail's preferences and
 * (via [br.acerola.comic.module.main.common.component.PeerPickerSheet]) the Home's
 * `ComicActionsSheet`. Two explicit actions — push (send this comic to a device) and pull
 * (fetch it from a device) — each open the peer picker; the actual `syncComic` call only fires
 * once a peer is chosen, with the direction already decided by which button started the flow.
 */
@Composable
fun Comic.Component.SyncWithPeerAction(
    state: SyncActionVisualState,
    onPush: () -> Unit,
    onPull: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val loading = state == SyncActionVisualState.LOADING

    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_sync_comic_send),
        description = stringResource(id = R.string.description_sync_comic_with_peer),
        icon = Icons.Rounded.PhoneAndroid,
        modifier = modifier,
        control = {
            Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                val pushText = stringResource(id = R.string.action_sync_comic_push)
                Acerola.Component.SyncIconButton(
                    state = state,
                    text = pushText,
                    loadingText = pushText,
                    onClick = onPush,
                    icon = Icons.Rounded.CloudUpload,
                )
                Acerola.Component.IconButton(
                    icon = Icons.Rounded.CloudDownload,
                    contentDescription = stringResource(id = R.string.action_sync_comic_pull),
                    onClick = onPull,
                    enabled = !loading,
                )
            }
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncWithPeerActionPreview() {
    AcerolaTheme {
        Comic.Component.SyncWithPeerAction(
            state = SyncActionVisualState.IDLE,
            onPush = {},
            onPull = {},
        )
    }
}
