package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.ExperimentalAnimationApi
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.modifier.float
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.raised
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R
import kotlinx.coroutines.delay

@OptIn(ExperimentalAnimationApi::class)
@Composable
fun Acerola.Component.Progress(
    modifier: Modifier = Modifier,
    progress: Float? = null,
    isLoading: Boolean,
) {
    var showIndicator by remember { mutableStateOf(false) }
    var isFinished by remember { mutableStateOf(false) }

    LaunchedEffect(isLoading) {
        if (isLoading) {
            isFinished = false
            showIndicator = true
        } else {
            if (showIndicator) {
                isFinished = true
                delay(2000)
                showIndicator = false
                isFinished = false
            }
        }
    }

    AnimatedVisibility(
        visible = showIndicator,
        enter = fadeIn() + slideInVertically { it / 2 },
        exit = fadeOut() + slideOutVertically { it / 2 },
        modifier = modifier,
    ) {
        Box(
            contentAlignment = Alignment.BottomStart,
        ) {
            // Toast (`.ac-toast`): rounded-card, bg-card, shadow-float.
            Surface(
                color = AcerolaExtendedTheme.colors.card,
                shape = AcerolaShapes.Card,
                modifier = Modifier.float(AcerolaShapes.Card),
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.MediumLarge),
                    horizontalArrangement = Arrangement.spacedBy(space = SpacingTokens.MediumLarge),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    AnimatedContent(
                        targetState = isFinished,
                        transitionSpec = {
                            fadeIn(animationSpec = tween(300)) togetherWith
                                fadeOut(animationSpec = tween(300))
                        },
                        label = "SyncStatusIcon",
                    ) { finished ->
                        if (finished) {
                            Icon(
                                imageVector = Icons.Rounded.CheckCircle,
                                contentDescription = null,
                                tint = AcerolaExtendedTheme.colors.success,
                                modifier = Modifier.size(SizeTokens.IconMedium),
                            )
                        } else {
                            if (progress == null) {
                                Acerola.Component.Spinner(size = SizeTokens.IconMedium)
                            } else {
                                CircularProgressIndicator(
                                    progress = { progress.coerceIn(0f, 1f) },
                                    modifier = Modifier.size(size = SizeTokens.IconMedium),
                                    color = MaterialTheme.colorScheme.primary,
                                    trackColor = AcerolaExtendedTheme.colors.surface,
                                    strokeWidth = 3.dp,
                                    strokeCap = StrokeCap.Round,
                                )
                            }
                        }
                    }

                    AnimatedContent(
                        targetState = isFinished,
                        label = "SyncStatusText",
                        transitionSpec = {
                            fadeIn(animationSpec = tween(300)) togetherWith
                                fadeOut(animationSpec = tween(300))
                        },
                    ) { finished ->
                        val text =
                            if (finished) {
                                stringResource(id = R.string.label_sync_complete)
                            } else {
                                if (progress != null) {
                                    stringResource(
                                        id = R.string.label_sync_progress_percent,
                                        (progress.coerceIn(0f, 1f) * 100).toInt(),
                                    )
                                } else {
                                    stringResource(id = R.string.label_sync_progress)
                                }
                            }

                        Text(
                            text = text,
                            style = MaterialTheme.typography.labelLarge,
                            color = if (finished) AcerolaExtendedTheme.colors.success else AcerolaExtendedTheme.colors.foreground,
                        )
                    }
                }
            }
        }
    }
}

/** Indeterminate spinner in the current content color, for a busy button or row. */
@Composable
fun Acerola.Component.Spinner(
    modifier: Modifier = Modifier,
    size: Dp = SizeTokens.IconSmall,
    color: Color = LocalContentColor.current,
) {
    CircularProgressIndicator(
        modifier = modifier.size(size),
        color = color,
        trackColor = Color.Transparent,
        strokeWidth = 2.5.dp,
        strokeCap = StrokeCap.Round,
    )
}

/** Thin progress bar (`.ac-progress`): `bg-surface` track, `bg-primary` fill, rounded ends. */
@Composable
fun Acerola.Component.ProgressBar(
    progress: Float,
    modifier: Modifier = Modifier,
) {
    LinearProgressIndicator(
        progress = { progress.coerceIn(0f, 1f) },
        modifier = modifier.fillMaxWidth().height(4.dp),
        color = MaterialTheme.colorScheme.primary,
        trackColor = AcerolaExtendedTheme.colors.raised,
        strokeCap = StrokeCap.Round,
        gapSize = 0.dp,
        drawStopIndicator = {},
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ProgressPreview() {
    AcerolaTheme {
        Acerola.Component.Progress(isLoading = true)
    }
}
