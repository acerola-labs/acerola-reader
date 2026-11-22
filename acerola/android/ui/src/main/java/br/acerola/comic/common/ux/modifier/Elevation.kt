package br.acerola.comic.common.ux.modifier

import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.theme.AcerolaColors

// The only two brand shadows, the Compose side of `shadow-lift` (a card at rest) and
// `shadow-float` (a sheet, toast or bar that floats over the screen).

fun Modifier.lift(shape: Shape): Modifier =
    shadow(
        elevation = 10.dp,
        shape = shape,
        ambientColor = AcerolaColors.Scrim.copy(alpha = 0.45f),
        spotColor = AcerolaColors.Scrim.copy(alpha = 0.45f),
    )

fun Modifier.float(shape: Shape): Modifier =
    shadow(
        elevation = 20.dp,
        shape = shape,
        ambientColor = AcerolaColors.Scrim.copy(alpha = 0.55f),
        spotColor = AcerolaColors.Scrim.copy(alpha = 0.55f),
    )
