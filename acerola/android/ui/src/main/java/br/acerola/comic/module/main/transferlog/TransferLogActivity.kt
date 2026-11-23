package br.acerola.comic.module.main.transferlog

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.navigation.NavGraphBuilder
import androidx.navigation.NavHostController
import androidx.navigation.compose.composable
import br.acerola.comic.common.activity.BaseActivity
import br.acerola.comic.common.navigation.Destination
import br.acerola.comic.module.main.Main
import dagger.hilt.android.AndroidEntryPoint

/**
 * Full-screen P2P transfer history, opened from the Network tab's hero button — same pattern
 * as [br.acerola.comic.module.main.remotelibrary.RemoteLibraryActivity]
 * (its own Activity, not a route inside `MainActivity`'s NavHost). No data via Intent:
 * [TransferLogViewModel] fetches the persisted history on its own.
 */
@AndroidEntryPoint
class TransferLogActivity(
    override val startDestinationRes: Int = Destination.TRANSFER_LOG.route,
) : BaseActivity() {
    override fun NavGraphBuilder.setupNavGraph(
        context: Context,
        navController: NavHostController,
    ) {
        composable(route = context.getString(Destination.TRANSFER_LOG.route)) {
            Main.TransferLog.Template.Screen(onBack = { finish() })
        }
    }

    @Composable
    override fun TopBar(navController: NavHostController) = Unit
}
