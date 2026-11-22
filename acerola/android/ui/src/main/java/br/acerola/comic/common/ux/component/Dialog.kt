package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.float
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

// Which footer slot a DialogButton sits in, so it picks the brand variant by itself: confirm is
// `bg-foreground` (contrast), dismiss is secondary.
private val LocalDialogSlotVariant = compositionLocalOf { ButtonVariant.Secondary }

/**
 * A dialog button. Without a [variant] it follows its slot: [ButtonVariant.Contrast] as the confirm
 * action, [ButtonVariant.Secondary] as the dismiss one. Pass [ButtonVariant.Destructive] when
 * confirming erases something.
 */
@Composable
fun Acerola.Component.DialogButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: ButtonVariant? = null,
    enabled: Boolean = true,
) {
    Acerola.Component.Button(
        text = text,
        onClick = onClick,
        modifier = modifier.fillMaxWidth(),
        variant = variant ?: LocalDialogSlotVariant.current,
        enabled = enabled,
    )
}

/**
 * Brand dialog (`.ac-sheet` on `bg-scrim/50`): `rounded-sheet bg-card p-6 shadow-float`, a
 * `font-black` title, muted body text and the actions as two pills of the same width.
 */
@Composable
@OptIn(ExperimentalMaterial3Api::class)
fun Acerola.Component.Dialog(
    show: Boolean,
    onDismiss: () -> Unit,
    title: String? = null,
    confirmButtonContent: (@Composable () -> Unit)? = null,
    dismissButtonContent: (@Composable () -> Unit)? = null,
    content: @Composable () -> Unit,
) {
    if (!show) return

    val brand = AcerolaExtendedTheme.colors
    BasicAlertDialog(onDismissRequest = onDismiss) {
        ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) {
            Column(
                modifier =
                    Modifier
                        .float(AcerolaShapes.Sheet)
                        .clip(AcerolaShapes.Sheet)
                        .background(brand.card)
                        .padding(SpacingTokens.Huge),
                verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
            ) {
                title?.let {
                    Text(text = it, style = MaterialTheme.typography.titleLarge, color = brand.foreground)
                }

                Box(modifier = Modifier.fillMaxWidth()) {
                    CompositionLocalProvider(LocalContentColor provides brand.mutedForeground) {
                        ProvideTextStyle(value = MaterialTheme.typography.bodyMedium) {
                            content()
                        }
                    }
                }

                // Stacked like the desktop dialog footer on a narrow screen (`flex-col-reverse`):
                // the confirm action on top, full width, so no label ever gets cut.
                if (dismissButtonContent != null || confirmButtonContent != null) {
                    Column(
                        modifier = Modifier.fillMaxWidth().padding(top = SpacingTokens.Small),
                        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
                    ) {
                        if (confirmButtonContent != null) {
                            CompositionLocalProvider(LocalDialogSlotVariant provides ButtonVariant.Contrast) {
                                confirmButtonContent()
                            }
                        }
                        if (dismissButtonContent != null) {
                            CompositionLocalProvider(LocalDialogSlotVariant provides ButtonVariant.Secondary) {
                                dismissButtonContent()
                            }
                        }
                    }
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun DialogPreview() {
    AcerolaTheme {
        Acerola.Component.Dialog(
            show = true,
            onDismiss = {},
            title = stringResource(R.string.dialog_delete_title),
            confirmButtonContent = {
                Acerola.Component.DialogButton(
                    text = stringResource(R.string.action_delete),
                    onClick = {},
                    variant = ButtonVariant.Destructive,
                )
            },
            dismissButtonContent = {
                Acerola.Component.DialogButton(text = stringResource(R.string.action_cancel), onClick = {})
            },
        ) {
            Text(text = stringResource(R.string.dialog_delete_message))
        }
    }
}
