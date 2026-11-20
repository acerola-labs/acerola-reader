package br.acerola.comic.common.ux.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

// Roles Material3 does not have (success, warning, vibrant accent), exposed through
// AcerolaExtendedTheme. Action icons follow the desktop: SOLID background in the semantic color
// with a neutral icon on top, so success/accent are the full tone and on* is
// `--primary-foreground`. The *Container pairs are the soft background + tone text
// (`--success-soft` + `--success`), the same as errorContainer (`--destructive-soft`).
data class AcerolaExtraColors(
    val success: Color,
    val onSuccess: Color,
    val accent: Color,
    val onAccent: Color,
    val successContainer: Color,
    val onSuccessContainer: Color,
    val warning: Color,
    val warningContainer: Color,
    val onWarningContainer: Color,
)

private val LocalAcerolaExtraColors =
    staticCompositionLocalOf<AcerolaExtraColors> {
        error("AcerolaExtraColors not provided: use it inside AcerolaTheme")
    }

object AcerolaExtendedTheme {
    val colors: AcerolaExtraColors
        @Composable get() = LocalAcerolaExtraColors.current
}

// One theme: only light/dark changes (following the system). Material3 roles take the brand
// tokens (Color.kt, generated from the acerola-brand tokens.json) with the same role each token
// has in CSS: background/card/surface for the surface levels, border for dividers, *Soft for
// containers.
private val DarkColorScheme =
    darkColorScheme(
        primary = AcerolaColors.Dark.Primary,
        onPrimary = AcerolaColors.Dark.PrimaryForeground,
        primaryContainer = AcerolaColors.Dark.PrimarySoft,
        onPrimaryContainer = AcerolaColors.Dark.Primary,
        secondary = AcerolaColors.Dark.AccentHero,
        onSecondary = AcerolaColors.Dark.PrimaryForeground,
        secondaryContainer = AcerolaColors.Dark.Secondary,
        onSecondaryContainer = AcerolaColors.Dark.AccentHero,
        tertiary = AcerolaColors.Dark.Chart5,
        onTertiary = AcerolaColors.Dark.PrimaryForeground,
        tertiaryContainer = AcerolaColors.Dark.Secondary,
        onTertiaryContainer = AcerolaColors.Dark.Chart5,
        background = AcerolaColors.Dark.Background,
        onBackground = AcerolaColors.Dark.Foreground,
        surface = AcerolaColors.Dark.Background,
        onSurface = AcerolaColors.Dark.Foreground,
        surfaceVariant = AcerolaColors.Dark.Surface,
        onSurfaceVariant = AcerolaColors.Dark.MutedForeground,
        surfaceContainerLowest = AcerolaColors.Dark.Crust,
        surfaceContainerLow = AcerolaColors.Dark.Mantle,
        surfaceContainer = AcerolaColors.Dark.Card,
        surfaceContainerHigh = AcerolaColors.Dark.Surface,
        surfaceContainerHighest = AcerolaColors.Dark.Border,
        outline = AcerolaColors.Dark.Border,
        outlineVariant = AcerolaColors.Dark.Border,
        error = AcerolaColors.Dark.Destructive,
        onError = AcerolaColors.Dark.PrimaryForeground,
        errorContainer = AcerolaColors.Dark.DestructiveSoft,
        onErrorContainer = AcerolaColors.Dark.Destructive,
        scrim = AcerolaColors.Scrim,
    )

private val DarkExtraColors =
    AcerolaExtraColors(
        success = AcerolaColors.Dark.Success,
        onSuccess = AcerolaColors.Dark.PrimaryForeground,
        accent = AcerolaColors.Dark.AccentHero,
        onAccent = AcerolaColors.Dark.PrimaryForeground,
        successContainer = AcerolaColors.Dark.SuccessSoft,
        onSuccessContainer = AcerolaColors.Dark.Success,
        warning = AcerolaColors.Dark.Warning,
        warningContainer = AcerolaColors.Dark.WarningSoft,
        onWarningContainer = AcerolaColors.Dark.Warning,
    )

private val LightColorScheme =
    lightColorScheme(
        primary = AcerolaColors.Light.Primary,
        onPrimary = AcerolaColors.Light.PrimaryForeground,
        primaryContainer = AcerolaColors.Light.PrimarySoft,
        onPrimaryContainer = AcerolaColors.Light.Primary,
        secondary = AcerolaColors.Light.AccentHero,
        onSecondary = AcerolaColors.Light.PrimaryForeground,
        secondaryContainer = AcerolaColors.Light.Secondary,
        onSecondaryContainer = AcerolaColors.Light.AccentHero,
        tertiary = AcerolaColors.Light.Chart5,
        onTertiary = AcerolaColors.Light.PrimaryForeground,
        tertiaryContainer = AcerolaColors.Light.Secondary,
        onTertiaryContainer = AcerolaColors.Light.Chart5,
        background = AcerolaColors.Light.Background,
        onBackground = AcerolaColors.Light.Foreground,
        surface = AcerolaColors.Light.Background,
        onSurface = AcerolaColors.Light.Foreground,
        surfaceVariant = AcerolaColors.Light.Surface,
        onSurfaceVariant = AcerolaColors.Light.MutedForeground,
        surfaceContainerLowest = AcerolaColors.Light.Card,
        surfaceContainerLow = AcerolaColors.Light.Base,
        surfaceContainer = AcerolaColors.Light.Mantle,
        surfaceContainerHigh = AcerolaColors.Light.Surface,
        surfaceContainerHighest = AcerolaColors.Light.Crust,
        outline = AcerolaColors.Light.Border,
        outlineVariant = AcerolaColors.Light.Border,
        error = AcerolaColors.Light.Destructive,
        onError = AcerolaColors.Light.PrimaryForeground,
        errorContainer = AcerolaColors.Light.DestructiveSoft,
        onErrorContainer = AcerolaColors.Light.Destructive,
        scrim = AcerolaColors.Scrim,
    )

private val LightExtraColors =
    AcerolaExtraColors(
        success = AcerolaColors.Light.Success,
        onSuccess = AcerolaColors.Light.PrimaryForeground,
        accent = AcerolaColors.Light.AccentHero,
        onAccent = AcerolaColors.Light.PrimaryForeground,
        successContainer = AcerolaColors.Light.SuccessSoft,
        onSuccessContainer = AcerolaColors.Light.Success,
        warning = AcerolaColors.Light.Warning,
        warningContainer = AcerolaColors.Light.WarningSoft,
        onWarningContainer = AcerolaColors.Light.Warning,
    )

@Composable
fun AcerolaTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme
    val extraColors = if (darkTheme) DarkExtraColors else LightExtraColors

    CompositionLocalProvider(LocalAcerolaExtraColors provides extraColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            content = content,
        )
    }
}
