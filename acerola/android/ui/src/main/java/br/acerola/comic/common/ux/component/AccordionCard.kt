package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.lift
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.MotionTokens
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * A [SettingSection] whose body folds away: the same `rounded-card bg-card shadow-lift` card, the
 * header (round icon, `text-lg font-black` title, description) toggles [expanded] with a turning
 * chevron. `collapsible = false` keeps it always open, with no chevron.
 */
@Composable
fun Acerola.Component.AccordionCard(
    title: String,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
    description: String? = null,
    collapsible: Boolean = true,
    icon: ImageVector? = null,
    tone: Tone = Tone.Neutral,
    content: @Composable ColumnScope.() -> Unit,
) {
    val brand = AcerolaExtendedTheme.colors
    val chevronRotation by animateFloatAsState(
        targetValue = if (expanded) 180f else 0f,
        animationSpec = MotionTokens.brand(),
        label = "accordion_chevron_rotation",
    )
    val open = !collapsible || expanded

    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .lift(AcerolaShapes.Card)
                .clip(AcerolaShapes.Card)
                .background(brand.card)
                .animateContentSize(animationSpec = MotionTokens.brand()),
    ) {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .then(if (collapsible) Modifier.clickable(onClick = onToggleExpanded) else Modifier)
                    .padding(SpacingTokens.ExtraLarge),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            if (icon != null) {
                Acerola.Component.RoundIcon(icon = icon, tone = tone)
            }

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = title,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.titleMedium,
                    color = brand.foreground,
                )
                if (description != null) {
                    Text(
                        text = description,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        style = MaterialTheme.typography.bodyMedium,
                        color = brand.mutedForeground,
                    )
                }
            }

            if (collapsible) {
                Icon(
                    imageVector = Icons.Default.ExpandMore,
                    contentDescription = null,
                    tint = brand.mutedForeground,
                    modifier =
                        Modifier
                            .size(SizeTokens.IconMedium)
                            .graphicsLayer { rotationZ = chevronRotation },
                )
            }
        }

        if (open) {
            ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) {
                Column(
                    modifier = Modifier.padding(start = SpacingTokens.ExtraLarge, end = SpacingTokens.ExtraLarge, bottom = SpacingTokens.ExtraLarge),
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
                    content = content,
                )
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AccordionCardPreview() {
    AcerolaTheme {
        Acerola.Component.AccordionCard(
            title = "Arquivos",
            description = "Pasta e modelos de nome",
            icon = Icons.Default.Folder,
            expanded = true,
            onToggleExpanded = {},
            modifier = Modifier.padding(SpacingTokens.Large),
        ) {
            Acerola.Component.HeroButton(title = "Selecionar pasta", icon = Icons.Default.Folder)
        }
    }
}
