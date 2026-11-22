package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * A setting turned on and off, the desktop's switch row: a [SettingRow] (round icon, title,
 * subtitle) with the brand [Acerola.Component.Switch] on the right; the whole row toggles. Extra
 * [content] unfolds under it while [expanded].
 */
@Composable
fun Acerola.Component.ToggleCard(
    title: String,
    active: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    enabled: Boolean = true,
    expanded: Boolean = false,
    icon: ImageVector? = null,
    content: (@Composable ColumnScope.() -> Unit)? = null,
) {
    Acerola.Component.SettingRow(
        title = title,
        description = subtitle,
        icon = icon,
        tone = if (active) Tone.Primary else Tone.Neutral,
        onClick = onClick,
        enabled = enabled,
        modifier = modifier,
        control = {
            Acerola.Component.Switch(checked = active, onCheckedChange = { onClick() }, enabled = enabled)
        },
        content =
            content?.let { body ->
                {
                    AnimatedVisibility(
                        visible = expanded,
                        enter = expandVertically() + fadeIn(),
                        exit = shrinkVertically() + fadeOut(),
                    ) {
                        Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small), content = body)
                    }
                }
            },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ToggleCardPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.padding(SpacingTokens.Large), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
            Acerola.Component.ToggleCard(
                title = "Usar o relay do Acerola",
                subtitle = "Gerenciado automaticamente",
                active = true,
                onClick = {},
                icon = Icons.Default.Wifi,
            )
            Acerola.Component.ToggleCard(
                title = "Relays próprios",
                subtitle = "2 configurados",
                active = false,
                expanded = true,
                onClick = {},
                icon = Icons.Default.Wifi,
                content = { Text(text = "https://relay-a.exemplo.com") },
            )
        }
    }
}
