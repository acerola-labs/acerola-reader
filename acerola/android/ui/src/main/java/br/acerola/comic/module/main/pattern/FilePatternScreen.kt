package br.acerola.comic.module.main.pattern

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.rounded.Rule
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.ScreenHeader
import br.acerola.comic.common.ux.component.SettingSection
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.dto.archive.ArchiveTemplateDto
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.pattern.component.AddTemplateDialog
import br.acerola.comic.module.main.pattern.component.TemplateItem
import br.acerola.comic.module.main.pattern.state.FilePatternAction
import br.acerola.comic.module.main.pattern.state.FilePatternUiState
import br.acerola.comic.ui.R
import br.acerola.comic.util.sort.SortType

@Composable
fun Main.Pattern.Template.FilePatternScreen(
    viewModel: FilePatternViewModel = hiltViewModel(),
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current
    val uiState by viewModel.uiState.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.uiEvents.collect { message ->
            snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
        }
    }

    FilePatternLayout(
        uiState = uiState,
        onAction = viewModel::onAction,
        onBack = onBack,
    )
}

// Same page as the desktop's /config/templates: brand header with back and the create action,
// then one section per kind (chapter, volume) with a surface row per template.
@Composable
private fun FilePatternLayout(
    uiState: FilePatternUiState,
    onAction: (FilePatternAction) -> Unit,
    onBack: () -> Unit,
) {
    var showAddDialog by remember { mutableStateOf(false) }
    var editingTemplate by remember { mutableStateOf<ArchiveTemplateDto?>(null) }
    val groupedTemplates = uiState.templates.groupBy { it.type }

    LazyColumn(
        modifier =
            Modifier
                .fillMaxSize()
                .background(AcerolaExtendedTheme.colors.background),
        contentPadding = PaddingValues(bottom = SpacingTokens.SuperGiant),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Large),
    ) {
        item {
            Acerola.Component.ScreenHeader(
                eyebrow = stringResource(id = R.string.label_template_eyebrow),
                title = stringResource(id = R.string.label_template_config_activity),
                lead = stringResource(id = R.string.description_template_screen),
                onBack = onBack,
                actions = {
                    Acerola.Component.Button(
                        text = stringResource(id = R.string.action_add_template),
                        onClick = { showAddDialog = true },
                        icon = Icons.Default.Add,
                    )
                },
            )
        }

        SortType.entries.forEach { type ->
            item(key = "section_$type") {
                val templates = groupedTemplates[type].orEmpty()
                Acerola.Component.SettingSection(
                    title =
                        stringResource(
                            id = if (type == SortType.CHAPTER) R.string.title_template_chapters else R.string.title_template_volumes,
                        ),
                    modifier = Modifier.padding(horizontal = SpacingTokens.Large),
                ) {
                    if (templates.isEmpty()) {
                        Acerola.Component.EmptyState(
                            title =
                                stringResource(
                                    id =
                                        if (type == SortType.CHAPTER) {
                                            R.string.label_template_empty_chapter
                                        } else {
                                            R.string.label_template_empty_volume
                                        },
                                ),
                            icon = Icons.Rounded.Rule,
                            modifier = Modifier.align(Alignment.CenterHorizontally),
                        )
                    } else {
                        templates.forEach { template ->
                            Main.Pattern.Component.TemplateItem(
                                template = template,
                                onEdit = { editingTemplate = template },
                                onDelete = { onAction(FilePatternAction.DeleteTemplate(template.id)) },
                            )
                        }
                    }
                }
            }
        }
    }

    if (showAddDialog) {
        Main.Pattern.Component.AddTemplateDialog(
            onDismiss = { showAddDialog = false },
            onConfirm = { label, pattern, type ->
                onAction(FilePatternAction.AddTemplate(label, pattern, type))
                showAddDialog = false
            },
        )
    }

    editingTemplate?.let { template ->
        Main.Pattern.Component.AddTemplateDialog(
            isEditMode = true,
            initialLabel = template.label,
            initialPattern = template.pattern,
            initialType = template.type,
            onDismiss = { editingTemplate = null },
            onConfirm = { label, pattern, type ->
                onAction(FilePatternAction.EditTemplate(template.id, label, pattern, type))
                editingTemplate = null
            },
        )
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun FilePatternScreenPreview() {
    AcerolaTheme {
        FilePatternLayout(
            uiState =
                FilePatternUiState(
                    templates =
                        listOf(
                            ArchiveTemplateDto(
                                id = 1L,
                                label = "Padrao Manga",
                                pattern = "{title} - {chapter}",
                                type = SortType.CHAPTER,
                                isDefault = true,
                            ),
                        ),
                ),
            onAction = {},
            onBack = {},
        )
    }
}
