package br.acerola.comic.common.ux.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
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
    // Brand surface levels and text, by the same name the desktop uses (`bg-card`, `bg-surface`,
    // `bg-input`, `text-muted-foreground`), so a screen reads the same on both sides.
    val crust: Color,
    val mantle: Color,
    val background: Color,
    val card: Color,
    val surface: Color,
    val input: Color,
    val foreground: Color,
    val mutedForeground: Color,
    val primarySoft: Color,
    val info: Color,
    val infoSoft: Color,
    val destructiveSoft: Color,
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
        crust = AcerolaColors.Dark.Crust,
        mantle = AcerolaColors.Dark.Mantle,
        background = AcerolaColors.Dark.Background,
        card = AcerolaColors.Dark.Card,
        surface = AcerolaColors.Dark.Surface,
        input = AcerolaColors.Dark.Input,
        foreground = AcerolaColors.Dark.Foreground,
        mutedForeground = AcerolaColors.Dark.MutedForeground,
        primarySoft = AcerolaColors.Dark.PrimarySoft,
        info = AcerolaColors.Dark.Info,
        infoSoft = AcerolaColors.Dark.InfoSoft,
        destructiveSoft = AcerolaColors.Dark.DestructiveSoft,
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
        surfaceContainerLowest = AcerolaColors.Light.Crust,
        surfaceContainerLow = AcerolaColors.Light.Mantle,
        surfaceContainer = AcerolaColors.Light.Card,
        surfaceContainerHigh = AcerolaColors.Light.Surface,
        surfaceContainerHighest = AcerolaColors.Light.Border,
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
        crust = AcerolaColors.Light.Crust,
        mantle = AcerolaColors.Light.Mantle,
        background = AcerolaColors.Light.Background,
        card = AcerolaColors.Light.Card,
        surface = AcerolaColors.Light.Surface,
        input = AcerolaColors.Light.Input,
        foreground = AcerolaColors.Light.Foreground,
        mutedForeground = AcerolaColors.Light.MutedForeground,
        primarySoft = AcerolaColors.Light.PrimarySoft,
        info = AcerolaColors.Light.Info,
        infoSoft = AcerolaColors.Light.InfoSoft,
        destructiveSoft = AcerolaColors.Light.DestructiveSoft,
    )

@Composable
fun AcerolaTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme
    val extraColors = if (darkTheme) DarkExtraColors else LightExtraColors

    CompositionLocalProvider(
        LocalAcerolaExtraColors provides extraColors,
        LocalAcerolaSurfaceLevel provides AcerolaSurfaceLevel.Background,
    ) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            shapes = BrandShapes,
            content = content,
        )
    }
}

// Every Material3 component picks its corner from these, so a stock component already lands on a
// brand radius: field/menu Inner, card/list row Tile, dialog/large card Card, sheet Sheet.
private val BrandShapes =
    Shapes(
        extraSmall = AcerolaShapes.Inner,
        small = AcerolaShapes.Inner,
        medium = AcerolaShapes.Tile,
        large = AcerolaShapes.Card,
        extraLarge = AcerolaShapes.Sheet,
    )

/**
 * Which brand level a piece of UI sits on, the Compose side of the desktop's
 * `data-surface="surface"`: a button or field on a `surface` row lifts to `card`, otherwise it
 * would vanish into the row.
 */
enum class AcerolaSurfaceLevel { Background, Card, Surface }

val LocalAcerolaSurfaceLevel = compositionLocalOf { AcerolaSurfaceLevel.Background }

/** Fill for a neutral control (secondary button, field, icon button) on the current level. */
val AcerolaExtraColors.raised: Color
    @Composable get() = if (LocalAcerolaSurfaceLevel.current == AcerolaSurfaceLevel.Surface) card else surface

/** Provides [level] to [content], for a container painted with that level's color. */
@Composable
fun ProvideSurfaceLevel(
    level: AcerolaSurfaceLevel,
    content: @Composable () -> Unit,
) {
    CompositionLocalProvider(LocalAcerolaSurfaceLevel provides level, content = content)
}
