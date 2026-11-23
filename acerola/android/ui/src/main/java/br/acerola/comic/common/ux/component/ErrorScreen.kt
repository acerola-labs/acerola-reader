package br.acerola.comic.common.ux.component

import android.content.res.Configuration
import android.provider.Settings
import androidx.annotation.StringRes
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredWidth
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shadow
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaColors
import br.acerola.comic.common.ux.theme.AcerolaDimens
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaFonts
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.ui.R
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

/**
 * The one error screen of the app (acerola-design skill, section 6.1), the Compose version of the
 * brand book's `AcerolaErrorScreen`. Any failure that leaves a screen without content lands here:
 * never a system dialog, a blank screen or a raw exception message.
 */
enum class ErrorKind(
    @param:StringRes val title: Int,
    @param:StringRes val text: Int,
    val retry: Boolean,
) {
    BadRequest(R.string.error_bad_request_title, R.string.error_bad_request_text, retry = false),
    Unauthorized(R.string.error_unauthorized_title, R.string.error_unauthorized_text, retry = false),
    Forbidden(R.string.error_forbidden_title, R.string.error_forbidden_text, retry = false),
    NotFound(R.string.error_not_found_title, R.string.error_not_found_text, retry = false),
    Timeout(R.string.error_timeout_title, R.string.error_timeout_text, retry = true),
    TooMany(R.string.error_too_many_title, R.string.error_too_many_text, retry = true),
    Server(R.string.error_server_title, R.string.error_server_text, retry = true),
    Unavailable(R.string.error_unavailable_title, R.string.error_unavailable_text, retry = true),
    Offline(R.string.error_offline_title, R.string.error_offline_text, retry = true),
    Unknown(R.string.error_unknown_title, R.string.error_unknown_text, retry = true),
    ;

    companion object {
        /** Same mapping as the web `errorKind`: the status picks the variant, offline wins. */
        fun from(
            status: Int,
            online: Boolean = true,
        ): ErrorKind =
            when {
                !online -> Offline
                status == 400 -> BadRequest
                status == 401 -> Unauthorized
                status == 403 -> Forbidden
                status == 404 -> NotFound
                status == 408 -> Timeout
                status == 429 -> TooMany
                status == 500 -> Server
                status in 502..504 -> Unavailable
                status >= 500 -> Server
                status >= 400 -> BadRequest
                else -> Unknown
            }
    }
}

/**
 * @param status the code shown as decoration on the stage (hidden from accessibility); 0 or
 *   [ErrorKind.Offline] shows no code.
 * @param onHome go to the start of the app. Primary for 4xx-like kinds.
 * @param onRetry try again. Primary for 5xx, timeout, limit, offline and unknown.
 * @param onBack go back, the secondary action when home is primary.
 */
@Composable
fun Acerola.Component.ErrorScreen(
    kind: ErrorKind,
    onHome: () -> Unit,
    onRetry: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    status: Int = 0,
) {
    val eyebrow =
        when {
            kind == ErrorKind.Offline -> stringResource(R.string.error_eyebrow_offline)
            status > 0 -> stringResource(R.string.error_eyebrow, status)
            else -> stringResource(R.string.error_eyebrow_generic)
        }
    val title = stringResource(kind.title)

    BoxWithConstraints(
        modifier = modifier.fillMaxSize().background(MaterialTheme.colorScheme.background),
        contentAlignment = Alignment.Center,
    ) {
        // Small screen: the stage first and smaller, copy centered, actions split the width.
        val small = maxWidth < 720.dp
        val code = if (kind == ErrorKind.Offline || status == 0) "" else status.toString()

        if (small) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center,
                modifier =
                    Modifier
                        .fillMaxSize()
                        .verticalScroll(rememberScrollState())
                        .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Large),
            ) {
                ErrorStage(code = code, tape = listOf(title, eyebrow), modifier = Modifier.widthIn(max = 304.dp).fillMaxWidth())
                ErrorCopy(kind, eyebrow, title, onHome, onRetry, onBack, centered = true)
            }
        } else {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(48.dp),
                modifier = Modifier.widthIn(max = 1152.dp).padding(horizontal = SpacingTokens.Large),
            ) {
                Box(modifier = Modifier.weight(1f)) {
                    ErrorCopy(kind, eyebrow, title, onHome, onRetry, onBack, centered = false)
                }
                ErrorStage(code = code, tape = listOf(title, eyebrow), modifier = Modifier.weight(1.1f))
            }
        }
    }
}

@Composable
private fun ErrorCopy(
    kind: ErrorKind,
    eyebrow: String,
    title: String,
    onHome: () -> Unit,
    onRetry: () -> Unit,
    onBack: () -> Unit,
    centered: Boolean,
) {
    val align = if (centered) TextAlign.Center else TextAlign.Start
    Column(horizontalAlignment = if (centered) Alignment.CenterHorizontally else Alignment.Start) {
        Text(
            text = eyebrow,
            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.ExtraBold),
            color = MaterialTheme.colorScheme.error,
            modifier =
                Modifier
                    .clip(AcerolaShapes.Full)
                    .background(MaterialTheme.colorScheme.errorContainer)
                    .padding(horizontal = 14.dp, vertical = 6.dp),
        )
        Text(
            text = title,
            style =
                MaterialTheme.typography.displaySmall.copy(
                    fontSize = if (centered) 36.sp else 60.sp,
                    lineHeight = if (centered) 36.sp else 58.sp,
                    letterSpacing = (-1.5).sp,
                ),
            color = MaterialTheme.colorScheme.onBackground,
            textAlign = align,
            modifier = Modifier.padding(top = SpacingTokens.Medium).semantics { heading() },
        )
        Text(
            text = stringResource(kind.text),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = align,
            modifier = Modifier.padding(top = SpacingTokens.Medium).widthIn(max = 480.dp),
        )

        // Two ways out: one primary (home for 4xx, retry for 5xx/offline), one secondary.
        val primary = if (kind.retry) R.string.error_action_retry to onRetry else R.string.error_action_home to onHome
        val secondary = if (kind.retry) R.string.error_action_home to onHome else R.string.error_action_back to onBack
        Row(
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            modifier = (if (centered) Modifier.fillMaxWidth() else Modifier).padding(top = SpacingTokens.Large),
        ) {
            Button(
                onClick = primary.second,
                shape = AcerolaShapes.Full,
                modifier = Modifier.heightIn(min = AcerolaDimens.ControlHeight).then(if (centered) Modifier.weight(1f) else Modifier),
            ) {
                Text(stringResource(primary.first), style = MaterialTheme.typography.labelLarge)
            }
            Button(
                onClick = secondary.second,
                shape = AcerolaShapes.Full,
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant,
                        contentColor = MaterialTheme.colorScheme.onSurface,
                    ),
                modifier = Modifier.heightIn(min = AcerolaDimens.ControlHeight).then(if (centered) Modifier.weight(1f) else Modifier),
            ) {
                Text(stringResource(secondary.first), style = MaterialTheme.typography.labelLarge)
            }
        }
    }
}

/** Sticker star (40 points), the stamp behind the mascot, as in the brand hero. */
private val BurstShape =
    object : Shape {
        override fun createOutline(
            size: Size,
            layoutDirection: LayoutDirection,
            density: Density,
        ): Outline {
            val path = Path()
            repeat(40) { i ->
                val angle = (i / 40.0) * 2 * PI
                val r = if (i % 2 == 1) 0.42f else 0.5f
                val point = Offset(size.width * (0.5f + r * cos(angle).toFloat()), size.height * (0.5f + r * sin(angle).toFloat()))
                if (i == 0) path.moveTo(point.x, point.y) else path.lineTo(point.x, point.y)
            }
            path.close()
            return Outline.Generic(path)
        }
    }

@Composable
private fun ErrorStage(
    code: String,
    tape: List<String>,
    modifier: Modifier = Modifier,
) {
    // Loops only when the system allows motion (animator scale 0 = reduced motion).
    val context = LocalContext.current
    val motion =
        remember(context) {
            Settings.Global.getFloat(context.contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) > 0f
        }
    val loop = rememberInfiniteTransition(label = "error_stage")
    val spin by loop.animateFloat(0f, if (motion) 360f else 0f, infiniteRepeatable(tween(50_000, easing = LinearEasing)), label = "spin")
    val breathe by loop.animateFloat(
        0f,
        if (motion) -12f else 0f,
        infiniteRepeatable(tween(2_400), RepeatMode.Reverse),
        label = "breathe",
    )
    val run by loop.animateFloat(0f, if (motion) -0.5f else 0f, infiniteRepeatable(tween(18_000, easing = LinearEasing)), label = "tape")

    val crust = if (MaterialTheme.colorScheme.background == AcerolaColors.Dark.Background) AcerolaColors.Dark.Crust else AcerolaColors.Light.Crust
    val tilts = listOf(-7f, 4f, -3f)

    // Pure decoration: the eyebrow already says the status in words.
    BoxWithConstraints(modifier = modifier.aspectRatio(1f).clearAndSetSemantics {}, contentAlignment = Alignment.Center) {
        val side = maxWidth

        Box(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(side * 0.06f)
                    .rotate(spin)
                    .clip(BurstShape)
                    .background(MaterialTheme.colorScheme.primary),
        )

        if (code.isNotEmpty()) {
            Row(modifier = Modifier.align(Alignment.TopCenter).offset(y = side * 0.02f)) {
                code.forEachIndexed { i, digit ->
                    Text(
                        text = digit.toString(),
                        style =
                            TextStyle(
                                fontFamily = AcerolaFonts.Display,
                                fontSize = (side.value * 0.3f).sp,
                                lineHeight = (side.value * 0.26f).sp,
                                color = if (i % 2 == 0) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.onBackground,
                                shadow = Shadow(color = crust, offset = Offset(8f, 8f), blurRadius = 0f),
                            ),
                        modifier = Modifier.rotate(tilts[i % tilts.size]),
                    )
                }
            }
        }

        Image(
            painter = painterResource(id = R.drawable.acerola_mascot),
            contentDescription = null,
            modifier =
                Modifier
                    .fillMaxWidth(0.62f)
                    .offset(y = side * 0.11f)
                    .graphicsLayer { translationY = breathe * density },
        )

        // The tape repeats the title and the eyebrow, running sideways.
        val phrase = (tape + tape).joinToString("  ✦  ").uppercase()
        Box(
            modifier =
                Modifier
                    .align(Alignment.BottomCenter)
                    .offset(y = -(side * 0.12f))
                    .requiredWidth(side * 1.12f)
                    .rotate(-5f)
                    .background(MaterialTheme.colorScheme.onBackground)
                    .padding(vertical = 8.dp),
        ) {
            Text(
                text = "$phrase  ✦  $phrase",
                maxLines = 1,
                softWrap = false,
                style = TextStyle(fontFamily = AcerolaFonts.Display, fontSize = 16.sp, color = MaterialTheme.colorScheme.background),
                modifier = Modifier.graphicsLayer { translationX = run * size.width },
            )
        }

        Text(
            text = stringResource(R.string.error_sticker).uppercase(),
            style = TextStyle(fontFamily = AcerolaFonts.Display, fontSize = 16.sp, color = crust),
            modifier =
                Modifier
                    .align(Alignment.TopEnd)
                    .offset(x = -(side * 0.04f), y = side * 0.3f)
                    .rotate(10f)
                    .clip(AcerolaShapes.Full)
                    .background(AcerolaExtendedTheme.colors.warning)
                    .padding(horizontal = 14.dp, vertical = 6.dp),
        )
    }
}

@Preview(name = "Light", showBackground = true, widthDp = 400, heightDp = 860)
@Preview(name = "Dark", showBackground = true, widthDp = 400, heightDp = 860, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun ErrorScreenPreview() {
    AcerolaTheme {
        Acerola.Component.ErrorScreen(kind = ErrorKind.NotFound, status = 404, onHome = {}, onRetry = {}, onBack = {})
    }
}

@Preview(name = "Tablet", showBackground = true, widthDp = 1100, heightDp = 700)
@Composable
private fun ErrorScreenWidePreview() {
    AcerolaTheme {
        Acerola.Component.ErrorScreen(kind = ErrorKind.Unknown, onHome = {}, onRetry = {}, onBack = {})
    }
}
