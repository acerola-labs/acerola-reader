package br.acerola.comic.module.main.transferlog

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.rounded.SwapVert
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.RoundIcon
import br.acerola.comic.common.ux.component.ScreenHeader
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.component.Spinner
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.component.colors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.sync.describeEntry
import br.acerola.comic.module.main.sync.formatLogTimestamp
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.module.main.transferlog.state.TransferLogUiState
import br.acerola.comic.ui.R

/**
 * Full-screen P2P transfer history — same format as LocalSend: its own bar
 * with a back arrow, "Refresh"/"Clear" buttons right below, then the list. A real screen
 * (not a dialog/sheet/accordion squeezed into the Network screen) makes scrolling obvious (the
 * next item cut off at the bottom edge already hints there's more) and never competes for space
 * with the rest of the Network tab's UI.
 */
@Composable
fun Main.TransferLog.Template.Screen(
    onBack: () -> Unit,
    viewModel: TransferLogViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsState()

    TransferLogScreenContent(
        uiState = uiState,
        onBack = onBack,
        onRefresh = viewModel::refresh,
        onClear = viewModel::clear,
    )
}

// Brand page (the desktop's transfers card, full screen here like LocalSend): header with back,
// "Refresh" and "Clear" as pills, then one surface row per entry.
@Composable
private fun TransferLogScreenContent(
    uiState: TransferLogUiState,
    onBack: () -> Unit,
    onRefresh: () -> Unit,
    onClear: () -> Unit,
) {
    var showClearDialog by remember { mutableStateOf(false) }

    Column(modifier = Modifier.fillMaxSize().background(AcerolaExtendedTheme.colors.background)) {
        Acerola.Component.ScreenHeader(
            eyebrow = stringResource(id = R.string.label_sync_activity),
            title = stringResource(id = R.string.title_sync_activity_log),
            lead = stringResource(id = R.string.description_sync_activity_log),
            onBack = onBack,
        )

        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_sync_activity_log_refresh),
                onClick = onRefresh,
                variant = ButtonVariant.Secondary,
                icon = Icons.Default.Refresh,
            )
            if (uiState.entries.isNotEmpty()) {
                Acerola.Component.Button(
                    text = stringResource(id = R.string.action_sync_activity_log_clear),
                    onClick = { showClearDialog = true },
                    variant = ButtonVariant.Destructive,
                    icon = Icons.Default.Delete,
                )
            }
        }

        TransferLogList(entries = uiState.entries, modifier = Modifier.fillMaxSize())
    }

    Acerola.Component.Dialog(
        show = showClearDialog,
        onDismiss = { showClearDialog = false },
        title = stringResource(id = R.string.title_sync_activity_log_clear_confirm),
        confirmButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_sync_activity_log_clear_confirm),
                onClick = {
                    onClear()
                    showClearDialog = false
                },
                variant = ButtonVariant.Destructive,
            )
        },
        dismissButtonContent = {
            Acerola.Component.DialogButton(
                text = stringResource(id = R.string.action_cancel),
                onClick = { showClearDialog = false },
            )
        },
    ) {
        Text(text = stringResource(id = R.string.description_sync_activity_log_clear_confirm))
    }
}

@Composable
private fun TransferLogList(
    entries: List<TransferLogEntry>,
    modifier: Modifier = Modifier,
) {
    if (entries.isEmpty()) {
        Box(modifier = modifier, contentAlignment = Alignment.Center) {
            Acerola.Component.EmptyState(
                title = stringResource(id = R.string.label_sync_activity_log_empty),
                icon = Icons.Rounded.SwapVert,
            )
        }
    } else {
        LazyColumn(
            modifier = modifier,
            contentPadding = PaddingValues(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            items(items = entries, key = { it.id }) { entry ->
                TransferLogCard(entry = entry)
            }
        }
    }
}

/** LocalSend-style mini card: square avatar with the status icon + title/subtitle on
 *  two lines, instead of a small icon + text + date crammed into a single line (the accordion's
 *  old format, unreadable on a screen dedicated to this). */
@Composable
private fun TransferLogCard(entry: TransferLogEntry) {
    // State as a round icon in its tone: success, error, or a spinner while it runs.
    val tone =
        when (entry.state) {
            LogState.SUCCESS -> Tone.Success
            LogState.ERROR -> Tone.Destructive
            LogState.IN_PROGRESS -> Tone.Neutral
        }

    Acerola.Component.SettingRow(
        title = describeEntry(entry),
        description = formatLogTimestamp(entry.timestamp),
        leading = {
            when (entry.state) {
                LogState.IN_PROGRESS ->
                    Box(
                        modifier = Modifier.size(SizeTokens.ClickTargetSmall).clip(AcerolaShapes.Full).background(tone.colors().container),
                        contentAlignment = Alignment.Center,
                    ) {
                        Acerola.Component.Spinner(color = MaterialTheme.colorScheme.primary)
                    }
                LogState.SUCCESS -> Acerola.Component.RoundIcon(icon = Icons.Default.CheckCircle, tone = tone)
                LogState.ERROR -> Acerola.Component.RoundIcon(icon = Icons.Default.Error, tone = tone)
            }
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TransferLogListPreview() {
    AcerolaTheme {
        TransferLogList(
            entries =
                listOf(
                    TransferLogEntry(id = 1, kind = "files", status = "complete", state = LogState.SUCCESS),
                    TransferLogEntry(
                        id = 2,
                        kind = "history",
                        status = "error",
                        state = LogState.ERROR,
                        message = "conexão perdida",
                    ),
                ),
        )
    }
}

@Preview(name = "Empty", showBackground = true)
@Composable
private fun TransferLogListEmptyPreview() {
    AcerolaTheme {
        TransferLogList(entries = emptyList())
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TransferLogScreenContentPreview() {
    AcerolaTheme {
        TransferLogScreenContent(
            uiState =
                TransferLogUiState(
                    entries =
                        listOf(
                            TransferLogEntry(id = 1, kind = "files", status = "complete", state = LogState.SUCCESS),
                            TransferLogEntry(
                                id = 2,
                                kind = "history",
                                status = "error",
                                state = LogState.ERROR,
                                message = "conexão perdida",
                            ),
                        ),
                    loaded = true,
                ),
            onBack = {},
            onRefresh = {},
            onClear = {},
        )
    }
}

@Preview(name = "Empty screen", showBackground = true)
@Composable
private fun TransferLogScreenContentEmptyPreview() {
    AcerolaTheme {
        TransferLogScreenContent(
            uiState = TransferLogUiState(entries = emptyList(), loaded = true),
            onBack = {},
            onRefresh = {},
            onClear = {},
        )
    }
}
