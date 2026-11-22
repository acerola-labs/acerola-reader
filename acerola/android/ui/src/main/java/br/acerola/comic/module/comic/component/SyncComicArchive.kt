package br.acerola.comic.module.comic.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoAwesomeMotion
import androidx.compose.material.icons.filled.Collections
import androidx.compose.material.icons.filled.ImageSearch
import androidx.compose.material.icons.filled.SyncAlt
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SyncButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.ui.R

/** File sync section rows: sync the local chapters, and find the cover/banner already in the folder. */
@Composable
fun Comic.Component.FileSyncActions(
    onSyncChapters: () -> Unit,
    onRescanCover: () -> Unit,
    syncChaptersState: SyncActionVisualState = SyncActionVisualState.IDLE,
    rescanCoverState: SyncActionVisualState = SyncActionVisualState.IDLE,
    enabled: Boolean = true,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
        ActionRow(
            title = stringResource(id = R.string.title_sync_chapters),
            description = stringResource(id = R.string.description_sync_chapters_local),
            icon = Icons.Default.SyncAlt,
            action = stringResource(id = R.string.action_comic_preferences_sync),
            state = syncChaptersState,
            enabled = enabled,
            onClick = onSyncChapters,
        )
        ActionRow(
            title = stringResource(id = R.string.title_sync_cover_banner),
            description = stringResource(id = R.string.description_sync_cover_banner),
            icon = Icons.Default.Collections,
            action = stringResource(id = R.string.action_comic_preferences_find),
            state = rescanCoverState,
            enabled = enabled,
            onClick = onRescanCover,
        )
    }
}

/** Cover section rows: use the first page as cover, and (volume covers view) generate volume covers. */
@Composable
fun Comic.Component.CoverActions(
    onExtractFirstPageAsCover: () -> Unit,
    onExtractVolumeCovers: (() -> Unit)? = null,
    extractFirstPageState: SyncActionVisualState = SyncActionVisualState.IDLE,
    extractVolumeCoversState: SyncActionVisualState = SyncActionVisualState.IDLE,
    enabled: Boolean = true,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
        ActionRow(
            title = stringResource(id = R.string.title_extract_first_page_as_cover),
            description = stringResource(id = R.string.description_extract_first_page_as_cover),
            icon = Icons.Default.ImageSearch,
            action = stringResource(id = R.string.action_comic_preferences_generate),
            state = extractFirstPageState,
            enabled = enabled,
            onClick = onExtractFirstPageAsCover,
        )
        if (onExtractVolumeCovers != null) {
            ActionRow(
                title = stringResource(id = R.string.title_extract_volume_covers),
                description = stringResource(id = R.string.description_extract_volume_covers),
                icon = Icons.Default.AutoAwesomeMotion,
                action = stringResource(id = R.string.action_comic_preferences_generate),
                state = extractVolumeCoversState,
                enabled = enabled,
                onClick = onExtractVolumeCovers,
            )
        }
    }
}

@Composable
private fun ActionRow(
    title: String,
    description: String,
    icon: ImageVector,
    action: String,
    state: SyncActionVisualState,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Acerola.Component.SettingRow(
        title = title,
        description = description,
        icon = icon,
        content = {
            Acerola.Component.SyncButton(
                state = state,
                text = action,
                loadingText = stringResource(id = R.string.action_comic_preferences_syncing),
                onClick = onClick,
                enabled = enabled,
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FileSyncActionsPreview() {
    AcerolaTheme {
        Comic.Component.FileSyncActions(onSyncChapters = {}, onRescanCover = {})
    }
}
