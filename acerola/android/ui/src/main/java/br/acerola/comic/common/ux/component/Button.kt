package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaDimens
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.raised
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * The brand button variants, the same as the desktop's `AcerolaButton`:
 * - [Primary]: `bg-primary`, the one main action of the screen.
 * - [Secondary]: `bg-surface` (`bg-card` on a surface row), every other action.
 * - [Contrast]: `bg-foreground text-background`, the confirm action of a sheet or dialog.
 * - [Destructive]: `bg-destructive-soft text-destructive`, removing or erasing.
 * - [Success]: `bg-success-soft text-success`, a confirmed state (copied, saved).
 * - [Ghost]: no fill, a quiet action next to another one.
 */
enum class ButtonVariant { Primary, Secondary, Contrast, Destructive, Success, Ghost }

private data class ButtonColors(
    val container: Color,
    val content: Color,
)

@Composable
private fun buttonColors(variant: ButtonVariant): ButtonColors {
    val brand = AcerolaExtendedTheme.colors
    return when (variant) {
        ButtonVariant.Primary -> ButtonColors(MaterialTheme.colorScheme.primary, MaterialTheme.colorScheme.onPrimary)
        ButtonVariant.Secondary -> ButtonColors(brand.raised, brand.foreground)
        ButtonVariant.Contrast -> ButtonColors(brand.foreground, brand.background)
        ButtonVariant.Destructive -> ButtonColors(brand.destructiveSoft, MaterialTheme.colorScheme.error)
        ButtonVariant.Success -> ButtonColors(brand.successContainer, brand.success)
        ButtonVariant.Ghost -> ButtonColors(Color.Transparent, brand.foreground)
    }
}

/** Pill `h-11` (44dp) with `font-black` text, the brand `.ac-btn`. */
@Composable
fun Acerola.Component.Button(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: ButtonVariant = ButtonVariant.Primary,
    enabled: Boolean = true,
    icon: ImageVector? = null,
    trailingIcon: ImageVector? = null,
    loading: Boolean = false,
) {
    Acerola.Component.ButtonSurface(
        onClick = onClick,
        modifier = modifier,
        variant = variant,
        enabled = enabled && !loading,
        contentPadding = PaddingValues(horizontal = SpacingTokens.ExtraLarge),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small, Alignment.CenterHorizontally),
        ) {
            when {
                loading -> Acerola.Component.Spinner(size = SizeTokens.IconExtraSmall)
                icon != null -> Icon(imageVector = icon, contentDescription = null, modifier = Modifier.size(SizeTokens.IconSmall))
            }
            Text(
                text = text,
                style = MaterialTheme.typography.labelLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (trailingIcon != null) {
                Icon(imageVector = trailingIcon, contentDescription = null, modifier = Modifier.size(SizeTokens.IconSmall))
            }
        }
    }
}

/** Round icon-only button (`.ac-btn--icon`, 44dp); [contentDescription] is its accessible name. */
@Composable
fun Acerola.Component.IconButton(
    icon: ImageVector,
    contentDescription: String?,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: ButtonVariant = ButtonVariant.Secondary,
    enabled: Boolean = true,
    size: Dp = AcerolaDimens.ControlHeight,
) {
    Acerola.Component.ButtonSurface(
        onClick = onClick,
        modifier = modifier.size(size),
        variant = variant,
        enabled = enabled,
        contentPadding = PaddingValues(),
    ) {
        Icon(imageVector = icon, contentDescription = contentDescription, modifier = Modifier.size(SizeTokens.IconSmall))
    }
}

/**
 * Round button over a cover or page art: `bg-scrim/50` with `text-on-scrim`, the same in both
 * modes on purpose so it reads on any image.
 */
@Composable
fun Acerola.Component.ScrimButton(
    icon: ImageVector,
    contentDescription: String?,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
) {
    Surface(
        onClick = onClick,
        enabled = enabled,
        shape = AcerolaShapes.Full,
        color = AcerolaColors.Scrim.copy(alpha = 0.5f),
        contentColor = AcerolaColors.OnScrim,
        modifier = modifier.size(AcerolaDimens.ControlHeight).alpha(if (enabled) 1f else 0.6f),
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(imageVector = icon, contentDescription = contentDescription, modifier = Modifier.size(SizeTokens.IconSmall))
        }
    }
}

/** The pill every button variant shares: fill, pressed scale (0.98) and disabled opacity (0.6). */
@Composable
fun Acerola.Component.ButtonSurface(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: ButtonVariant = ButtonVariant.Secondary,
    enabled: Boolean = true,
    contentPadding: PaddingValues = PaddingValues(horizontal = SpacingTokens.ExtraLarge),
    content: @Composable () -> Unit,
) {
    val colors = buttonColors(variant)
    val interactionSource = remember { MutableInteractionSource() }
    val pressed by interactionSource.collectIsPressedAsState()

    Surface(
        onClick = onClick,
        enabled = enabled,
        shape = AcerolaShapes.Full,
        color = colors.container,
        contentColor = colors.content,
        interactionSource = interactionSource,
        modifier =
            modifier
                .defaultMinSize(minWidth = AcerolaDimens.ControlHeight, minHeight = AcerolaDimens.ControlHeight)
                .graphicsLayer {
                    val scale = if (pressed) 0.98f else 1f
                    scaleX = scale
                    scaleY = scale
                    alpha = if (enabled) 1f else 0.6f
                },
    ) {
        Box(contentAlignment = Alignment.Center, modifier = Modifier.padding(contentPadding)) {
            CompositionLocalProvider(LocalContentColor provides colors.content) { content() }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ButtonVariantsPreview() {
    AcerolaTheme {
        Column(
            modifier = Modifier.padding(SpacingTokens.Large),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            ButtonVariant.entries.forEach { variant ->
                Acerola.Component.Button(text = variant.name, onClick = {}, variant = variant, icon = Icons.Default.Add)
            }
            Acerola.Component.IconButton(icon = Icons.Default.Star, contentDescription = null, onClick = {})
            Acerola.Component.ScrimButton(icon = Icons.Default.Star, contentDescription = null, onClick = {})
        }
    }
}
