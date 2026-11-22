package br.acerola.comic.common.ux.component
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.WarningAmber
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarVisuals
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.float
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens

enum class SnackbarVariant { Error, Success, Warn }

private data class AcerolaSnackbarVisuals(
    val variant: SnackbarVariant,
    override val message: String,
    override val actionLabel: String? = null,
    override val withDismissAction: Boolean = false,
    override val duration: SnackbarDuration = SnackbarDuration.Short,
) : SnackbarVisuals

internal fun resolveSnackbarVariant(visuals: SnackbarVisuals): SnackbarVariant =
    (visuals as? AcerolaSnackbarVisuals)?.variant ?: SnackbarVariant.Error

suspend fun SnackbarHostState.showSnackbar(
    message: String,
    variant: SnackbarVariant,
) {
    showSnackbar(
        AcerolaSnackbarVisuals(
            message = message,
            variant = variant,
        ),
    )
}

@Composable
fun Acerola.Component.SnackbarError(
    message: String,
    modifier: Modifier = Modifier,
) {
    Toast(message, Tone.Destructive, Icons.Rounded.ErrorOutline, modifier)
}

@Composable
fun Acerola.Component.SnackbarSuccess(
    message: String,
    modifier: Modifier = Modifier,
) {
    Toast(message, Tone.Success, Icons.Rounded.CheckCircle, modifier)
}

@Composable
fun Acerola.Component.SnackbarWarn(
    message: String,
    modifier: Modifier = Modifier,
) {
    Toast(message, Tone.Warning, Icons.Rounded.WarningAmber, modifier)
}

// Brand toast (`.ac-toast`): rounded-card, shadow-float, no border, background and text in the
// tone of the type (`success-soft` + `success`, and so on), an icon in the tone.
@Composable
private fun Toast(
    message: String,
    tone: Tone,
    icon: ImageVector,
    modifier: Modifier,
) {
    val colors = tone.colors()
    Row(
        modifier =
            modifier
                .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small)
                .fillMaxWidth()
                .float(AcerolaShapes.Card)
                .clip(AcerolaShapes.Card)
                .background(colors.container)
                .padding(horizontal = SpacingTokens.ExtraLarge - SpacingTokens.ExtraSmall / 2, vertical = SpacingTokens.MediumLarge),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.MediumLarge),
    ) {
        Icon(imageVector = icon, contentDescription = null, tint = colors.content, modifier = Modifier.size(SizeTokens.IconSmall))
        Text(text = message, style = MaterialTheme.typography.labelLarge, color = colors.content)
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SnackbarErrorPreview() {
    AcerolaTheme {
        Acerola.Component.SnackbarError(message = "Error message")
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SnackbarSuccessPreview() {
    AcerolaTheme {
        Acerola.Component.SnackbarSuccess(message = "Success message")
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SnackbarWarnPreview() {
    AcerolaTheme {
        Acerola.Component.SnackbarWarn(message = "Warning message")
    }
}
