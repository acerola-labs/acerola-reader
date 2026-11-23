package br.acerola.comic.module.main.common.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Devices
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.AdaptiveSheet
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.HeroButton
import br.acerola.comic.common.ux.component.SheetHeader
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.ui.R
import br.acerola.comic.util.p2p.PairingCode

/**
 * Bottom sheet that lists the currently paired peers, used by any entry point that fires
 * `SyncComicWithPeerUseCase` (Home's [ComicActionsSheet] and the Comic Detail's preferences) — the
 * underlying `acerola/sync-comic/1` session is the same either way, only who asked for it differs.
 * Same look as the desktop's peer picker: one surface row per device, the empty state otherwise.
 */
@Composable
fun Main.Common.Component.PeerPickerSheet(
    peers: List<PairedPeer>,
    onSelect: (peerId: String) -> Unit,
    onDismiss: () -> Unit,
) {
    Acerola.Component.AdaptiveSheet(
        onDismissRequest = onDismiss,
        isScrollable = false,
    ) {
        Acerola.Component.SheetHeader(title = stringResource(id = R.string.title_sync_peer_picker))

        if (peers.isEmpty()) {
            Acerola.Component.EmptyState(
                title = stringResource(id = R.string.label_sync_no_peers_title),
                hint = stringResource(id = R.string.label_sync_no_peers),
                icon = Icons.Rounded.Devices,
                modifier = Modifier.fillMaxWidth().padding(horizontal = SpacingTokens.Huge),
            )
        } else {
            LazyColumn(
                contentPadding = PaddingValues(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
                verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                items(items = peers, key = { it.peerId }) { peer ->
                    Acerola.Component.HeroButton(
                        title = peer.nickname ?: peer.deviceName ?: PairingCode.shortId(peer.peerId),
                        icon = Icons.Rounded.PhoneAndroid,
                        onClick = { onSelect(peer.peerId) },
                    )
                }
            }
        }

        Spacer(modifier = Modifier.navigationBarsPadding().padding(bottom = SpacingTokens.Large))
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PeerPickerSheetPreview() {
    AcerolaTheme {
        Main.Common.Component.PeerPickerSheet(
            peers =
                listOf(
                    PairedPeer(peerId = "peer-1", deviceName = "Pixel 8"),
                    PairedPeer(peerId = "peer-2", deviceName = null),
                ),
            onSelect = {},
            onDismiss = {},
        )
    }
}
