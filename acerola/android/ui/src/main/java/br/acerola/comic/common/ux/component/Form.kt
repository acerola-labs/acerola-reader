package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.focusRing
import br.acerola.comic.common.ux.theme.AcerolaDimens
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.raised
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

/**
 * Brand field (`.ac-input`): 44dp pill in `bg-input` (`bg-card` on a surface row), no border,
 * `ring` outline while focused. An error paints it `destructive-soft`; the message goes below,
 * through [errorText].
 */
@Composable
fun Acerola.Component.TextField(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    placeholder: String? = null,
    enabled: Boolean = true,
    singleLine: Boolean = true,
    isError: Boolean = false,
    errorText: String? = null,
    leadingIcon: ImageVector? = null,
    trailing: @Composable (() -> Unit)? = null,
    visualTransformation: VisualTransformation = VisualTransformation.None,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    keyboardActions: KeyboardActions = KeyboardActions.Default,
) {
    val brand = AcerolaExtendedTheme.colors
    val interactionSource = remember { MutableInteractionSource() }
    val focused by interactionSource.collectIsFocusedAsState()
    val container = if (isError) brand.destructiveSoft else brand.raised
    val shape = if (singleLine) AcerolaShapes.Full else AcerolaShapes.Inner

    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall)) {
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            enabled = enabled,
            singleLine = singleLine,
            textStyle = MaterialTheme.typography.bodyLarge.copy(color = brand.foreground),
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            visualTransformation = visualTransformation,
            keyboardOptions = keyboardOptions,
            keyboardActions = keyboardActions,
            interactionSource = interactionSource,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .alpha(if (enabled) 1f else 0.6f)
                    .focusRing(focused, shape)
                    .clip(shape)
                    .background(container)
                    .defaultMinSize(minHeight = AcerolaDimens.ControlHeight),
            decorationBox = { innerTextField ->
                Row(
                    modifier =
                        Modifier.padding(
                            start = SpacingTokens.ExtraLarge - SpacingTokens.ExtraSmall / 2,
                            end = if (trailing != null) SpacingTokens.ExtraSmall else SpacingTokens.ExtraLarge,
                            top = if (singleLine) 0.dp else SpacingTokens.Medium,
                            bottom = if (singleLine) 0.dp else SpacingTokens.Medium,
                        ),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                ) {
                    if (leadingIcon != null) {
                        Icon(
                            imageVector = leadingIcon,
                            contentDescription = null,
                            tint = brand.mutedForeground,
                            modifier = Modifier.size(SizeTokens.IconSmall),
                        )
                    }
                    Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                        if (value.isEmpty() && placeholder != null) {
                            Text(
                                text = placeholder,
                                style = MaterialTheme.typography.bodyLarge,
                                color = brand.mutedForeground,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        innerTextField()
                    }
                    trailing?.invoke()
                }
            },
        )

        if (isError && errorText != null) {
            Text(
                text = errorText,
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(horizontal = SpacingTokens.Large),
            )
        }
    }
}

/**
 * Brand switch, the same as the desktop's `AcerolaSwitch`: checked track `primary` with a
 * `primary-foreground` thumb; unchecked track `muted-foreground` at 35% with a `foreground` thumb,
 * so it never disappears into the row behind it. No border.
 */
@Composable
fun Acerola.Component.Switch(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
) {
    val brand = AcerolaExtendedTheme.colors
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        enabled = enabled,
        modifier = modifier,
        colors =
            SwitchDefaults.colors(
                checkedThumbColor = MaterialTheme.colorScheme.onPrimary,
                checkedTrackColor = MaterialTheme.colorScheme.primary,
                checkedBorderColor = Color.Transparent,
                uncheckedThumbColor = brand.foreground,
                uncheckedTrackColor = brand.mutedForeground.copy(alpha = 0.35f),
                uncheckedBorderColor = Color.Transparent,
                disabledCheckedThumbColor = MaterialTheme.colorScheme.onPrimary.copy(alpha = 0.6f),
                disabledCheckedTrackColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.6f),
                disabledCheckedBorderColor = Color.Transparent,
                disabledUncheckedThumbColor = brand.foreground.copy(alpha = 0.6f),
                disabledUncheckedTrackColor = brand.mutedForeground.copy(alpha = 0.2f),
                disabledUncheckedBorderColor = Color.Transparent,
            ),
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FormPreview() {
    AcerolaTheme {
        Column(
            modifier = Modifier.padding(SpacingTokens.Large),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            Acerola.Component.TextField(value = "", onValueChange = {}, placeholder = "Buscar títulos", leadingIcon = Icons.Default.Search)
            Acerola.Component.TextField(value = "abc", onValueChange = {}, isError = true, errorText = "Código inválido")
            Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
                Acerola.Component.Switch(checked = true, onCheckedChange = {})
                Acerola.Component.Switch(checked = false, onCheckedChange = {})
            }
        }
    }
}
