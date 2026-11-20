package br.acerola.comic.config.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities

/** `true` when the active network is cellular (mobile data) — used to ask for confirmation before
 *  triggering a P2P sync (see `SyncViewModel`), since it may consume the user's data
 *  allowance. Requires `ACCESS_NETWORK_STATE` (declared in `app/src/main/AndroidManifest.xml`). */
fun isOnCellularConnection(context: Context): Boolean {
    val connectivityManager = context.getSystemService(ConnectivityManager::class.java) ?: return false
    val network = connectivityManager.activeNetwork ?: return false
    val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return false
    return capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)
}
