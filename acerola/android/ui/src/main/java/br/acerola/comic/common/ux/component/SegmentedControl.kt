package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.raised
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

private val ChipLabelStyle @Composable get() = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold, fontSize = 13.sp)

/**
 * Chip tabs (`AcerolaChipTabs`): a `bg-surface` track with N mutually exclusive chips; the
 * selected one is `bg-foreground text-background`. For switching views in the same context
 * (e.g. "My code" / "Connect"), unlike [Acerola.Component.RadioGroup], which picks a setting.
 */
@Composable
fun Acerola.Component.SegmentedControl(
    options: List<String>,
    selectedIndex: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val brand = AcerolaExtendedTheme.colors
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(AcerolaShapes.Full)
                .background(brand.raised)
                .padding(SpacingTokens.ExtraSmall),
        horizontalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        options.forEachIndexed { index, label ->
            val selected = index == selectedIndex
            val container by animateColorAsState(if (selected) brand.foreground else Color.Transparent, label = "chipTabBg")
            val content by animateColorAsState(if (selected) brand.background else brand.mutedForeground, label = "chipTabFg")

            Box(
                contentAlignment = Alignment.Center,
                modifier =
                    Modifier
                        .weight(1f)
                        .height(36.dp)
                        .clip(AcerolaShapes.Full)
                        .background(container)
                        .semantics {
                            this.selected = selected
                            role = Role.Tab
                        }.clickable(enabled = !selected) { onSelect(index) },
            ) {
                Text(text = label, style = ChipLabelStyle, color = content, textAlign = TextAlign.Center, maxLines = 1)
            }
        }
    }
}

/**
 * Standalone filter chip (`.ac-chip`, 36dp pill): selected `bg-foreground text-background`,
 * otherwise `bg-surface` (`bg-card` on a surface row) with muted text.
 */
@Composable
fun Acerola.Component.Chip(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    icon: ImageVector? = null,
    dotColor: Color? = null,
    enabled: Boolean = true,
) {
    val brand = AcerolaExtendedTheme.colors
    val container by animateColorAsState(if (selected) brand.foreground else brand.raised, label = "chipBg")
    val content by animateColorAsState(if (selected) brand.background else brand.foreground, label = "chipFg")
    val leading = icon ?: if (selected) Icons.Default.Check else null

    Row(
        modifier =
            modifier
                .height(36.dp)
                .clip(AcerolaShapes.Full)
                .background(container)
                .semantics { this.selected = selected }
                .clickable(enabled = enabled, onClick = onClick)
                .padding(horizontal = SpacingTokens.MediumLarge),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        if (dotColor != null) {
            // A user-chosen color (bookmark), shown as an 8dp dot next to its name.
            Box(modifier = Modifier.size(8.dp).clip(AcerolaShapes.Full).background(dotColor))
        }
        if (leading != null) {
            Icon(imageVector = leading, contentDescription = null, tint = content, modifier = Modifier.size(SizeTokens.IconExtraSmall))
        }
        Text(text = label, style = ChipLabelStyle, color = content, maxLines = 1)
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SegmentedControlPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.padding(SpacingTokens.Large), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
            Acerola.Component.SegmentedControl(options = listOf("Meu código", "Conectar"), selectedIndex = 0, onSelect = {})
            Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                Acerola.Component.Chip(label = "Lendo", selected = true, onClick = {})
                Acerola.Component.Chip(label = "Favoritos", selected = false, onClick = {})
            }
        }
    }
}
