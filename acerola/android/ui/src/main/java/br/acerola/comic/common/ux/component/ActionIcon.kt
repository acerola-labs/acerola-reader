package br.acerola.comic.common.ux.component

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import br.acerola.comic.common.ux.Acerola

/**
 * Icon-only action, a round 44dp [Acerola.Component.IconButton]: secondary by default (`bg-surface`,
 * `bg-card` on a surface row), [ButtonVariant.Destructive] / [ButtonVariant.Success] for soft tones.
 */
@Composable
fun Acerola.Component.ActionIcon(
    icon: ImageVector,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    contentDescription: String? = null,
    variant: ButtonVariant = ButtonVariant.Secondary,
) {
    Acerola.Component.IconButton(
        icon = icon,
        contentDescription = contentDescription,
        onClick = onClick,
        modifier = modifier,
        variant = variant,
        enabled = enabled,
    )
}
