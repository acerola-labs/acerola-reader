package br.acerola.comic.common.ux.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * Action row of the "three dots" menus (comic and chapter), the desktop's menu tile: a round icon
 * in its [tone], the `font-black` title and an optional subtitle, on the sheet's own background.
 * A [Tone.Destructive] action also paints the title in `destructive`.
 */
@Composable
fun Acerola.Component.ActionListItem(
    icon: ImageVector,
    title: String,
    onClick: () -> Unit,
    subtitle: String? = null,
    tone: Tone = Tone.Neutral,
) {
    val brand = AcerolaExtendedTheme.colors
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp)
                .clip(AcerolaShapes.Tile)
                .clickable(onClick = onClick)
                .padding(horizontal = SpacingTokens.Medium, vertical = SpacingTokens.Small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        Acerola.Component.RoundIcon(icon = icon, tone = tone)
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Black),
                color = if (tone == Tone.Destructive) MaterialTheme.colorScheme.error else brand.foreground,
            )
            if (subtitle != null) {
                Text(text = subtitle, style = MaterialTheme.typography.bodyMedium, color = brand.mutedForeground)
            }
        }
    }
}
