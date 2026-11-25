package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.state.SyncActionVisualState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.SyncIconButton
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.SyncAnilistData(
    onRescan: () -> Unit,
    state: SyncActionVisualState = SyncActionVisualState.IDLE,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
) {
    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_config_sync_anilist),
        description = stringResource(id = R.string.description_config_sync_anilist),
        modifier = modifier,
        leading = {
            Image(
                painter = painterResource(id = R.drawable.anilist),
                contentDescription = null,
                modifier = Modifier.size(40.dp),
            )
        },
        control = {
            Acerola.Component.SyncIconButton(
                state = state,
                text = stringResource(id = R.string.action_config_sync),
                loadingText = stringResource(id = R.string.action_config_syncing),
                onClick = onRescan,
                enabled = enabled && state != SyncActionVisualState.LOADING,
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SyncAnilistDataPreview() {
    AcerolaTheme {
        Main.Config.Component.SyncAnilistData(
            onRescan = {},
        )
    }
}
