package br.acerola.comic.common.ux.modifier

import androidx.compose.foundation.border
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.dp

/** Visible focus (`focus-visible:ring-2`): a 2dp `ring` outline, only while [focused]. */
@Composable
fun Modifier.focusRing(
    focused: Boolean,
    shape: Shape,
): Modifier = if (focused) border(2.dp, MaterialTheme.colorScheme.primary, shape) else this
