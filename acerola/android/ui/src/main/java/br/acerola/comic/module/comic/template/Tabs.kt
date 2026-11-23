package br.acerola.comic.module.comic.template

import android.content.res.Configuration
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.SegmentedControl
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.module.comic.state.MainTab

/** Chapters / preferences switch, as the brand chip tabs (`AcerolaChipTabs`). */
@Composable
fun Comic.Template.Tabs(
    totalChapters: Int,
    activeTab: MainTab,
    onTabSelected: (MainTab) -> Unit,
) {
    val tabs = MainTab.entries
    Acerola.Component.SegmentedControl(
        options =
            tabs.map { tab ->
                tab.takeIf { it == MainTab.CHAPTERS }?.let { stringResource(id = it.titleRes, totalChapters) }
                    ?: stringResource(id = tab.titleRes)
            },
        selectedIndex = tabs.indexOf(activeTab),
        onSelect = { onTabSelected(tabs[it]) },
        modifier = Modifier.padding(horizontal = SpacingTokens.Large, vertical = SpacingTokens.Large),
    )
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TabsPreview() {
    AcerolaTheme {
        Comic.Template.Tabs(
            totalChapters = 10,
            activeTab = MainTab.CHAPTERS,
            onTabSelected = {},
        )
    }
}
