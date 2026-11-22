package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.lift
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.theme.raised
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/** Tone of a row's round icon: neutral, or a soft background with the tone's text. */
enum class Tone { Neutral, Primary, Success, Warning, Info, Destructive }

data class ToneColors(
    val container: Color,
    val content: Color,
)

/** `bg-*-soft text-*` for a tone; neutral is the raised fill (`bg-card` on a surface row). */
@Composable
fun Tone.colors(): ToneColors {
    val brand = AcerolaExtendedTheme.colors
    return when (this) {
        Tone.Neutral -> ToneColors(brand.raised, brand.foreground)
        Tone.Primary -> ToneColors(brand.primarySoft, MaterialTheme.colorScheme.primary)
        Tone.Success -> ToneColors(brand.successContainer, brand.success)
        Tone.Warning -> ToneColors(brand.warningContainer, brand.warning)
        Tone.Info -> ToneColors(brand.infoSoft, brand.info)
        Tone.Destructive -> ToneColors(brand.destructiveSoft, MaterialTheme.colorScheme.error)
    }
}

/**
 * A group of settings, the desktop's `AcerolaSettingSection`: `rounded-card bg-card p-5
 * shadow-lift`, a `text-lg font-black` title, an optional lead and header [actions].
 */
@Composable
fun Acerola.Component.SettingSection(
    title: String,
    modifier: Modifier = Modifier,
    lead: String? = null,
    actions: @Composable (RowScope.() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .lift(AcerolaShapes.Card)
                .clip(AcerolaShapes.Card)
                .background(AcerolaExtendedTheme.colors.card)
                .padding(SpacingTokens.ExtraLarge),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall)) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    color = AcerolaExtendedTheme.colors.foreground,
                )
                if (lead != null) {
                    Text(
                        text = lead,
                        style = MaterialTheme.typography.bodyMedium,
                        color = AcerolaExtendedTheme.colors.mutedForeground,
                    )
                }
            }
            if (actions != null) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                    verticalAlignment = Alignment.CenterVertically,
                    content = actions,
                )
            }
        }

        ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) {
            Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium), content = content)
        }
    }
}

/**
 * One setting, the desktop's `AcerolaSettingRow`: `rounded-tile bg-surface p-4`, a round 40dp
 * icon in its [tone], the title (`font-black`) and description, a [control] on the right
 * (switch, button) and extra [content] under it. Controls inside lift to `bg-card`.
 */
@Composable
fun Acerola.Component.SettingRow(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    icon: ImageVector? = null,
    tone: Tone = Tone.Neutral,
    onClick: (() -> Unit)? = null,
    enabled: Boolean = true,
    control: @Composable (() -> Unit)? = null,
    content: @Composable (ColumnScope.() -> Unit)? = null,
) {
    Acerola.Component.SettingRow(
        title = title,
        modifier = modifier,
        description = description,
        onClick = onClick,
        enabled = enabled,
        control = control,
        content = content,
        leading =
            icon?.let {
                { Acerola.Component.RoundIcon(icon = it, tone = tone) }
            },
    )
}

@Composable
fun Acerola.Component.SettingRow(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    onClick: (() -> Unit)? = null,
    enabled: Boolean = true,
    leading: @Composable (() -> Unit)?,
    control: @Composable (() -> Unit)? = null,
    content: @Composable (ColumnScope.() -> Unit)? = null,
) {
    val brand = AcerolaExtendedTheme.colors
    ProvideSurfaceLevel(AcerolaSurfaceLevel.Surface) {
        Column(
            modifier =
                modifier
                    .fillMaxWidth()
                    .clip(AcerolaShapes.Tile)
                    .background(brand.surface)
                    .then(if (onClick != null) Modifier.clickable(enabled = enabled, onClick = onClick) else Modifier)
                    .padding(SpacingTokens.Large),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
            ) {
                leading?.invoke()
                Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(
                        text = title,
                        style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
                        color = brand.foreground,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                    if (description != null) {
                        Text(
                            text = description,
                            style = MaterialTheme.typography.bodyMedium,
                            color = brand.mutedForeground,
                        )
                    }
                }
                control?.invoke()
            }
            content?.invoke(this)
        }
    }
}

/** Round icon badge (`size-10 rounded-full`) in a [tone]. */
@Composable
fun Acerola.Component.RoundIcon(
    icon: ImageVector,
    modifier: Modifier = Modifier,
    tone: Tone = Tone.Neutral,
    size: Dp = SizeTokens.ClickTargetSmall,
) {
    val colors = tone.colors()
    Box(
        modifier =
            modifier
                .size(size)
                .clip(AcerolaShapes.Full)
                .background(colors.container),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = colors.content,
            modifier = Modifier.size(maxOf(SizeTokens.IconSmall, size * 0.45f)),
        )
    }
}

/** Small pill label (`.ac-badge`) in a [tone]; neutral is `bg-surface`. */
@Composable
fun Acerola.Component.Badge(
    text: String,
    modifier: Modifier = Modifier,
    tone: Tone = Tone.Neutral,
) {
    val colors =
        if (tone == Tone.Neutral) {
            ToneColors(AcerolaExtendedTheme.colors.surface, AcerolaExtendedTheme.colors.foreground)
        } else {
            tone.colors()
        }
    Text(
        text = text,
        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
        color = colors.content,
        maxLines = 1,
        modifier =
            modifier
                .clip(AcerolaShapes.Full)
                .background(colors.container)
                .padding(horizontal = 10.dp, vertical = SpacingTokens.ExtraSmall),
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SettingSectionPreview() {
    AcerolaTheme {
        Box(modifier = Modifier.background(AcerolaExtendedTheme.colors.background).padding(SpacingTokens.Large)) {
            Acerola.Component.SettingSection(title = "Arquivos", lead = "Onde ficam os seus quadrinhos") {
                Acerola.Component.SettingRow(
                    title = "Pasta da biblioteca",
                    description = "/storage/Quadrinhos",
                    icon = Icons.Default.Folder,
                    control = { Acerola.Component.Button(text = "Trocar", onClick = {}, variant = ButtonVariant.Secondary) },
                )
                Acerola.Component.SettingRow(
                    title = "Sincronizar",
                    description = "Procura arquivos novos",
                    icon = Icons.Default.Sync,
                    tone = Tone.Primary,
                    control = { Acerola.Component.Switch(checked = true, onCheckedChange = {}) },
                )
            }
        }
    }
}
