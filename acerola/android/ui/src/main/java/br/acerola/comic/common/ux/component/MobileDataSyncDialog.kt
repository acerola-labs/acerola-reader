package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

/**
 * Dialog shared by ANY screen that fires a P2P sync while the
 * device is on mobile data (see `MobileDataSyncGate`/`MobileDataSyncViewModel`) — mounted
 * only once at the app root (`br.acerola.comic.common.activity.BaseActivity`), not inside
 * a specific screen, since Home/History/Comic/Remote library/Sync can each fire this
 * confirmation through their own path.
 */
@Composable
fun Acerola.Component.MobileDataSyncDialog(
    onConfirm: (remember: Boolean) -> Unit,
    onCancel: () -> Unit,
) {
    var rememberChoice by remember { mutableStateOf(false) }

    Acerola.Component.Dialog(
        show = true,
        onDismiss = onCancel,
        title = stringResource(id = R.string.title_mobile_data_sync_confirm),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_mobile_data_sync_confirm),
                onClick = { onConfirm(rememberChoice) },
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = onCancel,
            )
        },
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large)) {
            Text(text = stringResource(id = R.string.description_mobile_data_sync_confirm))

            Acerola.Component.SettingRow(
                title = stringResource(id = R.string.label_mobile_data_sync_remember),
                onClick = { rememberChoice = !rememberChoice },
                control = { Acerola.Component.Switch(checked = rememberChoice, onCheckedChange = { rememberChoice = it }) },
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun MobileDataSyncDialogPreview() {
    AcerolaTheme {
        Acerola.Component.MobileDataSyncDialog(onConfirm = {}, onCancel = {})
    }
}
