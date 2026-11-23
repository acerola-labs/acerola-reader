package br.acerola.comic.module.comic.template

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.LayersClear
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SettingSection
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.module.comic.component.ComicCategorySelector
import br.acerola.comic.module.comic.component.ComicExternalSyncToggle
import br.acerola.comic.module.comic.component.ComicInfoSync
import br.acerola.comic.module.comic.component.CoverActions
import br.acerola.comic.module.comic.component.ExternalMetadataSync
import br.acerola.comic.module.comic.component.FileSyncActions
import br.acerola.comic.module.comic.component.PaginationPreference
import br.acerola.comic.module.comic.component.SyncWithPeerAction
import br.acerola.comic.module.comic.component.VolumeStylePreference
import br.acerola.comic.module.comic.state.ComicAction
import br.acerola.comic.module.comic.state.ComicSyncAction
import br.acerola.comic.module.comic.state.ComicUiState
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.common.component.PeerPickerSheet
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.ui.R

private val sectionModifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp)

// Same layout as the desktop's comic preferences (acerola-comic-preferences.svelte): one brand
// section per subject, all open — reading, metadata, files, devices, cover and the danger zone.
// `expandedCategories`/`onToggleCategory` stay in the signature for `ComicScreen`, which still
// owns that state, but every section is open now.
@Suppress("UNUSED_PARAMETER")
fun Comic.Template.configSection(
    scope: LazyListScope,
    uiState: ComicUiState,
    expandedCategories: Set<String>,
    onToggleCategory: (String) -> Unit,
    getSyncActionVisualState: (ComicSyncAction) -> SyncActionVisualState = { SyncActionVisualState.IDLE },
    pairedPeers: List<PairedPeer> = emptyList(),
    syncWithPeerState: SyncActionVisualState = SyncActionVisualState.IDLE,
    onLoadPairedPeers: () -> Unit = {},
    onAction: (ComicAction) -> Unit,
    onSyncAction: (ComicSyncAction) -> Unit,
) {
    val fileActions =
        listOf(
            ComicSyncAction.SyncChaptersLocal,
            ComicSyncAction.RescanComic,
            ComicSyncAction.ExtractFirstPageAsCover,
            ComicSyncAction.ExtractVolumeCovers,
        )

    scope.item { Spacer(modifier = Modifier.height(8.dp)) }

    scope.item {
        Acerola.Component.SettingSection(
            title = stringResource(id = R.string.title_comic_preferences_reading),
            lead = stringResource(id = R.string.description_comic_preferences_reading),
            modifier = sectionModifier,
        ) {
            if (uiState.hasVolumeStructure) {
                Comic.Component.VolumeStylePreference(
                    selected = uiState.volumeViewMode,
                    onSelect = { mode -> onAction(ComicAction.UpdateVolumeView(mode)) },
                )
            }
            Comic.Component.PaginationPreference(
                selected = uiState.selectedChapterPerPage,
                onSelect = { onAction(ComicAction.UpdatePageSize(it)) },
            )
            Comic.Component.ComicCategorySelector(
                selectedCategory = uiState.comic.category,
                allCategories = uiState.allCategories,
                onUpdateMangaCategory = { id -> onAction(ComicAction.UpdateCategory(id)) },
            )
        }
    }

    scope.item {
        Acerola.Component.SettingSection(
            title = stringResource(id = R.string.title_config_metadata),
            lead = stringResource(id = R.string.description_comic_preferences_metadata),
            modifier = sectionModifier,
        ) {
            val externalSyncEnabled = uiState.comic.directory.externalSyncEnabled
            Comic.Component.ComicExternalSyncToggle(
                enabled = externalSyncEnabled,
                onToggle = { onAction(ComicAction.ToggleExternalSync(it)) },
            )
            if (externalSyncEnabled) {
                Comic.Component.ExternalMetadataSync(
                    remoteInfo = uiState.comic.remoteInfo,
                    onSyncMangadexInfo = { onSyncAction(ComicSyncAction.SyncMangadexInfo) },
                    onSyncAnilistInfo = { onSyncAction(ComicSyncAction.SyncAnilistInfo) },
                    mangadexInfoState = getSyncActionVisualState(ComicSyncAction.SyncMangadexInfo),
                    anilistInfoState = getSyncActionVisualState(ComicSyncAction.SyncAnilistInfo),
                )
            }
            Comic.Component.ComicInfoSync(
                remoteInfo = uiState.comic.remoteInfo,
                onSyncComicInfo = { onSyncAction(ComicSyncAction.SyncComicInfo) },
                comicInfoState = getSyncActionVisualState(ComicSyncAction.SyncComicInfo),
            )
        }
    }

    scope.item {
        val anyFileActionLoading = fileActions.any { getSyncActionVisualState(it) == SyncActionVisualState.LOADING }
        Acerola.Component.SettingSection(
            title = stringResource(id = R.string.title_comic_preferences_files),
            lead = stringResource(id = R.string.description_comic_preferences_files),
            modifier = sectionModifier,
        ) {
            Comic.Component.FileSyncActions(
                onSyncChapters = { onSyncAction(ComicSyncAction.SyncChaptersLocal) },
                onRescanCover = { onSyncAction(ComicSyncAction.RescanComic) },
                syncChaptersState = getSyncActionVisualState(ComicSyncAction.SyncChaptersLocal),
                rescanCoverState = getSyncActionVisualState(ComicSyncAction.RescanComic),
                enabled = !anyFileActionLoading,
            )
        }
    }

    scope.item {
        var showPeerPicker by remember { mutableStateOf(false) }
        var pendingDirection by remember { mutableStateOf(SyncDirection.PUSH) }

        Acerola.Component.SettingSection(
            title = stringResource(id = R.string.title_comic_preferences_devices),
            lead = stringResource(id = R.string.description_comic_preferences_devices),
            modifier = sectionModifier,
        ) {
            Comic.Component.SyncWithPeerAction(
                state = syncWithPeerState,
                onPush = {
                    onLoadPairedPeers()
                    pendingDirection = SyncDirection.PUSH
                    showPeerPicker = true
                },
                onPull = {
                    onLoadPairedPeers()
                    pendingDirection = SyncDirection.PULL
                    showPeerPicker = true
                },
            )
        }

        if (showPeerPicker) {
            Main.Common.Component.PeerPickerSheet(
                peers = pairedPeers,
                onSelect = { peerId ->
                    showPeerPicker = false
                    onSyncAction(ComicSyncAction.SyncWithPeer(peerId, pendingDirection))
                },
                onDismiss = { showPeerPicker = false },
            )
        }
    }

    scope.item {
        val anyFileActionLoading = fileActions.any { getSyncActionVisualState(it) == SyncActionVisualState.LOADING }
        Acerola.Component.SettingSection(
            title = stringResource(id = R.string.title_comic_preferences_cover),
            lead = stringResource(id = R.string.description_comic_preferences_cover),
            modifier = sectionModifier,
        ) {
            Comic.Component.CoverActions(
                onExtractFirstPageAsCover = { onSyncAction(ComicSyncAction.ExtractFirstPageAsCover) },
                onExtractVolumeCovers =
                    if (uiState.volumeViewMode == VolumeViewType.COVER_VOLUME) {
                        { onSyncAction(ComicSyncAction.ExtractVolumeCovers) }
                    } else {
                        null
                    },
                extractFirstPageState = getSyncActionVisualState(ComicSyncAction.ExtractFirstPageAsCover),
                extractVolumeCoversState = getSyncActionVisualState(ComicSyncAction.ExtractVolumeCovers),
                enabled = !anyFileActionLoading,
            )
        }
    }

    scope.item {
        var showClearMetadataDialog by remember { mutableStateOf(false) }

        Acerola.Component.SettingSection(
            title = stringResource(id = R.string.title_comic_preferences_danger),
            lead = stringResource(id = R.string.description_comic_preferences_danger),
            modifier = sectionModifier,
        ) {
            Acerola.Component.SettingRow(
                title = stringResource(id = R.string.action_clear_metadata),
                description = stringResource(id = R.string.description_clear_metadata),
                icon = Icons.Rounded.LayersClear,
                tone = Tone.Destructive,
                content = {
                    Acerola.Component.Button(
                        text = stringResource(id = R.string.action_comic_preferences_clear),
                        onClick = { showClearMetadataDialog = true },
                        variant = ButtonVariant.Destructive,
                    )
                },
            )
        }

        if (showClearMetadataDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showClearMetadataDialog = false },
                title = stringResource(id = R.string.dialog_clear_metadata_title),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_comic_preferences_clear),
                        onClick = {
                            showClearMetadataDialog = false
                            onAction(ComicAction.ClearMetadata)
                        },
                        variant = ButtonVariant.Destructive,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showClearMetadataDialog = false },
                    )
                },
                content = { Text(text = stringResource(id = R.string.dialog_clear_metadata_message)) },
            )
        }
    }

    scope.item { Spacer(modifier = Modifier.height(32.dp)) }
}
