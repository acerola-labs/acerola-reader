package br.acerola.comic.common.ux.component

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.size
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.tokens.SizeTokens

/** Brand popup menu (the desktop's dropdown): `rounded-card bg-card` with `shadow-float`, no border. */
@Composable
fun Acerola.Component.Menu(
    expanded: Boolean,
    onDismissRequest: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    DropdownMenu(
        expanded = expanded,
        onDismissRequest = onDismissRequest,
        modifier = modifier,
        shape = AcerolaShapes.Card,
        containerColor = AcerolaExtendedTheme.colors.card,
        tonalElevation = 0.dp,
        shadowElevation = 20.dp,
        content = content,
    )
}

/** One menu entry: icon and `font-bold` text; [destructive] paints both in `destructive`. */
@Composable
fun Acerola.Component.MenuItem(
    text: String,
    onClick: () -> Unit,
    icon: ImageVector? = null,
    enabled: Boolean = true,
    destructive: Boolean = false,
) {
    val brand = AcerolaExtendedTheme.colors
    val content = if (destructive) MaterialTheme.colorScheme.error else brand.foreground
    DropdownMenuItem(
        text = { Text(text = text, style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Bold)) },
        onClick = onClick,
        enabled = enabled,
        leadingIcon = icon?.let { { Icon(imageVector = it, contentDescription = null, modifier = Modifier.size(SizeTokens.IconSmall)) } },
        colors =
            MenuDefaults.itemColors(
                textColor = content,
                leadingIconColor = if (destructive) content else brand.mutedForeground,
                disabledTextColor = content.copy(alpha = 0.6f),
                disabledLeadingIconColor = brand.mutedForeground.copy(alpha = 0.6f),
            ),
    )
}
