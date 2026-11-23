package br.acerola.comic.module.main.sync.state

sealed interface SyncAction {
    /** Sets a custom local device alias (LocalSend-style) — applies immediately on the P2P
     *  node (next handshake already uses it, no restart needed) and persists it for future
     *  launches. */
    data class RenameDevice(
        val name: String,
    ) : SyncAction

    /** Decodes a code (pasted or scanned) and proposes pairing for confirmation. */
    data class ProposeConnect(
        val code: String,
    ) : SyncAction

    data object ConfirmConnect : SyncAction

    data object CancelConnect : SyncAction

    data class SyncHistory(
        val peerId: String,
    ) : SyncAction

    data class SyncFiles(
        val peerId: String,
    ) : SyncAction

    data class SyncAll(
        val peerId: String,
    ) : SyncAction

    data object DismissTrustDialog : SyncAction

    data object DismissConnectError : SyncAction

    data class RemovePeer(
        val peerId: String,
    ) : SyncAction

    /** Sets (or clears, with an empty string) a peer's local nickname — only on this device,
     *  it never reaches the protocol (see `PeerNicknamePreference`). */
    data class RenamePeer(
        val peerId: String,
        val nickname: String,
    ) : SyncAction

    /** Requests the peer's comic list (name + chapter count), without syncing
     *  anything — opens [br.acerola.comic.module.main.sync.RemoteLibrarySheet]. */
    data class BrowseLibrary(
        val peerId: String,
    ) : SyncAction

    data object DismissLibraryBrowse : SyncAction

    /** Syncs a single comic (discovered via [BrowseLibrary]) with the peer. */
    data class SyncComic(
        val peerId: String,
        val comicName: String,
    ) : SyncAction

    /** All relay actions below persist to DataStore AND apply right away
     *  (`P2pUseCase.applyRelaySettings`, switches the relay mode on an `Endpoint` that stays
     *  alive, without dropping connections) — see [RelaySettingsUiState]. A rare case the live
     *  switch doesn't solve on its own (connection stuck after an OS physical network change) has
     *  the manual [RestartP2p] button as a way out, which tears down and restarts the whole node. */
    data class ToggleUseAcerolaRelay(
        val value: Boolean,
    ) : SyncAction

    data class ToggleUseIrohPublicNetwork(
        val value: Boolean,
    ) : SyncAction

    data class AddCustomRelayUrl(
        val url: String,
    ) : SyncAction

    data class RemoveCustomRelayUrl(
        val url: String,
    ) : SyncAction

    /** Validates the format before persisting — if malformed, `SyncUiState.irohServicesTicketError`
     *  becomes `true` (see `SyncViewModel`). */
    data class SetIrohServicesTicket(
        val ticket: String,
    ) : SyncAction

    data object ClearIrohServicesTicket : SyncAction

    data object DismissIrohServicesTicketError : SyncAction

    /** Manual "Restart" button — shuts down the current P2P node and starts a new one from scratch
     *  (same identity/storage), LocalSend style. Escape hatch for when the live relay switch
     *  isn't enough (e.g. connection stuck after an OS physical network change). */
    data object RestartP2p : SyncAction
}
