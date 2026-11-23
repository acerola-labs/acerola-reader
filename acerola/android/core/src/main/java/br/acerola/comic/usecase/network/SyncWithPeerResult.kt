package br.acerola.comic.usecase.network

/**
 * Result of [SyncComicWithPeerUseCase]/[SyncHistoryEntryWithPeerUseCase]/
 * [SyncHistoryWithPeerUseCase] — three states instead of a `Boolean` because the caller must
 * tell "did nothing because of an error" (shows a snackbar) from "did nothing because the
 * user declined the mobile data dialog" (not an error at all, shows nothing extra — the
 * dialog itself already communicated the choice).
 */
enum class SyncWithPeerResult {
    /** `peerId` is no longer paired — nothing was triggered. */
    NOT_PAIRED,

    /** User declined using mobile data in the dialog (see `MobileDataSyncGate`) — nothing was
     *  triggered, and the caller must not set any "syncing" state nor show
     *  an error. */
    DECLINED_MOBILE_DATA,

    /** Session actually triggered — the caller may set the "syncing" spinner and
     *  wait for the matching [br.acerola.comic.service.network.P2pEvent]. */
    STARTED,
}
