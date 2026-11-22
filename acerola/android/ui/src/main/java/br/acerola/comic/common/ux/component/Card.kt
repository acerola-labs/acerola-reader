package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.lift
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

/**
 * Cover (`AcerolaMediaCard`): the image in a `rounded-tile` frame on `bg-surface` while it loads,
 * no border or shadow; a [footer] sits on `bg-scrim/50` with `text-on-scrim`.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun Acerola.Component.ImageCard(
    image: Painter,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onLongClick: (() -> Unit)? = null,
    title: String? = null,
    footer: String? = null,
    shape: Shape = AcerolaShapes.Tile,
) {
    Box(
        modifier =
            modifier
                .clip(shape)
                .background(AcerolaExtendedTheme.colors.surface)
                .combinedClickable(onClick = onClick, onLongClick = onLongClick),
    ) {
        Image(
            painter = image,
            contentDescription = title,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxSize(),
        )

        if (footer != null) {
            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .align(Alignment.BottomCenter)
                        .background(AcerolaColors.Scrim.copy(alpha = 0.5f))
                        .padding(horizontal = SpacingTokens.Medium, vertical = SpacingTokens.Small),
            ) {
                Text(text = footer, style = MaterialTheme.typography.labelMedium, color = AcerolaColors.OnScrim)
            }
        }
    }
}

/** Brand card (`.ac-card`): `rounded-card bg-card p-5 shadow-lift`, with an optional title. */
@Composable
fun Acerola.Component.Card(
    modifier: Modifier = Modifier,
    title: String? = null,
    onClick: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .lift(AcerolaShapes.Card)
                .clip(AcerolaShapes.Card)
                .background(AcerolaExtendedTheme.colors.card)
                .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
                .padding(SpacingTokens.ExtraLarge),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        if (title != null) {
            Text(text = title, style = MaterialTheme.typography.titleMedium, color = AcerolaExtendedTheme.colors.foreground)
        }
        ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) {
            Column(verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium), content = content)
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun CardPreview() {
    AcerolaTheme {
        Acerola.Component.Card(title = stringResource(R.string.title_settings_display_config), modifier = Modifier.padding(SpacingTokens.Large)) {
            Text(
                text = stringResource(R.string.description_config_categories),
                style = MaterialTheme.typography.bodyMedium,
                color = AcerolaExtendedTheme.colors.mutedForeground,
            )
        }
    }
}
