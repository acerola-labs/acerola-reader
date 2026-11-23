package br.acerola.comic.common.ux.modifier

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// Visually signals that a LazyRow continues past the visible edge — without this, if the
// item fits an exact number of times in the screen width, the list looks like a closed grid instead of something to swipe.
// `edgeColor` must be the background color behind the list, so it dissolves into it.
fun Modifier.horizontalScrollFade(
    state: LazyListState,
    edgeColor: Color,
    edgeWidth: Dp = 32.dp,
): Modifier =
    this.drawWithContent {
        drawContent()

        val fadeWidthPx = edgeWidth.toPx()

        if (state.canScrollBackward) {
            drawRect(
                brush = Brush.horizontalGradient(colors = listOf(edgeColor, Color.Transparent), endX = fadeWidthPx),
            )
        }

        if (state.canScrollForward) {
            drawRect(
                brush =
                    Brush.horizontalGradient(
                        colors = listOf(Color.Transparent, edgeColor),
                        startX = size.width - fadeWidthPx,
                    ),
            )
        }
    }
