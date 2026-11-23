package br.acerola.comic.module.main.config.component

import android.content.res.Configuration
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Folder
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.SettingRow
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.main.Main
import br.acerola.comic.ui.R

@Composable
fun Main.Config.Component.SelectComicDirectory(
    folderName: String?,
    onFolderSelected: (Uri?) -> Unit,
    modifier: Modifier = Modifier,
) {
    val launcher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocumentTree(),
            onResult = { uri ->
                onFolderSelected(uri)
            },
        )

    Acerola.Component.SettingRow(
        title = stringResource(id = R.string.title_text_config_select_path_comic),
        description = folderName ?: stringResource(id = R.string.label_config_comic_folder_empty),
        icon = Icons.Filled.Folder,
        modifier = modifier,
        content = {
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_config_choose_folder),
                onClick = { launcher.launch(input = null) },
                variant = ButtonVariant.Secondary,
            )
        },
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun SelectComicDirectoryPreview() {
    AcerolaTheme {
        Main.Config.Component.SelectComicDirectory(
            folderName = "Comics",
            onFolderSelected = {},
        )
    }
}
