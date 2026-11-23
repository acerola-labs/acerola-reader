package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaFonts
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R

// Screen title (`text-[44px] font-black tracking-tighter leading-none`), the same on every app.
private val ScreenTitleStyle =
    TextStyle(
        fontFamily = AcerolaFonts.Sans,
        fontWeight = FontWeight.Black,
        fontSize = 44.sp,
        lineHeight = 44.sp,
        letterSpacing = (-2.2).sp,
    )

/**
 * Brand screen header, the desktop's `AcerolaSettingsHeader`: optional back button, the
 * [eyebrow] (`text-xs font-semibold text-muted-foreground`) over the 44sp title, and a [lead].
 * Solid `bg-background`, no glass; [actions] sit at the right of the back row.
 */
@Composable
fun Acerola.Component.ScreenHeader(
    title: String,
    modifier: Modifier = Modifier,
    eyebrow: String? = null,
    lead: String? = null,
    onBack: (() -> Unit)? = null,
    actions: @Composable (RowScope.() -> Unit)? = null,
) {
    val brand = AcerolaExtendedTheme.colors
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .background(brand.background)
                .windowInsetsPadding(WindowInsets.statusBars)
                .padding(horizontal = SpacingTokens.ExtraLarge)
                .padding(top = SpacingTokens.Large, bottom = SpacingTokens.ExtraLarge),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
    ) {
        if (onBack != null || actions != null) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(bottom = SpacingTokens.Small),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            ) {
                if (onBack != null) {
                    Acerola.Component.IconButton(
                        icon = Icons.AutoMirrored.Filled.ArrowBack,
                        contentDescription = stringResource(R.string.common_back),
                        onClick = onBack,
                    )
                }
                Row(
                    modifier = Modifier.weight(1f),
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small, Alignment.End),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    actions?.invoke(this)
                }
            }
        }

        if (eyebrow != null) {
            Text(
                text = eyebrow,
                style = MaterialTheme.typography.labelMedium,
                color = brand.mutedForeground,
            )
        }
        Text(text = title, style = ScreenTitleStyle, color = brand.foreground)
        if (lead != null) {
            Text(
                text = lead,
                style = MaterialTheme.typography.bodyMedium,
                color = brand.mutedForeground,
                modifier = Modifier.padding(top = SpacingTokens.ExtraSmall),
            )
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ScreenHeaderPreview() {
    AcerolaTheme {
        Acerola.Component.ScreenHeader(
            eyebrow = "Preferências do app",
            title = "Configurações",
            lead = "Pasta, biblioteca, metadados e marcadores",
            onBack = {},
        )
    }
}
