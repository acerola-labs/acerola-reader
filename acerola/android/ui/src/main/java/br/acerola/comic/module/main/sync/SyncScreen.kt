package br.acerola.comic.module.main.sync

import android.content.Intent
import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.rounded.Sync
import androidx.compose.material.icons.filled.ContentPaste
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Devices
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.PersonRemove
import androidx.compose.material.icons.filled.PhoneAndroid
import androidx.compose.material.icons.filled.Public
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.SettingsSuggest
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ActionIcon
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.component.Menu
import br.acerola.comic.common.ux.component.MenuItem
import br.acerola.comic.common.ux.component.ScreenHeader
import br.acerola.comic.common.ux.component.SettingAction
import br.acerola.comic.common.ux.component.SettingActionIcon
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SettingSection
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.Spinner
import br.acerola.comic.common.ux.component.Switch
import br.acerola.comic.common.ux.component.TextField
import br.acerola.comic.common.ux.component.ToggleCard
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.component.colors
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.network.MobileDataSyncViewModel
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.sync.component.RemoteLibrarySheet
import br.acerola.comic.module.main.sync.state.ConnectError
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.module.main.sync.state.RelaySettingsUiState
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.module.main.sync.state.SyncResult
import br.acerola.comic.module.main.sync.state.SyncUiState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.module.main.transferlog.TransferLogActivity
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.ui.R
import br.acerola.comic.util.p2p.PairingCode
import br.acerola.comic.util.p2p.QrBitmapGenerator
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

@Composable
fun Main.Sync.Template.Screen(
    viewModel: SyncViewModel = hiltViewModel(),
    mobileDataSyncViewModel: MobileDataSyncViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsState()
    val allowMobileDataSync by mobileDataSyncViewModel.allowMobileDataSync.collectAsState()

    SyncLayout(
        uiState = uiState,
        onAction = viewModel::onAction,
        allowMobileDataSync = allowMobileDataSync,
        onToggleAllowMobileDataSync = mobileDataSyncViewModel::setAllowMobileDataSync,
    )
}

// Same screen as the desktop's /network: brand header, then one section per subject — this
// device, the pairing code, connecting to another device, paired devices, connection (relays)
// and transfers. In landscape the pairing and connect sections move to a second column.
@Composable
private fun SyncLayout(
    uiState: SyncUiState,
    onAction: (SyncAction) -> Unit,
    allowMobileDataSync: Boolean,
    onToggleAllowMobileDataSync: (Boolean) -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = LocalSnackbarHostState.current
    val scanFailedMessage = stringResource(id = R.string.error_sync_scan_failed)
    val scanEmptyMessage = stringResource(id = R.string.error_sync_scan_empty)
    var pasteValue by remember { mutableStateOf("") }
    var peerPendingRemoval by remember { mutableStateOf<PairedPeer?>(null) }

    fun startScan(onResult: (String) -> Unit) {
        val options = GmsBarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).build()
        GmsBarcodeScanning
            .getClient(context, options)
            .startScan()
            .addOnSuccessListener { barcode ->
                val value = barcode.rawValue
                if (value != null) {
                    onResult(value)
                } else {
                    AcerolaLogger.w("SyncScreen", "QR scan succeeded but rawValue was null", LogSource.UI)
                    scope.launch { snackbarHostState.showSnackbar(scanEmptyMessage, SnackbarVariant.Error) }
                }
            }.addOnFailureListener { error ->
                AcerolaLogger.e("SyncScreen", "QR scan failed", LogSource.UI, error)
                scope.launch { snackbarHostState.showSnackbar(scanFailedMessage, SnackbarVariant.Error) }
            }
    }

    LaunchedEffect(uiState.connectError) {
        val message =
            when (uiState.connectError) {
                ConnectError.INVALID_CODE -> context.getString(R.string.error_sync_connect_invalid_code)
                ConnectError.CONNECTION_FAILED -> context.getString(R.string.error_sync_connect_failed)
                null -> return@LaunchedEffect
            }
        snackbarHostState.showSnackbar(message, SnackbarVariant.Error)
    }

    // A valid code moves on to the confirmation dialog, so the field is cleared for the next one.
    LaunchedEffect(uiState.pendingConnect) {
        if (uiState.pendingConnect != null) pasteValue = ""
    }

    val isLandscape = LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE
    val onScan = { startScan { value -> onAction(SyncAction.ProposeConnect(value)) } }

    val mainSections: @Composable () -> Unit = {
        ThisDeviceSection(uiState = uiState, onAction = onAction)
        PeersSection(uiState = uiState, onAction = onAction, onRemoveClick = { peerPendingRemoval = it })
        RelaySettingsSection(
            relaySettings = uiState.relaySettings,
            irohServicesTicketError = uiState.irohServicesTicketError,
            allowMobileDataSync = allowMobileDataSync,
            onToggleAllowMobileDataSync = onToggleAllowMobileDataSync,
            onAction = onAction,
        )
        TransfersSection(uiState = uiState)
    }
    val pairingSections: @Composable () -> Unit = {
        PairingSection(uiState = uiState)
        ConnectSection(
            uiState = uiState,
            pasteValue = pasteValue,
            onPasteValueChange = { pasteValue = it },
            onConnect = { onAction(SyncAction.ProposeConnect(pasteValue)) },
            onScan = onScan,
            onDismissError = { onAction(SyncAction.DismissConnectError) },
        )
    }

    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .background(AcerolaExtendedTheme.colors.background)
                .verticalScroll(rememberScrollState())
                .padding(bottom = if (isLandscape) SpacingTokens.Huge else BottomBarClearance),
    ) {
        Acerola.Component.ScreenHeader(
            eyebrow = stringResource(id = R.string.label_sync_eyebrow),
            title = stringResource(id = R.string.label_sync_activity),
            lead = stringResource(id = R.string.description_sync_activity),
        )

        if (isLandscape) {
            Row(
                modifier = Modifier.padding(horizontal = SpacingTokens.Large),
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
            ) {
                Column(modifier = Modifier.weight(1.15f), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large)) {
                    mainSections()
                }
                Column(modifier = Modifier.weight(0.85f), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large)) {
                    pairingSections()
                }
            }
        } else {
            Column(
                modifier = Modifier.padding(horizontal = SpacingTokens.Large),
                verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
            ) {
                ThisDeviceSection(uiState = uiState, onAction = onAction)
                pairingSections()
                PeersSection(uiState = uiState, onAction = onAction, onRemoveClick = { peerPendingRemoval = it })
                RelaySettingsSection(
                    relaySettings = uiState.relaySettings,
                    irohServicesTicketError = uiState.irohServicesTicketError,
                    allowMobileDataSync = allowMobileDataSync,
                    onToggleAllowMobileDataSync = onToggleAllowMobileDataSync,
                    onAction = onAction,
                )
                TransfersSection(uiState = uiState)
            }
        }
    }

    if (uiState.pendingConnect != null) {
        ConfirmConnectDialog(
            peerId = uiState.pendingConnect.peerId,
            onConfirm = { onAction(SyncAction.ConfirmConnect) },
            onCancel = { onAction(SyncAction.CancelConnect) },
        )
    }

    if (uiState.trustedPeerDialogPeerId != null) {
        TofuDialog(
            peerId = uiState.trustedPeerDialogPeerId,
            onDismiss = { onAction(SyncAction.DismissTrustDialog) },
        )
    }

    peerPendingRemoval?.let { peer ->
        RemovePeerDialog(
            peer = peer,
            onConfirm = {
                onAction(SyncAction.RemovePeer(peer.peerId))
                peerPendingRemoval = null
            },
            onCancel = { peerPendingRemoval = null },
        )
    }

    uiState.browsingPeerId?.let { peerId ->
        val peerDisplayName =
            uiState.pairedPeers.find { it.peerId == peerId }?.let { it.nickname ?: it.deviceName }
                ?: PairingCode.shortId(peerId)
        Main.Sync.Component.RemoteLibrarySheet(
            peerDisplayName = peerDisplayName,
            comics = uiState.remoteLibrary,
            isLoading = !uiState.remoteLibraryLoaded && uiState.browseLibraryError == null,
            errorMessage = uiState.browseLibraryError,
            errorType = uiState.browseLibraryErrorType,
            onSelectComic = { comicName -> onAction(SyncAction.SyncComic(peerId, comicName)) },
            onDismiss = { onAction(SyncAction.DismissLibraryBrowse) },
            coverPathFor = { comicName -> uiState.remoteCoverPaths[coverKey(peerId, comicName)] },
        )
    }
}

// The floating pill navigation (52dp + 12dp gap) plus breathing room under the last section.
private val BottomBarClearance = 96.dp

/** "This device": its name and id, copy the id or rename it inline (LocalSend-style alias). */
@Composable
private fun ThisDeviceSection(
    uiState: SyncUiState,
    onAction: (SyncAction) -> Unit,
) {
    val clipboardManager = LocalClipboardManager.current
    val haptic = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = LocalSnackbarHostState.current
    val copiedMessage = stringResource(id = R.string.label_sync_copied)
    val renamedMessage = stringResource(id = R.string.label_sync_rename_success)

    var idCopiedRecently by remember { mutableStateOf(false) }
    LaunchedEffect(idCopiedRecently) {
        if (idCopiedRecently) {
            delay(2000)
            idCopiedRecently = false
        }
    }

    var editingName by remember { mutableStateOf(false) }
    var nameDraft by remember { mutableStateOf("") }

    Acerola.Component.SettingSection(
        title = stringResource(id = R.string.title_sync_this_device),
        lead = stringResource(id = R.string.description_sync_this_device),
    ) {
        Acerola.Component.SettingRow(
            title = uiState.localDeviceName,
            description = PairingCode.shortId(uiState.localId),
            icon = Icons.Default.PhoneAndroid,
            tone = Tone.Primary,
            control = {
                Acerola.Component.Badge(
                    text =
                        stringResource(
                            id =
                                when (uiState.mode) {
                                    NetworkMode.LOCAL -> R.string.label_sync_mode_local
                                    NetworkMode.RELAY -> R.string.label_sync_mode_relay
                                },
                        ),
                    tone = Tone.Success,
                )
            },
            // While renaming, the editor below takes the place of the two actions.
            actions =
                if (editingName) {
                    emptyList()
                } else {
                    listOf(
                        SettingAction(
                            text = stringResource(id = if (idCopiedRecently) R.string.label_sync_copied else R.string.action_sync_copy_id),
                            icon = Icons.Default.ContentCopy,
                            onClick = {
                                clipboardManager.setText(AnnotatedString(uiState.localId))
                                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                idCopiedRecently = true
                                scope.launch { snackbarHostState.showSnackbar(copiedMessage, SnackbarVariant.Success) }
                            },
                            state = if (idCopiedRecently) SyncActionVisualState.SUCCESS else SyncActionVisualState.IDLE,
                        ),
                        SettingAction(
                            text = stringResource(id = R.string.action_sync_rename_peer),
                            icon = Icons.Default.Edit,
                            onClick = {
                                nameDraft = uiState.localDeviceName
                                editingName = true
                            },
                        ),
                    )
                },
            content = {
                if (editingName) {
                    Acerola.Component.TextField(
                        value = nameDraft,
                        onValueChange = { nameDraft = it },
                        placeholder = stringResource(id = R.string.hint_sync_rename_device),
                    )
                    Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                        Acerola.Component.Button(
                            text = stringResource(id = R.string.action_sync_rename_save),
                            onClick = {
                                onAction(SyncAction.RenameDevice(nameDraft))
                                scope.launch { snackbarHostState.showSnackbar(renamedMessage, SnackbarVariant.Success) }
                                editingName = false
                            },
                            variant = ButtonVariant.Contrast,
                            enabled = nameDraft.isNotBlank(),
                        )
                        Acerola.Component.Button(
                            text = stringResource(id = R.string.action_cancel),
                            onClick = { editingName = false },
                            variant = ButtonVariant.Secondary,
                        )
                    }
                }
            },
        )
    }
}

/** "Pairing code": the QR on a white tile (it needs it to scan), copy / show the raw code. */
@Composable
private fun PairingSection(uiState: SyncUiState) {
    val clipboardManager = LocalClipboardManager.current
    val haptic = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()
    val snackbarHostState = LocalSnackbarHostState.current
    val copiedMessage = stringResource(id = R.string.label_sync_copied)
    var showRawCode by remember { mutableStateOf(false) }
    var codeCopiedRecently by remember { mutableStateOf(false) }
    val code = uiState.pairingCode

    LaunchedEffect(codeCopiedRecently) {
        if (codeCopiedRecently) {
            delay(2000)
            codeCopiedRecently = false
        }
    }

    Acerola.Component.SettingSection(
        title = stringResource(id = R.string.title_sync_pairing),
        lead = stringResource(id = R.string.description_sync_pairing),
    ) {
        Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            Box(
                modifier =
                    Modifier
                        .size(232.dp)
                        .clip(AcerolaShapes.Tile)
                        // The QR code needs a white background to scan, in both modes.
                        .background(AcerolaColors.OnScrim)
                        .padding(SpacingTokens.Large),
                contentAlignment = Alignment.Center,
            ) {
                if (code != null) {
                    val bitmap = remember(code) { QrBitmapGenerator.generate(code) }
                    Image(
                        bitmap = bitmap.asImageBitmap(),
                        contentDescription = stringResource(id = R.string.title_sync_pairing),
                        modifier = Modifier.fillMaxSize(),
                    )
                } else {
                    Acerola.Component.Spinner(size = SizeTokens.IconLarge, color = MaterialTheme.colorScheme.primary)
                }
            }
        }

        Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
            Acerola.Component.Button(
                text = stringResource(id = if (codeCopiedRecently) R.string.label_sync_copied else R.string.action_sync_copy_code),
                onClick = {
                    code?.let {
                        clipboardManager.setText(AnnotatedString(it))
                        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                        codeCopiedRecently = true
                        scope.launch { snackbarHostState.showSnackbar(copiedMessage, SnackbarVariant.Success) }
                    }
                },
                variant = if (codeCopiedRecently) ButtonVariant.Success else ButtonVariant.Secondary,
                icon = if (codeCopiedRecently) Icons.Default.Check else Icons.Default.ContentCopy,
                enabled = code != null,
                modifier = Modifier.weight(1f),
            )
            Acerola.Component.Button(
                text = stringResource(id = if (showRawCode) R.string.action_sync_hide_code else R.string.action_sync_show_code),
                onClick = { showRawCode = !showRawCode },
                variant = ButtonVariant.Secondary,
                icon = if (showRawCode) Icons.Default.VisibilityOff else Icons.Default.Visibility,
                enabled = code != null,
                modifier = Modifier.weight(1f),
            )
        }

        if (showRawCode && code != null) {
            SelectionContainer {
                Text(
                    text = code,
                    style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                    color = AcerolaExtendedTheme.colors.foreground,
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .clip(AcerolaShapes.Inner)
                            .background(AcerolaExtendedTheme.colors.surface)
                            .padding(SpacingTokens.Medium),
                )
            }
        }

        Notice(text = stringResource(id = R.string.description_sync_security_note), tone = Tone.Warning)
    }
}

/** "Connect to a device": paste (or scan) the other device's code, then connect. */
@Composable
private fun ConnectSection(
    uiState: SyncUiState,
    pasteValue: String,
    onPasteValueChange: (String) -> Unit,
    onConnect: () -> Unit,
    onScan: () -> Unit,
    onDismissError: () -> Unit,
) {
    val clipboardManager = LocalClipboardManager.current

    Acerola.Component.SettingSection(
        title = stringResource(id = R.string.title_sync_connect),
        lead = stringResource(id = R.string.description_sync_connect),
    ) {
        Acerola.Component.TextField(
            value = pasteValue,
            onValueChange = {
                onPasteValueChange(it)
                if (uiState.connectError != null) onDismissError()
            },
            placeholder = stringResource(id = R.string.hint_sync_connect_code),
            isError = uiState.connectError != null,
            errorText =
                when (uiState.connectError) {
                    ConnectError.INVALID_CODE -> stringResource(id = R.string.error_sync_connect_invalid_code)
                    ConnectError.CONNECTION_FAILED -> stringResource(id = R.string.error_sync_connect_failed)
                    null -> null
                },
            trailing = {
                Acerola.Component.IconButton(
                    icon = Icons.Default.ContentPaste,
                    contentDescription = stringResource(id = R.string.action_sync_paste),
                    onClick = {
                        onPasteValueChange(clipboardManager.getText()?.text.orEmpty())
                        if (uiState.connectError != null) onDismissError()
                    },
                    variant = ButtonVariant.Ghost,
                    size = 36.dp,
                )
            },
        )

        Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_sync_scan_short),
                onClick = onScan,
                variant = ButtonVariant.Secondary,
                icon = Icons.Default.QrCodeScanner,
                modifier = Modifier.weight(1f),
            )
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_sync_connect),
                onClick = onConnect,
                variant = ButtonVariant.Primary,
                enabled = pasteValue.isNotBlank(),
                loading = uiState.connecting,
                modifier = Modifier.weight(1f),
            )
        }
    }
}

/** "Paired devices": one row per peer, or the empty state while there is none. */
@Composable
private fun PeersSection(
    uiState: SyncUiState,
    onAction: (SyncAction) -> Unit,
    onRemoveClick: (PairedPeer) -> Unit,
) {
    Acerola.Component.SettingSection(
        title = stringResource(id = R.string.title_sync_peers),
        lead = stringResource(id = R.string.description_sync_peers),
    ) {
        if (uiState.pairedPeers.isEmpty()) {
            Acerola.Component.EmptyState(
                title = stringResource(id = R.string.label_sync_no_peers_title),
                hint = stringResource(id = R.string.label_sync_no_peers),
                icon = Icons.Default.Devices,
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )
        } else {
            uiState.pairedPeers.forEach { peer ->
                PeerRow(
                    peer = peer,
                    isOnline = peer.peerId in uiState.connectedPeerIds,
                    syncingKeys = uiState.syncingKeys,
                    lastSyncedAt = uiState.lastSyncedByPeer[peer.peerId],
                    lastResult = uiState.lastSyncResultByPeer[peer.peerId],
                    onAction = onAction,
                    onRemoveClick = { onRemoveClick(peer) },
                )
            }
        }
    }
}

@Composable
private fun PeerRow(
    peer: PairedPeer,
    isOnline: Boolean,
    syncingKeys: Set<String>,
    lastSyncedAt: Long?,
    lastResult: SyncResult?,
    onAction: (SyncAction) -> Unit,
    onRemoveClick: () -> Unit,
) {
    val historySyncing = syncKey(peer.peerId, SYNC_KIND_HISTORY) in syncingKeys
    val filesSyncing = syncKey(peer.peerId, SYNC_KIND_FILES) in syncingKeys
    val anySyncing = historySyncing || filesSyncing
    var menuExpanded by remember { mutableStateOf(false) }
    // "Sync all" fires history + files at once — asks for explicit confirmation first,
    // since there's no way to know the data volume involved beforehand.
    var showSyncAllConfirm by remember { mutableStateOf(false) }
    var editingNickname by remember { mutableStateOf(false) }
    var nicknameDraft by remember { mutableStateOf("") }

    // Flashes the success state for a moment when a session ends well, instead of going straight
    // back to idle with no feedback that the sync worked.
    var showSuccess by remember(peer.peerId) { mutableStateOf(false) }
    LaunchedEffect(lastResult) {
        if (lastResult?.state == LogState.SUCCESS) {
            showSuccess = true
            delay(2000)
            showSuccess = false
        }
    }

    val isErrorIdle = !anySyncing && !showSuccess && lastResult?.state == LogState.ERROR
    val description =
        when {
            isOnline -> stringResource(id = R.string.label_sync_peer_online)
            lastSyncedAt != null -> stringResource(id = R.string.label_sync_last_synced, formatLogTimestamp(lastSyncedAt))
            else -> stringResource(id = R.string.label_sync_never_synced)
        }

    Acerola.Component.SettingRow(
        title = peer.nickname ?: peer.deviceName ?: PairingCode.shortId(peer.peerId),
        description = description,
        icon = Icons.Default.Devices,
        tone = if (isOnline) Tone.Success else Tone.Neutral,
        control = {
            Box {
                Acerola.Component.ActionIcon(
                    icon = Icons.Default.MoreVert,
                    onClick = { menuExpanded = true },
                    contentDescription = stringResource(id = R.string.description_icon_sync_peer_more_actions),
                )
                Acerola.Component.Menu(expanded = menuExpanded, onDismissRequest = { menuExpanded = false }) {
                    Acerola.Component.MenuItem(
                        text = stringResource(id = R.string.action_sync_history),
                        icon = Icons.Default.History,
                        enabled = !historySyncing,
                        onClick = {
                            menuExpanded = false
                            onAction(SyncAction.SyncHistory(peer.peerId))
                        },
                    )
                    Acerola.Component.MenuItem(
                        text = stringResource(id = R.string.action_sync_files),
                        icon = Icons.Default.Description,
                        enabled = !filesSyncing,
                        onClick = {
                            menuExpanded = false
                            onAction(SyncAction.SyncFiles(peer.peerId))
                        },
                    )
                    Acerola.Component.MenuItem(
                        text = stringResource(id = R.string.action_sync_browse_library),
                        icon = Icons.Default.Search,
                        enabled = !filesSyncing,
                        onClick = {
                            menuExpanded = false
                            onAction(SyncAction.BrowseLibrary(peer.peerId))
                        },
                    )
                    Acerola.Component.MenuItem(
                        text = stringResource(id = R.string.action_sync_rename_peer),
                        icon = Icons.Default.Edit,
                        onClick = {
                            menuExpanded = false
                            nicknameDraft = peer.nickname ?: ""
                            editingNickname = true
                        },
                    )
                    Acerola.Component.MenuItem(
                        text = stringResource(id = R.string.action_sync_remove_peer),
                        icon = Icons.Default.PersonRemove,
                        destructive = true,
                        onClick = {
                            menuExpanded = false
                            onRemoveClick()
                        },
                    )
                }
            }
        },
        actions =
            if (editingNickname) {
                emptyList()
            } else {
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_sync_all),
                        icon = Icons.Rounded.Sync,
                        onClick = { showSyncAllConfirm = true },
                        enabled = !anySyncing,
                        state =
                            when {
                                anySyncing -> SyncActionVisualState.LOADING
                                showSuccess -> SyncActionVisualState.SUCCESS
                                else -> SyncActionVisualState.IDLE
                            },
                        loadingText = stringResource(id = R.string.action_config_syncing),
                    ),
                )
            },
        content = {
            if (isErrorIdle) {
                Text(
                    text =
                        lastResult?.errorType?.uiMessage?.asString()
                            ?: lastResult?.message?.let { stringResource(id = R.string.error_sync_last_attempt_failed, it) }
                            ?: stringResource(id = R.string.error_sync_connect_failed),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }
            if (editingNickname) {
                Acerola.Component.TextField(
                    value = nicknameDraft,
                    onValueChange = { nicknameDraft = it },
                    placeholder = stringResource(id = R.string.hint_sync_rename_peer),
                )
                Text(
                    text = stringResource(id = R.string.label_sync_rename_peer_hint),
                    style = MaterialTheme.typography.bodySmall,
                    color = AcerolaExtendedTheme.colors.mutedForeground,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                    Acerola.Component.Button(
                        text = stringResource(id = R.string.action_sync_rename_save),
                        onClick = {
                            onAction(SyncAction.RenamePeer(peer.peerId, nicknameDraft))
                            editingNickname = false
                        },
                        variant = ButtonVariant.Contrast,
                    )
                    Acerola.Component.Button(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { editingNickname = false },
                        variant = ButtonVariant.Secondary,
                    )
                }
            }
        },
    )

    if (showSyncAllConfirm) {
        SyncAllConfirmDialog(
            peer = peer,
            onConfirm = {
                showSyncAllConfirm = false
                onAction(SyncAction.SyncAll(peer.peerId))
            },
            onCancel = { showSyncAllConfirm = false },
        )
    }
}

/** "Connection": the relay sources as switch rows, plus mobile data and restarting the P2P node. */
@Composable
private fun RelaySettingsSection(
    relaySettings: RelaySettingsUiState,
    irohServicesTicketError: Boolean,
    allowMobileDataSync: Boolean,
    onToggleAllowMobileDataSync: (Boolean) -> Unit,
    onAction: (SyncAction) -> Unit,
) {
    var customExpanded by remember { mutableStateOf(false) }
    var ticketExpanded by remember { mutableStateOf(false) }
    // Switching the active relay source rebuilds the P2P node with a different `RelayMap` —
    // paired peers that don't converge on the same relay stay unreachable until that happens.
    // Confirms before applying, instead of switching right on click.
    var pendingRelayAction by remember { mutableStateOf<SyncAction?>(null) }

    val activeSourceCount = (if (relaySettings.useAcerolaRelay) 1 else 0) + relaySettings.customRelayUrls.size
    val summary =
        when {
            relaySettings.useIrohPublicNetwork -> stringResource(id = R.string.label_relay_settings_summary_iroh_public)
            activeSourceCount == 0 -> stringResource(id = R.string.label_relay_settings_summary_mdns_only)
            else -> stringResource(id = R.string.label_relay_settings_summary_active, activeSourceCount)
        }

    Acerola.Component.SettingSection(
        title = stringResource(id = R.string.title_relay_settings),
        lead = stringResource(id = R.string.description_relay_settings),
    ) {
        Notice(text = summary, tone = Tone.Info)

        Acerola.Component.ToggleCard(
            title = stringResource(id = R.string.label_relay_settings_use_acerola_relay),
            subtitle = stringResource(id = R.string.label_relay_settings_use_acerola_relay_desc, RelayPreference.DEFAULT_ACEROLA_RELAY_URL),
            active = relaySettings.useAcerolaRelay,
            enabled = !relaySettings.useIrohPublicNetwork,
            onClick = { pendingRelayAction = SyncAction.ToggleUseAcerolaRelay(!relaySettings.useAcerolaRelay) },
            icon = Icons.Default.Wifi,
        )

        Acerola.Component.SettingRow(
            title = stringResource(id = R.string.title_relay_settings_custom_relays),
            description =
                if (relaySettings.customRelayUrls.isEmpty()) {
                    stringResource(id = R.string.label_relay_settings_custom_relays_empty)
                } else {
                    stringResource(id = R.string.label_relay_settings_custom_relays_count, relaySettings.customRelayUrls.size)
                },
            icon = Icons.Default.SettingsSuggest,
            tone = if (relaySettings.customRelayUrls.isNotEmpty()) Tone.Primary else Tone.Neutral,
            actions =
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_relay_settings_manage),
                        icon = if (customExpanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                        onClick = { customExpanded = !customExpanded },
                        iconPlacement = SettingActionIcon.End,
                    ),
                ),
            content = {
                if (customExpanded) {
                    RelayUrlListEditor(
                        urls = relaySettings.customRelayUrls,
                        placeholder = stringResource(id = R.string.hint_relay_settings_custom_relay_add),
                        removeContentDescription = stringResource(id = R.string.action_relay_settings_custom_relay_remove),
                        enabled = !relaySettings.useIrohPublicNetwork,
                        onAdd = { onAction(SyncAction.AddCustomRelayUrl(it)) },
                        onRemove = { onAction(SyncAction.RemoveCustomRelayUrl(it)) },
                    )
                }
            },
        )

        Acerola.Component.SettingRow(
            title = stringResource(id = R.string.label_relay_settings_use_iroh_public_network),
            description =
                if (relaySettings.hasIrohServicesTicket) {
                    stringResource(id = R.string.label_relay_settings_use_iroh_public_network_desc)
                } else {
                    stringResource(id = R.string.label_relay_settings_iroh_services_ticket_required_hint)
                },
            icon = Icons.Default.Public,
            tone = if (relaySettings.useIrohPublicNetwork) Tone.Primary else Tone.Neutral,
            control = {
                Acerola.Component.Switch(
                    checked = relaySettings.useIrohPublicNetwork,
                    onCheckedChange = { pendingRelayAction = SyncAction.ToggleUseIrohPublicNetwork(it) },
                    enabled = relaySettings.hasIrohServicesTicket,
                )
            },
            actions =
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_relay_settings_ticket),
                        icon = if (ticketExpanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                        onClick = { ticketExpanded = !ticketExpanded },
                        iconPlacement = SettingActionIcon.End,
                    ),
                ),
            content = {
                if (ticketExpanded) {
                    IrohServicesTicketEditor(
                        hasTicket = relaySettings.hasIrohServicesTicket,
                        hasError = irohServicesTicketError,
                        onSave = { onAction(SyncAction.SetIrohServicesTicket(it)) },
                        onRemove = { onAction(SyncAction.ClearIrohServicesTicket) },
                        onDismissError = { onAction(SyncAction.DismissIrohServicesTicketError) },
                    )
                }
            },
        )

        Acerola.Component.ToggleCard(
            title = stringResource(id = R.string.label_relay_settings_allow_mobile_data_sync),
            subtitle = stringResource(id = R.string.label_relay_settings_allow_mobile_data_sync_desc),
            active = allowMobileDataSync,
            onClick = { onToggleAllowMobileDataSync(!allowMobileDataSync) },
            icon = Icons.Default.PhoneAndroid,
        )

        // Manual restart (LocalSend style): shuts the P2P node down and starts a new one with the
        // same identity — the escape hatch when the live relay switch above isn't enough.
        Acerola.Component.SettingRow(
            title = stringResource(id = R.string.title_relay_settings_restart),
            description = stringResource(id = R.string.label_relay_settings_restart_desc),
            icon = Icons.Default.Refresh,
            actions =
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_relay_settings_restart),
                        icon = Icons.Default.Refresh,
                        onClick = { onAction(SyncAction.RestartP2p) },
                        state = if (relaySettings.restarting) SyncActionVisualState.LOADING else SyncActionVisualState.IDLE,
                        loadingText = stringResource(id = R.string.action_relay_settings_restarting),
                    ),
                ),
            content = {
                if (relaySettings.restartError) {
                    Text(
                        text = stringResource(id = R.string.error_relay_settings_restart_failed),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            },
        )
    }

    pendingRelayAction?.let { action ->
        SwitchRelayConfirmDialog(
            onConfirm = {
                onAction(action)
                pendingRelayAction = null
            },
            onCancel = { pendingRelayAction = null },
        )
    }
}

/** Confirms before switching the active relay source — see the comment in [RelaySettingsSection]. */
@Composable
private fun SwitchRelayConfirmDialog(
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_relay_settings_switch_confirm),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_relay_settings_switch_confirm),
                onClick = onConfirm,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.label_relay_settings_switch_confirm_desc))
    }
}

/** Ticket for the user's OWN account at `services.iroh.computer` — never a project secret
 *  embedded in the build (see `RelayModeConfig::IrohDefault` on the Rust side). The value itself is
 *  never shown back (it's a real credential, stored in the node's encrypted vault) — only
 *  [hasTicket] reaches here. */
@Composable
private fun IrohServicesTicketEditor(
    hasTicket: Boolean,
    hasError: Boolean,
    onSave: (String) -> Unit,
    onRemove: () -> Unit,
    onDismissError: () -> Unit,
) {
    var draft by remember { mutableStateOf("") }

    Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
        Text(
            text =
                stringResource(
                    id =
                        if (hasTicket) {
                            R.string.label_relay_settings_iroh_services_ticket_configured
                        } else {
                            R.string.label_relay_settings_iroh_services_ticket_not_configured
                        },
                ),
            style = MaterialTheme.typography.bodySmall,
            color = AcerolaExtendedTheme.colors.mutedForeground,
        )
        Acerola.Component.TextField(
            value = draft,
            onValueChange = {
                draft = it
                if (hasError) onDismissError()
            },
            placeholder = stringResource(id = R.string.hint_relay_settings_iroh_services_ticket),
            isError = hasError,
            errorText = stringResource(id = R.string.error_relay_settings_iroh_services_ticket_invalid),
            visualTransformation = PasswordVisualTransformation(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
            Acerola.Component.Button(
                text =
                    stringResource(
                        id =
                            if (hasTicket) {
                                R.string.action_relay_settings_iroh_services_ticket_replace
                            } else {
                                R.string.action_relay_settings_iroh_services_ticket_save
                            },
                    ),
                onClick = {
                    val trimmed = draft.trim()
                    if (trimmed.isNotBlank()) {
                        onSave(trimmed)
                        draft = ""
                    }
                },
                variant = ButtonVariant.Contrast,
                enabled = draft.isNotBlank(),
            )
            if (hasTicket) {
                Acerola.Component.Button(
                    text = stringResource(id = R.string.action_relay_settings_iroh_services_ticket_remove),
                    onClick = onRemove,
                    variant = ButtonVariant.Destructive,
                )
            }
        }
        Text(
            text = stringResource(id = R.string.label_relay_settings_iroh_services_ticket_help),
            style = MaterialTheme.typography.bodySmall,
            color = AcerolaExtendedTheme.colors.mutedForeground,
        )
    }
}

/** Mirrors the desktop's own relay list (`acerola-network-relay-settings-card.svelte`): validates
 *  the URL (http/https) before accepting, disabled while the Iroh public network is active. */
@Composable
private fun RelayUrlListEditor(
    urls: List<String>,
    placeholder: String,
    removeContentDescription: String,
    enabled: Boolean,
    onAdd: (String) -> Unit,
    onRemove: (String) -> Unit,
) {
    var draft by remember { mutableStateOf("") }
    var showError by remember { mutableStateOf(false) }

    fun add() {
        val trimmed = draft.trim()
        if (isValidRelayUrl(trimmed)) {
            onAdd(trimmed)
            draft = ""
            showError = false
        } else {
            showError = true
        }
    }

    Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
        urls.forEach { url ->
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clip(AcerolaShapes.Full)
                        .background(AcerolaExtendedTheme.colors.card)
                        .padding(start = SpacingTokens.Large, end = SpacingTokens.ExtraSmall),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = url,
                    style = MaterialTheme.typography.bodyMedium,
                    color = AcerolaExtendedTheme.colors.foreground,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f),
                )
                Acerola.Component.IconButton(
                    icon = Icons.Default.Delete,
                    contentDescription = removeContentDescription,
                    onClick = { onRemove(url) },
                    variant = ButtonVariant.Ghost,
                    enabled = enabled,
                )
            }
        }

        Acerola.Component.TextField(
            value = draft,
            onValueChange = {
                draft = it
                showError = false
            },
            placeholder = placeholder,
            enabled = enabled,
            isError = showError,
            errorText = stringResource(id = R.string.error_relay_settings_invalid_url),
            trailing = {
                Acerola.Component.IconButton(
                    icon = Icons.Default.Add,
                    contentDescription = stringResource(id = R.string.action_add),
                    onClick = { add() },
                    variant = ButtonVariant.Contrast,
                    enabled = enabled && draft.isNotBlank(),
                    size = 36.dp,
                )
            },
        )
    }
}

private fun isValidRelayUrl(value: String): Boolean =
    try {
        val scheme = java.net.URI(value).scheme
        scheme == "http" || scheme == "https"
    } catch (error: java.net.URISyntaxException) {
        false
    }

/** "Transfers": the latest entry as a preview, and the full history on its own screen (like LocalSend). */
@Composable
private fun TransfersSection(uiState: SyncUiState) {
    val context = LocalContext.current
    val summary =
        uiState.transferLog.firstOrNull()?.let { describeEntry(it) }
            ?: stringResource(id = R.string.label_sync_activity_log_empty)

    Acerola.Component.SettingSection(
        title = stringResource(id = R.string.title_sync_activity_log),
        lead = stringResource(id = R.string.description_sync_activity_log),
    ) {
        Acerola.Component.SettingRow(
            title = summary,
            icon = Icons.Default.History,
            actions =
                listOf(
                    SettingAction(
                        text = stringResource(id = R.string.action_sync_activity_log_open),
                        icon = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                        onClick = { context.startActivity(Intent(context, TransferLogActivity::class.java)) },
                        iconPlacement = SettingActionIcon.End,
                    ),
                ),
        )
    }
}

// Brand notice: `rounded-tile`, the tone's soft background with the tone's text and an icon.
@Composable
private fun Notice(
    text: String,
    tone: Tone,
) {
    val colors = tone.colors()
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(AcerolaShapes.Tile)
                .background(colors.container)
                .padding(SpacingTokens.Large),
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        Icon(imageVector = Icons.Default.Info, contentDescription = null, tint = colors.content, modifier = Modifier.size(SizeTokens.IconSmall))
        Text(text = text, style = MaterialTheme.typography.bodyMedium, color = colors.content)
    }
}

/** Confirms before firing history + files at once — see the comment in [PeerRow]. */
@Composable
private fun SyncAllConfirmDialog(
    peer: PairedPeer,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    val peerLabel = peer.nickname ?: peer.deviceName ?: PairingCode.shortId(peer.peerId)
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_sync_all_confirm, peerLabel),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_all),
                onClick = onConfirm,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_all_confirm))
    }
}

@Composable
private fun ConfirmConnectDialog(
    peerId: String,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_sync_confirm_connect),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_connect),
                onClick = onConfirm,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_confirm_connect, PairingCode.shortId(peerId)))
    }
}

@Composable
private fun TofuDialog(
    peerId: String,
    onDismiss: () -> Unit,
) {
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onDismiss,
        title = stringResource(id = R.string.title_sync_trust_dialog),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_confirm),
                onClick = onDismiss,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_trust_dialog, peerId))
    }
}

@Composable
private fun RemovePeerDialog(
    peer: PairedPeer,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    val peerLabel = peer.deviceName ?: PairingCode.shortId(peer.peerId)
    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_sync_remove_peer_confirm, peerLabel),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_remove_peer),
                onClick = onConfirm,
                variant = ButtonVariant.Destructive,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_remove_peer_confirm))
    }
}

private fun previewUiState() =
    SyncUiState(
        localId = "z6MkfriRLZTX4GC93z2XFqEmaXbXPPnvVQpEEeCQBDGGeMSw",
        localDeviceName = "Pixel 8",
        pairingCode = "acerola1:eyJpIjoiZGVtbyJ9",
        mode = NetworkMode.LOCAL,
        relaySettings = RelaySettingsUiState(),
        pairedPeers =
            listOf(
                PairedPeer(peerId = "z6Mkabc123def456ghi789", deviceName = "Desktop-Vinicius"),
                PairedPeer(peerId = "z6Mkxyz789ghi012jkl345", deviceName = null),
                PairedPeer(peerId = "z6Mkerr000fail111peer2", deviceName = "Notebook-Trabalho"),
            ),
        syncingKeys = setOf(syncKey("z6Mkabc123def456ghi789", SYNC_KIND_HISTORY)),
        lastSyncedByPeer = mapOf("z6Mkxyz789ghi012jkl345" to System.currentTimeMillis()),
        connectedPeerIds = setOf("z6Mkabc123def456ghi789"),
        lastSyncResultByPeer =
            mapOf(
                "z6Mkerr000fail111peer2" to
                    SyncResult(
                        kind = SYNC_KIND_FILES,
                        state = LogState.ERROR,
                        message = "conexão perdida",
                        timestamp = System.currentTimeMillis(),
                    ),
            ),
        transferLog =
            listOf(
                TransferLogEntry(
                    id = 3,
                    kind = SYNC_KIND_FILES,
                    status = "progress",
                    state = LogState.IN_PROGRESS,
                    comicName = "Berserk",
                    chapter = "42",
                ),
                TransferLogEntry(id = 2, kind = SYNC_KIND_FILES, status = "error", state = LogState.ERROR, message = "connection reset"),
                TransferLogEntry(id = 1, kind = SYNC_KIND_HISTORY, status = "complete", state = LogState.SUCCESS),
            ),
    )

/** [SyncLayout] reads [LocalSnackbarHostState] unconditionally, which has no default and throws
 *  if not provided — every preview needs one, even those that never show a snackbar. */
@Composable
private fun PreviewSyncLayout(uiState: SyncUiState) {
    AcerolaTheme {
        CompositionLocalProvider(LocalSnackbarHostState provides remember { SnackbarHostState() }) {
            SyncLayout(uiState = uiState, onAction = {}, allowMobileDataSync = false, onToggleAllowMobileDataSync = {})
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncLayoutPreview() {
    PreviewSyncLayout(uiState = previewUiState())
}

@Preview(name = "Empty state", showBackground = true)
@Composable
private fun SyncLayoutEmptyPreview() {
    PreviewSyncLayout(uiState = SyncUiState())
}
