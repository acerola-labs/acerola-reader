package br.acerola.comic.common.ux.tokens

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.tween

/**
 * The brand motion (`--ease-brand` and `--duration-brand` in `theme/tokens.css`). Compose already
 * scales these by the system animation setting, so "remove animations" turns them off.
 */
object MotionTokens {
    val EaseBrand = CubicBezierEasing(0.32f, 0.72f, 0f, 1f)
    const val DurationBrand = 320

    fun <T> brand(): FiniteAnimationSpec<T> = tween(durationMillis = DurationBrand, easing = EaseBrand)
}
