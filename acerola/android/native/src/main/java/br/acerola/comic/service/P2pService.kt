package br.acerola.comic.service

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.os.Build
import android.util.Log
import p2p.CoverBrowseProvider
import p2p.FfiNetworkMode
import p2p.FfiPeerAddr
import p2p.FfiRelaySettings
import p2p.FfiSyncDirection
import p2p.FileSyncProvider
import p2p.HistorySyncProvider
import p2p.P2pCallback
import p2p.P2pNode
import p2p.RelayTicketException
import p2p.SecureBlobStore
import java.io.Closeable

enum class NetworkMode {
    LOCAL,
    RELAY,
}

/** Combinable relay configuration to pass to `P2PNode::new` — mirrors
 *  `FfiRelaySettings` (generated via UniFFI), same pattern as [NetworkMode]/[SyncDirection]:
 *  the `Ffi*` types do not leak beyond this `:native` module boundary. See
 *  `br.acerola.comic.config.preference.RelayPreference.RelaySettings`, the persisted source
 *  of these values. */
data class RelaySettings(
    val useAcerolaRelay: Boolean,
    val useIrohPublicNetwork: Boolean,
    val customRelayUrls: List<String>,
)

private fun RelaySettings.toFfi(): FfiRelaySettings =
    FfiRelaySettings(
        useAcerolaRelay = useAcerolaRelay,
        useIrohPublicNetwork = useIrohPublicNetwork,
        customRelayUrls = customRelayUrls,
    )

/** Explicit direction of a [syncComic] — `PUSH` sends the comic to the peer, `PULL` pulls it.
 *  Mirrors `FfiSyncDirection` (generated via UniFFI); same pattern as [NetworkMode]/[FfiNetworkMode]
 *  right above, so `Ffi*` does not leak beyond this `:native` boundary. */
enum class SyncDirection {
    PUSH,
    PULL,
}

private fun SyncDirection.toFfi(): FfiSyncDirection =
    when (this) {
        SyncDirection.PUSH -> FfiSyncDirection.PUSH
        SyncDirection.PULL -> FfiSyncDirection.PULL
    }

data class PeerAddress(
    val id: String,
    val deviceId: String?,
    val addrs: ByteArray,
    /** Only filled by the return of [getPairedPeers] — `null` in other uses (address only
     *  for dialing, e.g. [connect]/[syncComic]). */
    val deviceName: String? = null,
)

data class ConnectedPeerInfo(
    val peerId: String,
    val alpns: List<ByteArray>,
    val deviceName: String?,
)

class P2pService(
    context: Context,
    relaySettings: RelaySettings,
    /** Custom nickname saved by the user (DataStore, `DeviceAliasPreference`) — `null` when
     *  never set, and the default `Build.MODEL` is used. Only read here, when building the node;
     *  renaming after startup is done at runtime via [setLocalDeviceName], without
     *  rebuilding anything. */
    deviceNameOverride: String?,
    secureStore: SecureBlobStore,
    historyProvider: HistorySyncProvider,
    fileProvider: FileSyncProvider,
    coverProvider: CoverBrowseProvider,
    private val eventListener: (event: String, data: String) -> Unit,
) : Closeable {
    private val callbackHandler =
        object : P2pCallback {
            override fun onEvent(
                event: String,
                data: String,
            ) {
                Log.d("P2pService", "Event received: $event, Data: $data")
                eventListener(event, data)
            }
        }

    // `P2pNode(...)` calls `P2PNode::new()` in Rust, which does `runtime.block_on(...)` — a
    // synchronous blocking call (see TODO.md, critical ANR item). Before, this ran directly
    // in `init{}`, so it blocked ANY thread that instantiated `P2pService` — nowadays,
    // the main thread, because Hilt resolves this singleton for the first time when building
    // `HomeViewModel`/`P2pUseCase` on the first screen. Replaced by `lazy` (thread-safe by
    // default) triggered on a dedicated thread right below: the `P2pService` constructor now
    // returns immediately, and the heavy construction runs in the background. Whoever calls a method
    // of this service before that construction finishes still blocks (the `Lazy` syncs via a lock), but
    // that is no longer guaranteed every time — only when the call actually races the warm-up.
    private val p2pNode: P2pNode by lazy(LazyThreadSafetyMode.SYNCHRONIZED) {
        // Only used by Rust to migrate identity/peers/trust from the old plain-text format
        // into `SecureBlobStore` once — see `storage.rs`/`trust_store.rs`.
        val legacyDataDir = context.filesDir.resolve("p2p").absolutePath
        // Local content-addressed blob store (`iroh-blobs-adapter`) — cache separate from the
        // user's library tree (SAF), reclaimed by the lib's internal GC, not by the app.
        val blobsDir = context.filesDir.resolve("blobs").absolutePath
        val appVersion =
            runCatching {
                context.packageManager.getPackageInfo(context.packageName, 0).versionName
            }.getOrNull() ?: "unknown"
        P2pNode(
            callbackHandler,
            legacyDataDir,
            blobsDir,
            relaySettings.toFfi(),
            deviceNameOverride ?: Build.MODEL,
            appVersion,
            secureStore,
            historyProvider,
            fileProvider,
            coverProvider,
        )
    }

    // Iroh (the QUIC transport under `p2pNode`) detects network changes on its own on
    // most systems, but not on Android: the OS does not expose interface monitoring to
    // native code, only to Java/Kotlin via `ConnectivityManager`. Without forwarding this event, a
    // Wi-Fi/mobile data switch (or leaving Doze) leaves the transport reusing already dead
    // connections, and every browse/sync to the same peer times out until the app is restarted.
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    private val networkCallback =
        object : ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                Log.d("P2pService", "Network available, notifying P2P transport")
                p2pNode.notifyNetworkChange()
            }

            override fun onLost(network: Network) {
                Log.d("P2pService", "Network lost, notifying P2P transport")
                p2pNode.notifyNetworkChange()
            }

            override fun onCapabilitiesChanged(
                network: Network,
                networkCapabilities: NetworkCapabilities,
            ) {
                p2pNode.notifyNetworkChange()
            }
        }

    init {
        Thread({ p2pNode }, "P2pNode-warmup").apply { isDaemon = true }.start()
        connectivityManager.registerDefaultNetworkCallback(networkCallback)
    }

    fun getLocalId(): String = p2pNode.getLocalId()

    /** LocalSend-style custom nickname (instead of the default `Build.MODEL`) — applied right away
     *  to the P2P node (the next handshake already uses the new name, no app restart needed). */
    fun setLocalDeviceName(name: String) {
        Log.d("P2pService", "Renaming local device to: $name")
        p2pNode.setLocalDeviceName(name)
    }

    fun getLocalAddress(): PeerAddress {
        val addr = p2pNode.getLocalAddr()
        return PeerAddress(
            id = addr.id,
            deviceId = addr.deviceId,
            addrs = addr.addrs,
        )
    }

    fun connect(
        peerAddress: PeerAddress,
        alpn: ByteArray,
    ) {
        Log.d("P2pService", "Connecting to peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.connect(ffiAddr, alpn)
    }

    /** Syncs a single comic with the peer, in the explicit [direction] chosen by the
     *  user — `PUSH` sends the comic to the peer, `PULL` pulls it. `chapters` (chapter
     *  labels, empty by default = whole comic) scopes the session to a subset of
     *  chapters — used by the "Send" button of the multi-selection/three-dot menu. */
    fun syncComic(
        peerAddress: PeerAddress,
        comicName: String,
        direction: SyncDirection,
        chapters: List<String> = emptyList(),
    ) {
        Log.d("P2pService", "Syncing comic '$comicName' with peer: ${peerAddress.id} ($direction)")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.syncComic(ffiAddr, comicName, direction.toFfi(), chapters)
    }

    /** Pushes the progress + "read" markers of the selected chapter(s)
     *  (`chapterSorts`) of ONE single comic to the peer — lighter than [syncHistory]
     *  (whole library). Progress via the `sync:history-entry:*` events. */
    fun syncHistoryEntry(
        peerAddress: PeerAddress,
        comicName: String,
        chapterSorts: List<String>,
    ) {
        Log.d("P2pService", "Syncing history entry for '$comicName' with peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.syncHistoryEntry(ffiAddr, comicName, chapterSorts)
    }

    /** Requests the list of comics (name + chapter count) from the peer's library, without
     *  syncing anything — the result arrives via the `browse:library:result`/`error` event. */
    fun browseLibrary(peerAddress: PeerAddress) {
        Log.d("P2pService", "Browsing library of peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.browseLibrary(ffiAddr)
    }

    /** Fetches the cover of ONE remote comic — `knownVersion` is the version already cached locally
     *  (`null` if this cover was never fetched before). The result arrives via the
     *  `browse:cover:result`/`browse:cover:error` event, with the saved local path in the payload
     *  (`path`) when `status == "changed"`. */
    fun browseCover(
        peerAddress: PeerAddress,
        comicName: String,
        knownVersion: Long?,
    ) {
        Log.d("P2pService", "Browsing cover of '$comicName' from peer: ${peerAddress.id}")
        val ffiAddr =
            FfiPeerAddr(
                id = peerAddress.id,
                deviceId = peerAddress.deviceId,
                addrs = peerAddress.addrs,
            )
        p2pNode.browseCover(ffiAddr, comicName, knownVersion)
    }

    fun switchToLocal() {
        p2pNode.switchToLocal()
    }

    fun switchToRelay() {
        p2pNode.switchToRelay()
    }

    fun getMode(): NetworkMode =
        when (p2pNode.getMode()) {
            FfiNetworkMode.LOCAL -> NetworkMode.LOCAL
            FfiNetworkMode.RELAY -> NetworkMode.RELAY
        }

    fun getConnectedPeers(): Map<String, List<ByteArray>> = p2pNode.getConnectedPeers()

    fun getConnectedPeersWithInfo(): List<ConnectedPeerInfo> =
        p2pNode.getConnectedPeersWithInfo().map {
            ConnectedPeerInfo(peerId = it.peerId, alpns = it.alpns, deviceName = it.deviceName)
        }

    /**
     * Every peer ever paired (TOFU), with its last known address — persists across app
     * restarts. The handshake connection itself only lasts a few seconds (PING/PONG/DeviceInfo
     * exchange, then closes), so this — not [getConnectedPeers] — is the right source for
     * "paired devices" in the UI.
     */
    fun getPairedPeers(): List<PeerAddress> =
        p2pNode.getPairedPeers().map {
            PeerAddress(id = it.id, deviceId = it.deviceId, addrs = it.addrs, deviceName = it.deviceName)
        }

    /** Unpairs a peer — removes it from trust and from the known address cache. A future
     *  reconnection from that peer goes through the same TOFU flow as a new device. */
    fun removePairedPeer(id: String) {
        Log.d("P2pService", "Removing paired peer: $id")
        p2pNode.removePairedPeer(id)
    }

    /** `true` if the user already pasted and saved a ticket of their own account on
     *  `services.iroh.computer` — never returns the value itself (it is a real credential). */
    fun hasIrohServicesTicket(): Boolean = p2pNode.hasIrohServicesTicket()

    /** Validates the format before persisting. Does not apply the new config by itself — the caller
     *  also invokes [applyRelaySettings] right after (see `P2pUseCase`/`SyncViewModel`).
     *  @throws IllegalArgumentException if the ticket is malformed. */
    fun setIrohServicesTicket(ticket: String) {
        try {
            p2pNode.setIrohServicesTicket(ticket)
        } catch (error: RelayTicketException.Invalid) {
            throw IllegalArgumentException(error.reason)
        }
    }

    /** Removes the saved ticket — used when the user turns the source off or replaces it with a new one. */
    fun clearIrohServicesTicket() {
        p2pNode.clearIrohServicesTicket()
    }

    /** Re-reads the combinable relay config + the vault ticket and applies it to the ALREADY LIVE node
     *  (`insert_relay`/`remove_relay` on an `Endpoint` that keeps running, without dropping
     *  active connections) — called after ANY change in the relay sources (Acerola/Iroh
     *  toggle, add/remove of a custom URL, save/remove ticket). Before this existed,
     *  changing the relay config only took effect on the NEXT app boot.
     *  @throws IllegalArgumentException if the resulting config is invalid (e.g. malformed custom
     *  URL). */
    fun applyRelaySettings(relaySettings: RelaySettings) {
        try {
            p2pNode.applyRelaySettings(relaySettings.toFfi())
        } catch (error: RelayTicketException.Invalid) {
            throw IllegalArgumentException(error.reason)
        }
    }

    /** Shuts down the current P2P node and brings up a new one with the same identity/storage/handlers —
     *  LocalSend style. Unlike [applyRelaySettings] (which only switches the relay mode on an
     *  `Endpoint` that stays alive), this drops ALL active connections for a few
     *  seconds; it exists for the manual "Restart" button on the Network screen, to unstick a
     *  connection stuck after a physical OS network switch that [notifyNetworkChange]
     *  alone did not fix. `p2pNode` remains the SAME FFI object afterwards — Rust swaps
     *  the node underneath (`P2PNode::restart`, internal `RwLock`), it does not recreate the handle. */
    fun restart(relaySettings: RelaySettings) {
        Log.d("P2pService", "Restarting p2p node")
        p2pNode.restart(relaySettings.toFfi())
    }

    fun shutdown() {
        runCatching { connectivityManager.unregisterNetworkCallback(networkCallback) }
        p2pNode.destroy()
    }

    override fun close() {
        shutdown()
    }
}
