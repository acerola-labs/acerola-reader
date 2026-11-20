package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

private val AccordionShape = AcerolaShapes.Tile

// Collapsible card that groups several HeroButtons by category — same behavior as the
// desktop's AcerolaAccordionCard (see acerola-accordion-card.svelte): clickable header
// with a rotating chevron, a body whose HeroButton list hides/shows, and more than one card
// can be expanded at the same time (the caller controls that via `expanded`).
@Composable
fun Acerola.Component.AccordionCard(
    title: String,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
    description: String? = null,
    // Category identity color — same idea as the desktop's `CATEGORY_HOVER_BORDER` (chart-N),
    // except always visible instead of only on hover (there's no hover on touch): tints
    // the card border and the icon background. MaterialTheme roles instead of fixed hex because
    // they're the only tones guaranteed by every app theme (Catppuccin, Dracula, Nord...).
    accentColor: Color = MaterialTheme.colorScheme.outline,
    // `false` renders the section always open, with no chevron or header click — to
    // mix, on the same screen, sections worth seeing upfront with others dense
    // enough to stay hidden by default (same mix already used on the desktop's Network
    // screen, see `ui.collapsible` in `acerola-accordion-card.svelte`).
    collapsible: Boolean = true,
    icon: @Composable (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val chevronRotation by animateFloatAsState(targetValue = if (expanded) 90f else 0f, label = "accordion_chevron_rotation")
    val headerModifier = if (collapsible) Modifier.clickable(onClick = onToggleExpanded) else Modifier

    Surface(
        shape = AccordionShape,
        border = BorderStroke(SizeTokens.BorderThin, accentColor.copy(alpha = 0.5f)),
        color = MaterialTheme.colorScheme.surface,
        modifier =
            modifier
                .fillMaxWidth()
                .clip(AccordionShape),
    ) {
        Column(modifier = Modifier.animateContentSize()) {
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .then(headerModifier)
                        .padding(SpacingTokens.ExtraLarge),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (icon != null) {
                    Surface(
                        shape = AcerolaShapes.Inner,
                        color = accentColor.copy(alpha = 0.12f),
                        modifier = Modifier.size(SizeTokens.ClickTarget),
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            icon()
                        }
                    }

                    Spacer(modifier = Modifier.width(SpacingTokens.Large))
                }

                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = title,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        style = MaterialTheme.typography.titleMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                    )

                    if (description != null) {
                        Text(
                            text = description,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }

                if (collapsible) {
                    Spacer(modifier = Modifier.width(SpacingTokens.Small))

                    Icon(
                        imageVector = Icons.Default.ChevronRight,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier =
                            Modifier
                                .size(SizeTokens.IconMedium)
                                .graphicsLayer { rotationZ = chevronRotation },
                    )
                }
            }

            if (!collapsible || expanded) {
                HorizontalDivider(
                    modifier =
                        Modifier
                            .padding(horizontal = SpacingTokens.ExtraLarge)
                            .alpha(0.4f),
                )

                Column(
                    modifier = Modifier.padding(SpacingTokens.Medium),
                    verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
                    content = content,
                )
            }
        }
    }
}

@Composable
fun Acerola.Component.AccordionCard(
    title: String,
    icon: ImageVector,
    expanded: Boolean,
    onToggleExpanded: () -> Unit,
    modifier: Modifier = Modifier,
    description: String? = null,
    accentColor: Color = MaterialTheme.colorScheme.outline,
    collapsible: Boolean = true,
    content: @Composable ColumnScope.() -> Unit,
) {
    Acerola.Component.AccordionCard(
        title = title,
        expanded = expanded,
        onToggleExpanded = onToggleExpanded,
        modifier = modifier,
        description = description,
        accentColor = accentColor,
        collapsible = collapsible,
        icon = {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = accentColor,
                modifier = Modifier.size(SizeTokens.IconMedium),
            )
        },
        content = content,
    )
}

@Preview(name = "Collapsed - Light", showBackground = true)
@Preview(name = "Collapsed - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AccordionCardCollapsedPreview() {
    AcerolaTheme {
        Acerola.Component.AccordionCard(
            title = "Arquivos",
            icon = Icons.Default.Folder,
            accentColor = MaterialTheme.colorScheme.primary,
            expanded = false,
            onToggleExpanded = {},
        ) {
            Acerola.Component.HeroButton(title = "Selecionar pasta", icon = Icons.Default.Folder)
        }
    }
}

@Preview(name = "Expanded - Light", showBackground = true)
@Preview(name = "Expanded - Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun AccordionCardExpandedPreview() {
    AcerolaTheme {
        Acerola.Component.AccordionCard(
            title = "Aparência",
            description = "Tema e cores do app",
            icon = Icons.Default.Star,
            accentColor = MaterialTheme.colorScheme.tertiary,
            expanded = true,
            onToggleExpanded = {},
        ) {
            Acerola.Component.HeroButton(title = "Tema escuro", icon = Icons.Default.Star)
            Acerola.Component.HeroButton(title = "Cor de destaque", icon = Icons.Default.Star)
        }
    }
}
