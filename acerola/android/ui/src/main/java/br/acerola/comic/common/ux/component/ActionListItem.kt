package br.acerola.comic.common.ux.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * Action row reused by the "three dots" menus (comic and chapter) — a clickable
 * `ListItem` with a transparent background to sit inside a grouping `Surface`, and an
 * inset divider below (except on the group's last item). When `iconBackground` is given,
 * the icon gets a colored chip ("outline/heroicon" style, like HeroButton) instead of
 * staying raw — used to match the equivalent action button on Desktop.
 */
@Composable
fun Acerola.Component.ActionListItem(
    icon: ImageVector,
    title: String,
    onClick: () -> Unit,
    subtitle: String? = null,
    tint: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    iconBackground: Color? = null,
    isLast: Boolean = false,
) {
    // When there's an `iconBackground`, `tint` is the chip's contrast color (e.g. onAccent) — it's only
    // for the icon, never the text, otherwise the title/subtitle inherits a color meant to sit on top
    // of the colored chip and becomes unreadable on the ListItem's neutral background.
    val titleColor =
        when {
            iconBackground != null -> MaterialTheme.colorScheme.onSurface
            tint == MaterialTheme.colorScheme.onSurfaceVariant -> MaterialTheme.colorScheme.onSurface
            else -> tint
        }
    val subtitleColor = if (iconBackground != null) MaterialTheme.colorScheme.onSurfaceVariant else tint

    ListItem(
        leadingContent = {
            if (iconBackground != null) {
                Surface(shape = AcerolaShapes.Inner, color = iconBackground, modifier = Modifier.size(SizeTokens.ClickTargetSmall)) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(imageVector = icon, contentDescription = null, tint = tint)
                    }
                }
            } else {
                Icon(imageVector = icon, contentDescription = null, tint = tint)
            }
        },
        headlineContent = { Text(text = title, color = titleColor) },
        supportingContent = subtitle?.let { { Text(text = it, color = subtitleColor) } },
        colors = ListItemDefaults.colors(containerColor = Color.Transparent),
        modifier = Modifier.clickable(onClick = onClick),
    )

    if (!isLast) {
        HorizontalDivider(
            modifier = Modifier.padding(start = SpacingTokens.ExtraGiant),
            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f),
        )
    }
}
