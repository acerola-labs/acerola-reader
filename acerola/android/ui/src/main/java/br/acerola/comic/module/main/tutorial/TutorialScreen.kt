package br.acerola.comic.module.main.tutorial

import android.content.res.Configuration
import android.net.Uri
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material.icons.filled.Bolt
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.FolderZip
import androidx.compose.material.icons.filled.Layers
import androidx.compose.material.icons.filled.PictureAsPdf
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.VerifiedUser
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Badge
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.ProgressBar
import br.acerola.comic.common.ux.component.RoundIcon
import br.acerola.comic.common.ux.component.Tone
import br.acerola.comic.common.ux.component.colors
import br.acerola.comic.common.ux.modifier.lift
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaFonts
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaSurfaceLevel
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.theme.ProvideSurfaceLevel
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.archive.FileSystemAccessViewModel
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.config.component.SelectComicDirectory
import br.acerola.comic.module.main.tutorial.state.TutorialPage
import br.acerola.comic.ui.R
import kotlinx.coroutines.launch

@Composable
fun Main.Tutorial.Template.Screen(
    viewModel: TutorialViewModel = hiltViewModel(),
    fileSystemAccessViewModel: FileSystemAccessViewModel = hiltViewModel(),
    onNavigateToHome: () -> Unit,
) {
    val folderName by fileSystemAccessViewModel.folderName.collectAsState()

    TutorialScreenContent(
        folderName = folderName,
        onFolderSelected = fileSystemAccessViewModel::saveFolderUri,
        onComplete = {
            viewModel.markOnboardingCompleted()
            onNavigateToHome()
        },
    )
}

// Same flow as the desktop onboarding (acerola-onboarding.svelte): solid stepper on top (steps as
// pills, current `bg-primary`, done `bg-primary-soft`, and the progress bar), the step in the
// middle and the secondary "Back" / primary "Next" at the bottom.
@Composable
private fun TutorialScreenContent(
    folderName: String?,
    onFolderSelected: (Uri?) -> Unit,
    onComplete: () -> Unit,
) {
    val pages = TutorialPage.entries
    val pagerState = rememberPagerState(pageCount = { pages.size })
    val scope = rememberCoroutineScope()
    val current = pagerState.currentPage
    val isFirstPage = current == 0
    val isLastPage = current == pages.lastIndex
    val canProceed = current != TutorialPage.SETTINGS.ordinal || !folderName.isNullOrEmpty()

    fun goTo(page: Int) {
        scope.launch { pagerState.animateScrollToPage(page) }
    }

    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .background(AcerolaExtendedTheme.colors.background)
                .windowInsetsPadding(WindowInsets.statusBars)
                .windowInsetsPadding(WindowInsets.navigationBars),
    ) {
        Stepper(current = current, count = pages.size)

        HorizontalPager(
            state = pagerState,
            modifier = Modifier.weight(1f),
            userScrollEnabled = false,
        ) { pageIndex ->
            when (pages[pageIndex]) {
                TutorialPage.WELCOME -> WelcomeSlide()
                TutorialPage.FILE_FORMATS -> FormatsSlide()
                TutorialPage.SETTINGS -> SettingsSlide(folderName, onFolderSelected)
                TutorialPage.COMPLETE -> CompleteSlide()
            }
        }

        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Large),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (!isFirstPage) {
                Acerola.Component.Button(
                    text = stringResource(id = R.string.tutorial_action_previous),
                    onClick = { goTo(current - 1) },
                    variant = ButtonVariant.Secondary,
                )
            }
            Box(modifier = Modifier.weight(1f))
            Acerola.Component.Button(
                text =
                    stringResource(
                        id =
                            when {
                                isFirstPage -> R.string.tutorial_action_start
                                isLastPage -> R.string.tutorial_action_finish
                                else -> R.string.tutorial_action_next
                            },
                    ),
                onClick = { if (isLastPage) onComplete() else goTo(current + 1) },
                trailingIcon = Icons.AutoMirrored.Filled.ArrowForward,
                enabled = canProceed,
            )
        }
    }
}

@Composable
private fun Stepper(
    current: Int,
    count: Int,
) {
    val brand = AcerolaExtendedTheme.colors
    val labels =
        listOf(
            R.string.tutorial_step_welcome,
            R.string.tutorial_step_formats,
            R.string.tutorial_step_settings,
            R.string.tutorial_step_complete,
        )
    val progress = if (count > 1) current.toFloat() / (count - 1) else 1f

    Column(
        modifier = Modifier.fillMaxWidth().padding(horizontal = SpacingTokens.ExtraLarge, vertical = SpacingTokens.Large),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(
                text = stringResource(id = R.string.tutorial_setup_title),
                style = MaterialTheme.typography.labelMedium,
                color = brand.mutedForeground,
            )
            Text(
                text = "${current + 1} / $count",
                style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Black),
                color = brand.mutedForeground,
            )
        }

        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(AcerolaShapes.Full)
                    .background(brand.surface)
                    .padding(SpacingTokens.ExtraSmall),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.ExtraSmall),
        ) {
            repeat(count) { index ->
                val (container, content) =
                    when {
                        index == current -> MaterialTheme.colorScheme.primary to MaterialTheme.colorScheme.onPrimary
                        index < current -> brand.primarySoft to MaterialTheme.colorScheme.primary
                        else -> Color.Transparent to brand.mutedForeground
                    }
                // Numbers only on the phone, like the desktop on narrow windows; the name is the
                // step's accessible label.
                val stepLabel = "${index + 1} ${stringResource(id = labels[index])}"
                Box(
                    modifier =
                        Modifier
                            .weight(1f)
                            .height(36.dp)
                            .clip(AcerolaShapes.Full)
                            .background(container),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = "${index + 1}",
                        modifier = Modifier.semantics { contentDescription = stepLabel },
                        style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Black),
                        color = content,
                        maxLines = 1,
                    )
                }
            }
        }

        Acerola.Component.ProgressBar(progress = progress)
    }
}

// Brand moment: the mascot welcomes and "Acerola" is the brand lettering (Titan One is allowed on
// the welcome/splash), then three highlights in soft tones.
@Composable
fun WelcomeSlide() {
    val brand = AcerolaExtendedTheme.colors
    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = SpacingTokens.Huge, vertical = SpacingTokens.Large),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Huge, Alignment.CenterVertically),
    ) {
        Image(
            painter = painterResource(id = R.drawable.acerola_mascot),
            contentDescription = null,
            modifier = Modifier.size(144.dp),
        )
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
            Text(
                text = "Acerola",
                style = TextStyle(fontFamily = AcerolaFonts.Display, fontSize = 48.sp, lineHeight = 52.sp),
                color = brand.foreground,
            )
            Text(
                text = stringResource(id = R.string.tutorial_desc_welcome),
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
                color = brand.mutedForeground,
            )
        }
        Column(modifier = Modifier.widthIn(max = 420.dp), verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
            Highlight(Icons.Default.Bolt, Tone.Primary, R.string.tutorial_welcome_fast_title, R.string.tutorial_welcome_fast_desc)
            Highlight(Icons.Default.Share, Tone.Info, R.string.tutorial_welcome_p2p_title, R.string.tutorial_welcome_p2p_desc)
            Highlight(Icons.Default.VerifiedUser, Tone.Success, R.string.tutorial_welcome_local_title, R.string.tutorial_welcome_local_desc)
        }
    }
}

@Composable
private fun Highlight(
    icon: ImageVector,
    tone: Tone,
    title: Int,
    description: Int,
) {
    val brand = AcerolaExtendedTheme.colors
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(AcerolaShapes.Tile)
                .background(brand.surface)
                .padding(SpacingTokens.Medium),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        Acerola.Component.RoundIcon(icon = icon, tone = tone)
        Column {
            Text(
                text = stringResource(id = title),
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
                color = brand.foreground,
            )
            Text(text = stringResource(id = description), style = MaterialTheme.typography.bodySmall, color = brand.mutedForeground)
        }
    }
}

@Composable
fun FormatsSlide() {
    val brand = AcerolaExtendedTheme.colors
    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Small),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
            Acerola.Component.RoundIcon(icon = Icons.Default.FolderZip, tone = Tone.Primary, size = 56.dp)
            Text(
                text = stringResource(id = R.string.tutorial_formats_title),
                style = MaterialTheme.typography.headlineMedium,
                textAlign = TextAlign.Center,
                color = brand.foreground,
            )
            Text(
                text = stringResource(id = R.string.tutorial_formats_subtitle),
                style = MaterialTheme.typography.bodyMedium,
                color = brand.mutedForeground,
                textAlign = TextAlign.Center,
            )
        }

        FormatCard(
            extension = "CBZ",
            name = stringResource(id = R.string.tutorial_formats_cbz_title),
            badge = stringResource(id = R.string.tutorial_formats_cbz_badge),
            description = stringResource(id = R.string.tutorial_formats_cbz_desc),
            features = listOf(stringResource(id = R.string.tutorial_formats_cbz_feat_1), stringResource(id = R.string.tutorial_formats_cbz_feat_2)),
            icon = Icons.Default.FolderZip,
            tone = Tone.Primary,
        )
        FormatCard(
            extension = "CBR",
            name = stringResource(id = R.string.tutorial_formats_cbr_title),
            badge = stringResource(id = R.string.tutorial_formats_cbr_badge),
            description = stringResource(id = R.string.tutorial_formats_cbr_desc),
            features = listOf(stringResource(id = R.string.tutorial_formats_cbr_feat_1), stringResource(id = R.string.tutorial_formats_cbr_feat_2)),
            icon = Icons.Default.Layers,
            tone = Tone.Info,
        )
        FormatCard(
            extension = "PDF",
            name = stringResource(id = R.string.tutorial_formats_pdf_title),
            badge = stringResource(id = R.string.tutorial_formats_pdf_badge),
            description = stringResource(id = R.string.tutorial_formats_pdf_desc),
            features = listOf(stringResource(id = R.string.tutorial_formats_pdf_feat_1), stringResource(id = R.string.tutorial_formats_pdf_feat_2)),
            icon = Icons.Default.PictureAsPdf,
            tone = Tone.Warning,
        )

        // Info notice (`rounded-card bg-info-soft text-info`).
        val info = Tone.Info.colors()
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(AcerolaShapes.Card)
                    .background(info.container)
                    .padding(SpacingTokens.ExtraLarge),
            horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
        ) {
            Icon(imageVector = Icons.Default.AutoAwesome, contentDescription = null, tint = info.content)
            Text(
                text =
                    buildAnnotatedString {
                        withStyle(SpanStyle(fontWeight = FontWeight.Black)) {
                            append(stringResource(id = R.string.tutorial_formats_sync_note_title))
                        }
                        append(": ")
                        append(stringResource(id = R.string.tutorial_formats_sync_note_content))
                    },
                style = MaterialTheme.typography.bodyMedium,
                color = info.content,
            )
        }
    }
}

// One format, as a brand card (`rounded-card bg-card p-5 shadow-lift`): round icon and badge in
// the format's tone, the extension in `font-black`, and its features on a `bg-surface` tile.
@Composable
private fun FormatCard(
    extension: String,
    name: String,
    badge: String,
    description: String,
    features: List<String>,
    icon: ImageVector,
    tone: Tone,
) {
    val brand = AcerolaExtendedTheme.colors
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .lift(AcerolaShapes.Card)
                .clip(AcerolaShapes.Card)
                .background(brand.card)
                .padding(SpacingTokens.ExtraLarge),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
    ) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            ProvideSurfaceLevel(AcerolaSurfaceLevel.Card) { Acerola.Component.RoundIcon(icon = icon, tone = tone) }
            Acerola.Component.Badge(text = badge, tone = tone)
        }
        Text(text = extension, style = MaterialTheme.typography.headlineMedium, color = brand.foreground)
        Text(text = name, style = MaterialTheme.typography.labelMedium, color = brand.mutedForeground)
        Text(text = description, style = MaterialTheme.typography.bodyMedium, color = brand.mutedForeground)
        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .padding(top = SpacingTokens.Small)
                    .clip(AcerolaShapes.Tile)
                    .background(brand.surface)
                    .padding(SpacingTokens.Medium),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            features.forEach { feature ->
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                    Icon(
                        imageVector = Icons.Default.CheckCircle,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(16.dp),
                    )
                    Text(
                        text = feature,
                        style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.SemiBold),
                        color = brand.foreground,
                    )
                }
            }
        }
    }
}

@Composable
fun SettingsSlide(
    folderName: String?,
    onFolderSelected: (Uri?) -> Unit,
) {
    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .padding(horizontal = SpacingTokens.Large),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large, Alignment.CenterVertically),
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
            Text(
                text = stringResource(id = R.string.tutorial_title_settings),
                style = MaterialTheme.typography.headlineMedium,
                textAlign = TextAlign.Center,
                color = AcerolaExtendedTheme.colors.foreground,
            )
            Text(
                text = stringResource(id = R.string.tutorial_desc_settings),
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center,
                color = AcerolaExtendedTheme.colors.mutedForeground,
            )
        }

        Main.Config.Component.SelectComicDirectory(
            folderName = folderName,
            onFolderSelected = onFolderSelected,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

@Composable
fun CompleteSlide() {
    val brand = AcerolaExtendedTheme.colors
    Column(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Large),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Huge, Alignment.CenterVertically),
    ) {
        Acerola.Component.RoundIcon(icon = Icons.Default.CheckCircle, tone = Tone.Success, size = 96.dp)
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
            Text(
                text = stringResource(id = R.string.tutorial_title_complete),
                style = MaterialTheme.typography.displaySmall,
                textAlign = TextAlign.Center,
                color = brand.foreground,
            )
            Text(
                text = stringResource(id = R.string.tutorial_desc_complete),
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
                color = brand.mutedForeground,
            )
        }

        Column(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .lift(AcerolaShapes.Card)
                    .clip(AcerolaShapes.Card)
                    .background(brand.card)
                    .padding(SpacingTokens.ExtraLarge),
            verticalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                Icon(imageVector = Icons.Default.AutoAwesome, contentDescription = null, tint = brand.success, modifier = Modifier.size(16.dp))
                Text(
                    text = stringResource(id = R.string.tutorial_checklist_title),
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Black),
                    color = brand.foreground,
                )
            }
            listOf(R.string.tutorial_checklist_formats, R.string.tutorial_checklist_folder, R.string.tutorial_checklist_sync).forEach { item ->
                Row(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .clip(AcerolaShapes.Tile)
                            .background(brand.surface)
                            .padding(SpacingTokens.Medium),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
                ) {
                    Icon(imageVector = Icons.Default.CheckCircle, contentDescription = null, tint = brand.success, modifier = Modifier.size(16.dp))
                    Text(text = stringResource(id = item), style = MaterialTheme.typography.bodyMedium, color = brand.foreground)
                }
            }
        }

        Text(
            text = stringResource(id = R.string.tutorial_note_complete),
            style = MaterialTheme.typography.bodyMedium,
            color = brand.mutedForeground,
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(AcerolaShapes.Card)
                    .background(brand.surface)
                    .padding(SpacingTokens.ExtraLarge),
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TutorialScreenContentPreview() {
    AcerolaTheme {
        TutorialScreenContent(
            folderName = null,
            onFolderSelected = {},
            onComplete = {},
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FormatsSlidePreview() {
    AcerolaTheme {
        FormatsSlide()
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun CompleteSlidePreview() {
    AcerolaTheme {
        CompleteSlide()
    }
}
