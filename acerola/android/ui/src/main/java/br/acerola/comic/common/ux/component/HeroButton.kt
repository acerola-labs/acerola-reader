package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * Brand list row, the desktop's `AcerolaHeroButton`: `rounded-tile bg-surface p-4`, a round 40dp
 * icon in its [tone], the `font-black` title and a muted description, an [action] on the right and
 * an optional [bottomContent] under it. No border: the surface separates it from the card.
 */
@Composable
fun Acerola.Component.HeroButton(
    title: String,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    description: String? = null,
    tone: Tone = Tone.Neutral,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    action: @Composable (() -> Unit)? = null,
    bottomContent: @Composable (() -> Unit)? = null,
) {
    Acerola.Component.HeroButton(
        title = title,
        modifier = modifier,
        description = description,
        onClick = onClick,
        onLongClick = onLongClick,
        action = action,
        bottomContent = bottomContent,
        icon = { Acerola.Component.RoundIcon(icon = icon, tone = tone) },
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Acerola.Component.HeroButton(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    action: @Composable (() -> Unit)? = null,
    bottomContent: @Composable (() -> Unit)? = null,
    icon: @Composable () -> Unit,
) {
    val brand = AcerolaExtendedTheme.colors
    val clickableModifier =
        if (onClick != null || onLongClick != null) {
            Modifier.combinedClickable(onClick = onClick ?: {}, onLongClick = onLongClick)
        } else {
            Modifier
        }

    ProvideSurfaceLevel(AcerolaSurfaceLevel.Surface) {
        Column(
            modifier =
                modifier
                    .fillMaxWidth()
                    .clip(AcerolaShapes.Tile)
                    .background(brand.surface)
                    .then(clickableModifier),
        ) {
            Row(
                modifier = Modifier.padding(SpacingTokens.Large),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
            ) {
                icon()
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
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
                action?.invoke()
            }

            if (bottomContent != null) {
                Column(modifier = Modifier.padding(start = SpacingTokens.Large, end = SpacingTokens.Large, bottom = SpacingTokens.Large)) {
                    bottomContent()
                }
            }
        }
    }
}

/** A compact row nested in a [HeroButton]'s `bottomContent`: round icon, title, description. */
@Composable
fun Acerola.Component.HeroNestedButton(
    title: String,
    icon: ImageVector,
    modifier: Modifier = Modifier,
    description: String? = null,
    tone: Tone = Tone.Neutral,
    onClick: () -> Unit,
) {
    val brand = AcerolaExtendedTheme.colors
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .clip(AcerolaShapes.Inner)
                .clickable(onClick = onClick)
                .padding(vertical = SpacingTokens.Small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        Acerola.Component.RoundIcon(icon = icon, tone = tone)
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
                color = brand.foreground,
            )
            if (description != null) {
                Text(text = description, style = MaterialTheme.typography.bodyMedium, color = brand.mutedForeground)
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HeroButtonPreview() {
    AcerolaTheme {
        Column(modifier = Modifier.padding(SpacingTokens.Large), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
            Acerola.Component.HeroButton(title = "Hero title", description = "Description", icon = Icons.Default.Star)
            Acerola.Component.HeroButton(
                title = "Hero title",
                icon = Icons.Default.Star,
                tone = Tone.Primary,
                bottomContent = { Acerola.Component.HeroNestedButton(title = "Nested", icon = Icons.Default.Star, onClick = {}) },
            )
        }
    }
}
