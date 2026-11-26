package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.FolderCopy
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material.icons.rounded.Sync
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.SettingAction
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.SyncLibraryArchive(
    onDeepScan: () -> Unit,
    onQuickSync: () -> Unit,
    deepScanState: SyncActionVisualState = SyncActionVisualState.IDLE,
    quickSyncState: SyncActionVisualState = SyncActionVisualState.IDLE,
    modifier: Modifier = Modifier,
) {
    val anyLoading = deepScanState == SyncActionVisualState.LOADING || quickSyncState == SyncActionVisualState.LOADING
    val syncingText = stringResource(id = R.string.action_config_syncing)

    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
        Acerola.Component.SettingRow(
            title = stringResource(id = R.string.description_text_home_quick_sync),
            description = stringResource(id = R.string.description_text_home_quick_sync_supporting),
            icon = Icons.Default.FolderCopy,
            actions =
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_config_sync),
                        icon = Icons.Rounded.Sync,
                        onClick = onQuickSync,
                        enabled = !anyLoading,
                        state = quickSyncState,
                        loadingText = syncingText,
                    ),
                ),
        )
        Acerola.Component.SettingRow(
            title = stringResource(id = R.string.description_text_home_deep_sync),
            description = stringResource(id = R.string.description_text_home_deep_sync_supporting),
            icon = Icons.Default.Storage,
            actions =
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_config_rebuild),
                        icon = Icons.Rounded.Sync,
                        onClick = onDeepScan,
                        enabled = !anyLoading,
                        state = deepScanState,
                        loadingText = syncingText,
                    ),
                ),
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncLibraryArchivePreview() {
    AcerolaTheme {
        Main.Config.Component.SyncLibraryArchive(
            onDeepScan = {},
            onQuickSync = {},
        )
    }
}
