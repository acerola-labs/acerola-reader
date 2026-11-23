package br.acerola.comic.common.activity

import android.app.Application
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Process
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.ErrorKind
import br.acerola.comic.common.ux.component.ErrorScreen
import br.acerola.comic.common.ux.theme.AcerolaTheme
import java.io.File
import kotlin.system.exitProcess

/**
 * Full-window error screen (acerola-design skill, 6.1): only the error, no navigation; its two
 * actions are the way out. It runs in its own `:error` process, so it still opens when the main
 * process is broken, and it replaces the system "app keeps stopping" dialog.
 */
class ErrorActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val kind = intent.getStringExtra(EXTRA_KIND)?.let { name -> ErrorKind.entries.firstOrNull { it.name == name } } ?: ErrorKind.Unknown
        val status = intent.getIntExtra(EXTRA_STATUS, 0)

        setContent {
            AcerolaTheme {
                Acerola.Component.ErrorScreen(
                    kind = kind,
                    status = status,
                    onHome = ::restartApp,
                    onRetry = ::restartApp,
                    onBack = ::finish,
                )
            }
        }
    }

    private fun restartApp() {
        packageManager.getLaunchIntentForPackage(packageName)?.let { launch ->
            startActivity(launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK))
        }
        finish()
    }

    companion object {
        private const val TAG = "ErrorActivity"
        private const val EXTRA_KIND = "ERROR_KIND"
        private const val EXTRA_STATUS = "ERROR_STATUS"
        private const val ERROR_PROCESS_SUFFIX = ":error"

        fun intent(
            context: Context,
            kind: ErrorKind,
            status: Int = 0,
        ): Intent =
            Intent(context, ErrorActivity::class.java)
                .putExtra(EXTRA_KIND, kind.name)
                .putExtra(EXTRA_STATUS, status)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)

        /**
         * Sends any uncaught exception of the main process to [ErrorActivity] instead of the
         * system crash dialog. Call once from `Application.onCreate`. In the `:error` process
         * itself it does nothing, so a failure there falls back to the default handler (no loop).
         */
        fun installCrashHandler(application: Application) {
            if (processName(application).endsWith(ERROR_PROCESS_SUFFIX)) return
            val fallback = Thread.getDefaultUncaughtExceptionHandler()
            Thread.setDefaultUncaughtExceptionHandler { thread, error ->
                // The stack trace still goes to logcat; the screen never shows it.
                Log.e(TAG, "Uncaught exception on ${thread.name}", error)
                try {
                    application.startActivity(intent(application, ErrorKind.Unknown))
                    Process.killProcess(Process.myPid())
                    exitProcess(10)
                } catch (_: Throwable) {
                    fallback?.uncaughtException(thread, error)
                }
            }
        }

        private fun processName(application: Application): String =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                Application.getProcessName()
            } else {
                runCatching { File("/proc/self/cmdline").readText().trim(Char(0)) }.getOrDefault(application.packageName)
            }
    }
}
