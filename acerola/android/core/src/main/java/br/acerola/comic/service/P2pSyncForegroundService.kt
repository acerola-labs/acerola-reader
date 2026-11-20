package br.acerola.comic.service

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.content.ContextCompat
import br.acerola.comic.core.R
import br.acerola.comic.util.notification.NotificationHelper
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

/**
 * "Empty" service — it does no transfer on its own, it only signals to the system that there is
 * active network work in progress (the P2P node/`P2pService` already run independently of this,
 * tied to the Hilt lifecycle). Without it, a long sync (minutes, large
 * chapters) runs with NO Doze/App Standby exemption — on vendors with their own power
 * manager (MIUI/EMUI/ColorOS) this is even more aggressive than stock Android, and can
 * suspend the connection even with the screen on (see TODO.md).
 *
 * Start/stop are triggered by `SyncViewModel` on every `syncingKeys` empty<->non-empty
 * transition — this service never decides on its own when to start or stop.
 */
@AndroidEntryPoint
class P2pSyncForegroundService : Service() {
    @Inject
    lateinit var notificationHelper: NotificationHelper

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(
        intent: Intent?,
        flags: Int,
        startId: Int,
    ): Int {
        val notification =
            notificationHelper
                .createBaseNotification(
                    getString(R.string.p2p_sync_notification_title),
                    getString(R.string.p2p_sync_notification_description),
                ).setProgress(0, 0, false)
                .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NotificationHelper.P2P_SYNC_NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
            )
        } else {
            startForeground(NotificationHelper.P2P_SYNC_NOTIFICATION_ID, notification)
        }

        return START_NOT_STICKY
    }

    override fun onDestroy() {
        notificationHelper.cancelNotification(NotificationHelper.P2P_SYNC_NOTIFICATION_ID)
        super.onDestroy()
    }

    companion object {
        /** Idempotent — calling again while already active only re-emits the same notification. */
        fun start(context: Context) {
            ContextCompat.startForegroundService(context, Intent(context, P2pSyncForegroundService::class.java))
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, P2pSyncForegroundService::class.java))
        }
    }
}
