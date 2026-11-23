package br.acerola.comic.common.ux.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import br.acerola.comic.ui.R

// Acerola fonts (acerola-design skill, section 4), the same roles as `font-sans`,
// `font-heading` and `font-display` in the brand tokens:
// - Sans: DM Sans, the whole interface (text, button, list, title, number).
// - Heading: Noto Serif, rare editorial titles. It is Android's own serif, so it is not bundled.
// - Display: Titan One, brand only (lettering, hero, splash, error stage). Never in a button,
//   tab, field, list or screen title.
// Licenses (SIL OFL 1.1) in assets/licenses.
object AcerolaFonts {
    private val sansWeights =
        listOf(FontWeight.Normal, FontWeight.Medium, FontWeight.SemiBold, FontWeight.Bold, FontWeight.Black)

    // DM Sans is a variable font: one file, each weight picked through the `wght` axis.
    // `variationSettings` on a resource font is still marked experimental in Compose.
    @OptIn(ExperimentalTextApi::class)
    val Sans =
        FontFamily(
            sansWeights.map { weight ->
                Font(
                    resId = R.font.dm_sans,
                    weight = weight,
                    variationSettings = FontVariation.Settings(FontVariation.weight(weight.weight)),
                )
            },
        )

    val Heading = FontFamily.Serif

    val Display = FontFamily(Font(R.font.titan_one, FontWeight.Normal))
}

// Material3 scale in DM Sans. Strong text is Black with tight tracking, as on the web
// (`font-black tracking-tight`); body text is 14sp minimum.
val Typography =
    Typography(
        displayLarge =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 57.sp,
                lineHeight = 60.sp,
                letterSpacing = (-1.5).sp,
            ),
        displayMedium =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 45.sp,
                lineHeight = 48.sp,
                letterSpacing = (-1).sp,
            ),
        displaySmall =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 36.sp,
                lineHeight = 40.sp,
                letterSpacing = (-0.75).sp,
            ),
        headlineLarge =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 32.sp,
                lineHeight = 36.sp,
                letterSpacing = (-0.5).sp,
            ),
        headlineMedium =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 28.sp,
                lineHeight = 32.sp,
                letterSpacing = (-0.5).sp,
            ),
        headlineSmall =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 24.sp,
                lineHeight = 28.sp,
                letterSpacing = (-0.25).sp,
            ),
        titleLarge =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 22.sp,
                lineHeight = 28.sp,
                letterSpacing = (-0.25).sp,
            ),
        titleMedium =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 18.sp,
                lineHeight = 24.sp,
                letterSpacing = (-0.15).sp,
            ),
        titleSmall =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Bold,
                fontSize = 14.sp,
                lineHeight = 20.sp,
                letterSpacing = 0.sp,
            ),
        bodyLarge =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Normal,
                fontSize = 16.sp,
                lineHeight = 24.sp,
                letterSpacing = 0.sp,
            ),
        bodyMedium =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Normal,
                fontSize = 14.sp,
                lineHeight = 20.sp,
                letterSpacing = 0.sp,
            ),
        bodySmall =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Normal,
                fontSize = 12.sp,
                lineHeight = 16.sp,
                letterSpacing = 0.1.sp,
            ),
        labelLarge =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.Black,
                fontSize = 14.sp,
                lineHeight = 20.sp,
                letterSpacing = 0.sp,
            ),
        labelMedium =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.SemiBold,
                fontSize = 12.sp,
                lineHeight = 16.sp,
                letterSpacing = 0.1.sp,
            ),
        labelSmall =
            TextStyle(
                fontFamily = AcerolaFonts.Sans,
                fontWeight = FontWeight.SemiBold,
                fontSize = 11.sp,
                lineHeight = 16.sp,
                letterSpacing = 0.1.sp,
            ),
    )
