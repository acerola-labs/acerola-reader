package br.acerola.comic.module.comic.component

import androidx.compose.foundation.layout.Column
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.module.comic.Comic
import org.junit.Rule
import org.junit.Test

class SyncMetadataTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    private val remoteInfo = ComicMetadataDto(title = "Test Comic", description = "Desc", status = "Ongoing")

    // Mirrors the metadata section: the external sources only while external sync is on.
    private fun setContent(externalSyncEnabled: Boolean) {
        composeTestRule.setContent {
            AcerolaTheme {
                Column {
                    if (externalSyncEnabled) {
                        Comic.Component.ExternalMetadataSync(
                            remoteInfo = remoteInfo,
                            onSyncMangadexInfo = {},
                            onSyncAnilistInfo = {},
                        )
                    }
                    Comic.Component.ComicInfoSync(remoteInfo = remoteInfo, onSyncComicInfo = {})
                }
            }
        }
    }

    @Test
    fun should_display_mangadex_anilist_and_comicinfo_when_external_sync_enabled() {
        setContent(externalSyncEnabled = true)

        composeTestRule.onNodeWithText("Sincronização com MangaDex").assertIsDisplayed()
        composeTestRule.onNodeWithText("Sincronização com AniList").assertIsDisplayed()
        composeTestRule.onNodeWithText("ComicInfo.xml").assertIsDisplayed()
    }

    @Test
    fun should_not_display_mangadex_and_anilist_when_external_sync_disabled() {
        setContent(externalSyncEnabled = false)

        composeTestRule.onNodeWithText("Sincronização com MangaDex").assertDoesNotExist()
        composeTestRule.onNodeWithText("Sincronização com AniList").assertDoesNotExist()
        composeTestRule.onNodeWithText("ComicInfo.xml").assertIsDisplayed()
    }
}
