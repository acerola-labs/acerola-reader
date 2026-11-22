package br.acerola.comic.module.main.home
import android.content.Intent
import android.content.res.Configuration
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ViewList
import androidx.compose.material.icons.filled.FolderOpen
import androidx.compose.material.icons.filled.GridView
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material.icons.rounded.Bookmark
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.LayersClear
import androidx.compose.material.icons.rounded.PhoneAndroid
import androidx.compose.material.icons.rounded.Visibility
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.zIndex
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import br.acerola.comic.common.state.LocalSnackbarHostState
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.component.Button
import br.acerola.comic.common.ux.component.ButtonSurface
import br.acerola.comic.common.ux.component.ButtonVariant
import br.acerola.comic.common.ux.component.Dialog
import br.acerola.comic.common.ux.component.DialogButton
import br.acerola.comic.common.ux.component.EmptyState
import br.acerola.comic.common.ux.component.IconButton
import br.acerola.comic.common.ux.component.SelectionAction
import br.acerola.comic.common.ux.component.SelectionActionDock
import br.acerola.comic.common.ux.component.SelectionTopBar
import br.acerola.comic.common.ux.component.SnackbarVariant
import br.acerola.comic.common.ux.component.showSnackbar
import br.acerola.comic.common.ux.theme.AcerolaExtendedTheme
import br.acerola.comic.common.ux.theme.AcerolaShapes
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.common.ux.tokens.SizeTokens
import br.acerola.comic.common.ux.tokens.SpacingTokens
import br.acerola.comic.common.viewmodel.archive.FileSystemAccessViewModel
import br.acerola.comic.common.viewmodel.library.archive.ComicDirectoryViewModel
import br.acerola.comic.config.preference.types.ComicSortType
import br.acerola.comic.config.preference.types.HomeLayoutType
import br.acerola.comic.config.preference.types.HomeSortPreference
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.history.ReadingHistoryDto
import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.module.comic.ComicActivity
import br.acerola.comic.module.main.Main
import br.acerola.comic.module.main.common.component.BatchComicCategorySheet
import br.acerola.comic.module.main.common.component.ComicActionsSheet
import br.acerola.comic.module.main.common.component.ComicListItem
import br.acerola.comic.module.main.common.component.PeerPickerSheet
import br.acerola.comic.module.main.home.component.ComicGridItem
import br.acerola.comic.module.main.home.component.HomeContinueBanner
import br.acerola.comic.module.main.home.component.HomeFilterSheet
import br.acerola.comic.module.main.home.component.HomeSearchBar
import br.acerola.comic.module.main.home.state.FilterSettings
import br.acerola.comic.module.main.home.state.HomeAction
import br.acerola.comic.module.main.home.state.HomeUiState
import br.acerola.comic.module.main.remotelibrary.RemoteLibraryActivity
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.module.reader.ReaderActivity
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.ui.R
import br.acerola.comic.util.p2p.PairingCode
import kotlinx.coroutines.launch

private val homeTopOverlayHeight = 56.dp

// Toolbar under the search (filter, other device, layout, sync all), the desktop's home toolbar.
private val homeToolbarHeight = 44.dp
private val homeTopContentOffset = 6.dp
private val mainBottomBarHeight = 64.dp
private val selectionActionButtonHeight = 54.dp

@Composable
fun Main.Home.Template.Screen(
    homeViewModel: HomeViewModel = hiltViewModel(),
    fileSystemAccessViewModel: FileSystemAccessViewModel = hiltViewModel(),
    comicDirectoryViewModel: ComicDirectoryViewModel = hiltViewModel(),
    onNavigateToConfig: () -> Unit,
) {
    val context = LocalContext.current
    val snackbarHostState = LocalSnackbarHostState.current

    LaunchedEffect(Unit) {
        launch {
            homeViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            fileSystemAccessViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
        launch {
            comicDirectoryViewModel.uiEvents.collect { message ->
                snackbarHostState.showSnackbar(message.uiMessage.asString(context), SnackbarVariant.Error)
            }
        }
    }

    val layout by homeViewModel.selectedHomeLayout.collectAsStateWithLifecycle()
    val isIndexing by homeViewModel.isIndexing.collectAsStateWithLifecycle()
    val comics by homeViewModel.comics.collectAsStateWithLifecycle()
    val allCategories by homeViewModel.allCategories.collectAsStateWithLifecycle()
    val sortSettings by homeViewModel.sortSettings.collectAsStateWithLifecycle()
    val filterSettings by homeViewModel.filterSettings.collectAsStateWithLifecycle()
    val searchQuery by homeViewModel.searchQuery.collectAsStateWithLifecycle()
    val isSearchExpanded by homeViewModel.isSearchExpanded.collectAsStateWithLifecycle()
    val selectedComicIds by homeViewModel.selectedComicIds.collectAsStateWithLifecycle()
    val pairedPeers by homeViewModel.pairedPeers.collectAsStateWithLifecycle()
    val folderName by fileSystemAccessViewModel.folderName.collectAsStateWithLifecycle()

    val isSelectionMode = selectedComicIds.isNotEmpty()

    val selectedComics =
        remember(comics, selectedComicIds) {
            comics?.filter { selectedComicIds.contains(it.first.directory.id) } ?: emptyList()
        }

    val areAllSelectedHidden =
        remember(selectedComics) {
            selectedComics.isNotEmpty() && selectedComics.all { it.first.directory.hidden }
        }

    val selectedCategoryCounts =
        remember(selectedComics) {
            val counts = mutableMapOf<Long, Int>()
            selectedComics.forEach { item ->
                item.first.category?.id?.let { catId ->
                    counts[catId] = (counts[catId] ?: 0) + 1
                }
            }
            counts
        }

    val lastRead by remember(comics) {
        derivedStateOf {
            comics?.filter { it.second != null }?.maxByOrNull { it.second?.updatedAt ?: 0L }
        }
    }

    val uiState =
        HomeUiState(
            layout = layout,
            isIndexing = isIndexing,
            comics = comics,
            sortType = sortSettings.type,
            sortDirection = sortSettings.direction,
            filter = filterSettings,
        )

    val onAction: (HomeAction) -> Unit = { action ->
        when (action) {
            is HomeAction.UpdateLayout -> homeViewModel.updateHomeLayout(action.layout)
            is HomeAction.ClickManga -> {
                val intent =
                    Intent(context, ComicActivity::class.java).apply {
                        putExtra(ComicActivity.ChapterExtra.COMIC, action.comic)
                    }
                context.startActivity(intent)
            }

            is HomeAction.ClickContinue -> {
                val intent =
                    Intent(context, ReaderActivity::class.java).apply {
                        putExtra(ReaderActivity.PageExtra.INITIAL_PAGE, action.history.lastPage)
                        putExtra(ReaderActivity.PageExtra.MANGA_ID, action.comic.directory.id)
                        putExtra(ReaderActivity.PageExtra.CHAPTER_ID, action.history.chapterArchiveId)
                        putExtra(ReaderActivity.PageExtra.CHAPTER_SORT, action.history.chapterSort)
                    }
                context.startActivity(intent)
            }
        }
    }

    HomeScreenContent(
        uiState = uiState,
        allCategories = allCategories,
        sortSettings = sortSettings,
        filterSettings = filterSettings,
        searchQuery = searchQuery,
        isSearchExpanded = isSearchExpanded,
        selectedComicIds = selectedComicIds,
        pairedPeers = pairedPeers,
        isSelectionMode = isSelectionMode,
        areAllSelectedHidden = areAllSelectedHidden,
        selectedCategoryCounts = selectedCategoryCounts,
        lastRead = lastRead,
        hasFolder = folderName != null,
        onAction = onAction,
        onQuickSyncLibrary = comicDirectoryViewModel::syncLibrary,
        onSelectFolder = { uri ->
            if (uri != null) {
                fileSystemAccessViewModel.saveFolderUri(uri)
                comicDirectoryViewModel.syncLibrary()
            }
        },
        onUpdateSearchQuery = homeViewModel::updateSearchQuery,
        onSetSearchExpanded = homeViewModel::setSearchExpanded,
        onClearComicSelection = homeViewModel::clearComicSelection,
        onSelectAllComics = homeViewModel::selectAllComics,
        onToggleComicSelection = homeViewModel::toggleComicSelection,
        onLoadPairedPeers = homeViewModel::loadPairedPeers,
        onSyncAllWithPeer = homeViewModel::syncAllWithPeer,
        onBrowseRemoteLibrary = { peerId, peerDisplayName ->
            val intent =
                Intent(context, RemoteLibraryActivity::class.java).apply {
                    putExtra(RemoteLibraryActivity.PeerExtra.PEER_ID, peerId)
                    putExtra(RemoteLibraryActivity.PeerExtra.PEER_DISPLAY_NAME, peerDisplayName)
                }
            context.startActivity(intent)
        },
        onHideManga = homeViewModel::hideManga,
        onDeleteComic = homeViewModel::deleteComic,
        onClearMetadata = homeViewModel::clearMetadata,
        onSetMangaCategory = homeViewModel::setMangaCategory,
        onSyncComicWithPeer = homeViewModel::syncComicWithPeer,
        onSetSelectedComicsCategory = homeViewModel::setSelectedComicsCategory,
        onHideSelectedComics = homeViewModel::hideSelectedComics,
        onUnhideSelectedComics = homeViewModel::unhideSelectedComics,
        onDeleteSelectedComics = homeViewModel::deleteSelectedComics,
        onClearMetadataForSelectedComics = homeViewModel::clearMetadataForSelectedComics,
        onUpdateSortSettings = homeViewModel::updateSortSettings,
        onUpdateFilterSettings = homeViewModel::updateFilterSettings,
    )
}

@Composable
private fun HomeScreenContent(
    uiState: HomeUiState,
    allCategories: List<CategoryDto>,
    sortSettings: HomeSortPreference,
    filterSettings: FilterSettings,
    searchQuery: String,
    isSearchExpanded: Boolean,
    selectedComicIds: Set<Long>,
    pairedPeers: List<PairedPeer>,
    isSelectionMode: Boolean,
    areAllSelectedHidden: Boolean,
    selectedCategoryCounts: Map<Long, Int>,
    lastRead: Triple<ComicDto, ReadingHistoryDto?, Int>?,
    onAction: (HomeAction) -> Unit,
    onQuickSyncLibrary: () -> Unit,
    onSelectFolder: (Uri?) -> Unit,
    hasFolder: Boolean = false,
    onUpdateSearchQuery: (String) -> Unit,
    onSetSearchExpanded: (Boolean) -> Unit,
    onClearComicSelection: () -> Unit,
    onSelectAllComics: (List<Long>) -> Unit,
    onToggleComicSelection: (Long) -> Unit,
    onLoadPairedPeers: () -> Unit,
    onSyncAllWithPeer: (peerId: String) -> Unit,
    onBrowseRemoteLibrary: (peerId: String, peerDisplayName: String) -> Unit,
    onHideManga: (Long) -> Unit,
    onDeleteComic: (Long) -> Unit,
    onClearMetadata: (Long) -> Unit,
    onSetMangaCategory: (Long, Long?) -> Unit,
    onSyncComicWithPeer: (peerId: String, comicName: String, direction: SyncDirection) -> Unit,
    onSetSelectedComicsCategory: (Long?) -> Unit,
    onHideSelectedComics: () -> Unit,
    onUnhideSelectedComics: () -> Unit,
    onDeleteSelectedComics: () -> Unit,
    onClearMetadataForSelectedComics: () -> Unit,
    onUpdateSortSettings: (HomeSortPreference) -> Unit,
    onUpdateFilterSettings: (FilterSettings) -> Unit,
) {
    val haptic = LocalHapticFeedback.current

    var showFilterSheet by remember { mutableStateOf(false) }
    var selectedMangaForActions by remember { mutableStateOf<ComicDto?>(null) }
    var isBannerExpanded by remember { mutableStateOf(true) }
    var showRemoteLibraryPeerPicker by remember { mutableStateOf(false) }
    var showSyncAllPeerPicker by remember { mutableStateOf(false) }
    // Peer picked in the picker, awaiting explicit confirmation before firing (may
    // involve a lot of data) — `null` = no pending confirmation.
    var syncAllConfirmPeer by remember { mutableStateOf<PairedPeer?>(null) }

    var showBatchCategorySheet by remember { mutableStateOf(false) }
    var showBatchHideDialog by remember { mutableStateOf(false) }
    var showBatchDeleteDialog by remember { mutableStateOf(false) }
    var showBatchClearMetadataDialog by remember { mutableStateOf(false) }

    Box(modifier = Modifier.fillMaxSize()) {
        val comicList = uiState.comics
        val configuration = LocalConfiguration.current
        val isLandscape = configuration.orientation == Configuration.ORIENTATION_LANDSCAPE

        val statusBarHeight = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
        val topOverlayTopPadding =
            if (isLandscape) {
                statusBarHeight + SpacingTokens.Small
            } else {
                SpacingTokens.ExtraSmall
            }
        val searchBarTopPadding =
            if (isSearchExpanded && !isLandscape) {
                0.dp
            } else {
                topOverlayTopPadding
            }
        val toolbarTopPadding = topOverlayTopPadding + homeTopOverlayHeight + SpacingTokens.Medium
        val gridTopPadding = toolbarTopPadding + homeToolbarHeight + SpacingTokens.Medium + homeTopContentOffset
        val navBarHeight = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
        val bottomNavigationHeight = if (isLandscape) 0.dp else mainBottomBarHeight
        val bottomSafeAreaPadding = navBarHeight + bottomNavigationHeight
        val selectionDockHeight = selectionActionButtonHeight + SpacingTokens.Small + SpacingTokens.Small
        val selectionDockBottomPadding = bottomSafeAreaPadding + SpacingTokens.Small
        val selectionDockGridPadding =
            if (isSelectionMode) {
                selectionDockHeight + SpacingTokens.Small
            } else {
                0.dp
            }
        val fabGridPadding = SpacingTokens.Large
        val gridBottomPadding =
            bottomSafeAreaPadding +
                if (isSelectionMode) {
                    selectionDockGridPadding + SpacingTokens.Medium
                } else {
                    fabGridPadding
                }

        when {
            comicList == null -> LoadingSkeleton()
            // A search with no match keeps the search open ("Nenhum resultado"), not the empty library.
            comicList.isEmpty() && searchQuery.isBlank() && !isSearchExpanded ->
                EmptyState(
                    isIndexing = uiState.isIndexing,
                    hasFolder = hasFolder,
                    onQuickSync = onQuickSyncLibrary,
                    onFolderSelected = onSelectFolder,
                )
            else -> {
                Box(modifier = Modifier.fillMaxSize()) {
                    // Solid top (`bg-background`, like the desktop topbar): the grid scrolls under it.
                    Box(
                        modifier =
                            Modifier
                                .align(Alignment.TopCenter)
                                .zIndex(0.5f)
                                .fillMaxWidth()
                                .height(gridTopPadding - homeTopContentOffset)
                                .background(AcerolaExtendedTheme.colors.background),
                    )

                    if (!isSelectionMode && !isSearchExpanded) {
                        HomeToolbar(
                            layout = uiState.layout,
                            activeFilterCount =
                                listOf(
                                    filterSettings.showHidden,
                                    filterSettings.bookmarkCategoryId != null,
                                    filterSettings.metadataSource != null,
                                ).count { it },
                            onFilterClick = { showFilterSheet = true },
                            onBrowseRemoteClick = {
                                onLoadPairedPeers()
                                showRemoteLibraryPeerPicker = true
                            },
                            onToggleLayout = {
                                onAction(
                                    HomeAction.UpdateLayout(
                                        layout =
                                            when (uiState.layout) {
                                                HomeLayoutType.LIST -> HomeLayoutType.GRID
                                                HomeLayoutType.GRID -> HomeLayoutType.LIST
                                            },
                                    ),
                                )
                            },
                            onSyncAllClick = {
                                onLoadPairedPeers()
                                showSyncAllPeerPicker = true
                            },
                            modifier =
                                Modifier
                                    .align(Alignment.TopStart)
                                    .zIndex(1f)
                                    .padding(top = toolbarTopPadding),
                        )
                    }

                    if (!isSelectionMode) {
                        Main.Home.Component.HomeSearchBar(
                            query = searchQuery,
                            onQueryChange = onUpdateSearchQuery,
                            onSearch = onUpdateSearchQuery,
                            expanded = isSearchExpanded,
                            onExpandedChange = onSetSearchExpanded,
                            comics = comicList,
                            onComicClick = { comic ->
                                onSetSearchExpanded(false)
                                onAction(HomeAction.ClickManga(comic))
                            },
                            modifier =
                                Modifier
                                    .align(Alignment.TopCenter)
                                    .zIndex(1f)
                                    .fillMaxWidth(if (isLandscape) 0.7f else 1f)
                                    .padding(
                                        // Open, the search covers the whole width (the field keeps its own inset).
                                        start =
                                            when {
                                                isSearchExpanded -> SpacingTokens.None
                                                isLandscape -> SizeTokens.LandscapeSidePadding
                                                else -> SpacingTokens.Small
                                            },
                                        end =
                                            when {
                                                isSearchExpanded -> SpacingTokens.None
                                                isLandscape -> SizeTokens.LandscapeSidePadding
                                                else -> SpacingTokens.Small
                                            },
                                        top = searchBarTopPadding,
                                    ),
                        )
                    }

                    AnimatedVisibility(
                        visible = isSelectionMode,
                        enter = fadeIn() + slideInVertically { -it },
                        exit = fadeOut() + slideOutVertically { -it },
                        modifier =
                            Modifier
                                .align(Alignment.TopCenter)
                                .zIndex(2f)
                                .fillMaxWidth(if (isLandscape) 0.7f else 1f)
                                .padding(
                                    start = if (isLandscape) SizeTokens.LandscapeSidePadding else SpacingTokens.Small,
                                    end = if (isLandscape) SizeTokens.LandscapeSidePadding else SpacingTokens.Small,
                                    top = topOverlayTopPadding,
                                ),
                    ) {
                        val allIds = comicList.map { it.first.directory.id }
                        val isAllSelected = selectedComicIds.size == allIds.size && allIds.isNotEmpty()

                        Acerola.Component.SelectionTopBar(
                            selectedCount = selectedComicIds.size,
                            isAllSelected = isAllSelected,
                            onClear = onClearComicSelection,
                            onToggleSelectAll = {
                                if (isAllSelected) {
                                    onClearComicSelection()
                                } else {
                                    onSelectAllComics(allIds)
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }

                    val (lastComic, lastHistory, _) = lastRead ?: Triple(null, null, 0)

                    val gridCells =
                        when (uiState.layout) {
                            HomeLayoutType.GRID -> GridCells.Adaptive(minSize = SizeTokens.ComicGridMinSize)
                            HomeLayoutType.LIST -> GridCells.Fixed(count = 1)
                        }

                    LazyVerticalGrid(
                        columns = gridCells,
                        verticalArrangement = Arrangement.spacedBy(space = SpacingTokens.Small),
                        horizontalArrangement = Arrangement.spacedBy(space = SpacingTokens.Small),
                        contentPadding =
                            PaddingValues(
                                start = SpacingTokens.Large,
                                top = gridTopPadding,
                                end = SpacingTokens.Large,
                                bottom = gridBottomPadding,
                            ),
                    ) {
                        if (lastComic != null && lastHistory != null && !isSelectionMode) {
                            item(
                                key = "continue_banner",
                                span = { GridItemSpan(maxLineSpan) },
                            ) {
                                Main.Home.Component.HomeContinueBanner(
                                    comic = lastComic,
                                    history = lastHistory,
                                    isExpanded = isBannerExpanded && !isLandscape,
                                    onExpandedChange = { isBannerExpanded = it },
                                    onContinueClick = { onAction(HomeAction.ClickContinue(lastComic, lastHistory)) },
                                    onComicClick = { onAction(HomeAction.ClickManga(lastComic)) },
                                    isLandscape = isLandscape,
                                    modifier = Modifier.padding(bottom = SpacingTokens.Small),
                                )
                            }
                        }

                        items(
                            items = comicList,
                            key = { (comic, _, _) -> "manga_${comic.directory.id}" },
                        ) { (comic, history, chapterCount) ->
                            val isSelected = selectedComicIds.contains(comic.directory.id)

                            val onItemClick = {
                                if (isSelectionMode) {
                                    onToggleComicSelection(comic.directory.id)
                                } else {
                                    onAction(HomeAction.ClickManga(comic))
                                }
                            }

                            val onItemLongClick = {
                                haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                                onToggleComicSelection(comic.directory.id)
                            }

                            val onShowItemActions = {
                                if (isSelectionMode) {
                                    onToggleComicSelection(comic.directory.id)
                                } else {
                                    selectedMangaForActions = comic
                                }
                            }
                            val onPlayClick =
                                if (!isSelectionMode && history != null) {
                                    { onAction(HomeAction.ClickContinue(comic, history)) }
                                } else {
                                    null
                                }

                            when (uiState.layout) {
                                HomeLayoutType.GRID ->
                                    Main.Home.Component.ComicGridItem(
                                        comic = comic,
                                        history = history,
                                        chapterCount = chapterCount,
                                        isSelected = isSelected,
                                        isSelectionMode = isSelectionMode,
                                        onLongClick = onItemLongClick,
                                        onShowActions = onShowItemActions,
                                        onClick = onItemClick,
                                    )

                                HomeLayoutType.LIST ->
                                    Main.Common.Component.ComicListItem(
                                        comic = comic,
                                        chapterCount = chapterCount,
                                        subtitle = comic.remoteInfo?.authors?.name,
                                        isSelected = isSelected,
                                        isSelectionMode = isSelectionMode,
                                        onLongClick = onItemLongClick,
                                        onClick = onItemClick,
                                        onPlayClick = onPlayClick,
                                        onShowActions = onShowItemActions,
                                    )
                            }
                        }
                    }
                }
            }
        }

        if (showRemoteLibraryPeerPicker) {
            Main.Common.Component.PeerPickerSheet(
                peers = pairedPeers,
                onSelect = { peerId ->
                    showRemoteLibraryPeerPicker = false
                    val peerDisplayName = pairedPeers.find { it.peerId == peerId }?.deviceName ?: PairingCode.shortId(peerId)
                    onBrowseRemoteLibrary(peerId, peerDisplayName)
                },
                onDismiss = { showRemoteLibraryPeerPicker = false },
            )
        }

        if (showSyncAllPeerPicker) {
            Main.Common.Component.PeerPickerSheet(
                peers = pairedPeers,
                onSelect = { peerId ->
                    showSyncAllPeerPicker = false
                    syncAllConfirmPeer = pairedPeers.find { it.peerId == peerId }
                },
                onDismiss = { showSyncAllPeerPicker = false },
            )
        }

        syncAllConfirmPeer?.let { peer ->
            val peerLabel = peer.nickname ?: peer.deviceName ?: PairingCode.shortId(peer.peerId)
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { syncAllConfirmPeer = null },
                title = stringResource(id = R.string.title_sync_all_confirm, peerLabel),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_sync_all),
                        onClick = {
                            onSyncAllWithPeer(peer.peerId)
                            syncAllConfirmPeer = null
                        },
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { syncAllConfirmPeer = null },
                    )
                },
            ) {
                Text(text = stringResource(id = R.string.description_sync_all_confirm))
            }
        }

        AnimatedVisibility(
            visible = isSelectionMode,
            enter = slideInVertically { it } + fadeIn(),
            exit = slideOutVertically { it } + fadeOut(),
            modifier =
                Modifier
                    .align(Alignment.BottomCenter)
                    .zIndex(3f)
                    .padding(bottom = selectionDockBottomPadding),
        ) {
            Acerola.Component.SelectionActionDock(
                actions =
                    listOf(
                        SelectionAction(
                            icon = Icons.Rounded.Bookmark,
                            label = stringResource(id = R.string.action_bookmark),
                            onClick = { showBatchCategorySheet = true },
                        ),
                        SelectionAction(
                            icon = if (areAllSelectedHidden) Icons.Rounded.Visibility else Icons.Rounded.VisibilityOff,
                            label = stringResource(id = if (areAllSelectedHidden) R.string.action_unhide else R.string.action_hide),
                            onClick = { showBatchHideDialog = true },
                        ),
                        SelectionAction(
                            icon = Icons.Rounded.LayersClear,
                            label = stringResource(id = R.string.action_clear_metadata),
                            onClick = { showBatchClearMetadataDialog = true },
                        ),
                        SelectionAction(
                            icon = Icons.Rounded.Delete,
                            label = stringResource(id = R.string.action_delete),
                            onClick = { showBatchDeleteDialog = true },
                            isError = true,
                        ),
                    ),
                modifier =
                    Modifier
                        .fillMaxWidth(if (isLandscape) 0.6f else 1f)
                        .widthIn(max = SizeTokens.MaxContentWidth)
                        .padding(horizontal = SpacingTokens.Medium),
            )
        }

        val activeManga = selectedMangaForActions
        if (activeManga != null) {
            Main.Common.Component.ComicActionsSheet(
                comic = activeManga,
                categories = allCategories,
                onHide = { onHideManga(activeManga.directory.id) },
                onDelete = { onDeleteComic(activeManga.directory.id) },
                onClearMetadata = { onClearMetadata(activeManga.directory.id) },
                onBookmark = { categoryId -> onSetMangaCategory(activeManga.directory.id, categoryId) },
                onDismiss = { selectedMangaForActions = null },
                pairedPeers = pairedPeers,
                onLoadPairedPeers = onLoadPairedPeers,
                onSyncWithPeer = { peerId, direction ->
                    onSyncComicWithPeer(peerId, activeManga.directory.name, direction)
                },
            )
        }

        if (showBatchCategorySheet) {
            Main.Common.Component.BatchComicCategorySheet(
                categories = allCategories,
                categoryCounts = selectedCategoryCounts,
                totalSelectedCount = selectedComicIds.size,
                onSelectCategory = { categoryId ->
                    onSetSelectedComicsCategory(categoryId)
                    showBatchCategorySheet = false
                },
                onRemoveCategory = {
                    onSetSelectedComicsCategory(null)
                    showBatchCategorySheet = false
                },
                onDismiss = { showBatchCategorySheet = false },
            )
        }

        if (showBatchHideDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showBatchHideDialog = false },
                title =
                    stringResource(
                        id = if (areAllSelectedHidden) R.string.dialog_batch_unhide_title else R.string.dialog_batch_hide_title,
                        selectedComicIds.size,
                    ),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = if (areAllSelectedHidden) R.string.action_unhide else R.string.action_hide),
                        onClick = {
                            showBatchHideDialog = false
                            if (areAllSelectedHidden) {
                                onUnhideSelectedComics()
                            } else {
                                onHideSelectedComics()
                            }
                        },
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showBatchHideDialog = false },
                    )
                },
                content = {
                    Text(
                        text =
                            stringResource(
                                id = if (areAllSelectedHidden) R.string.dialog_batch_unhide_message else R.string.dialog_batch_hide_message,
                            ),
                    )
                },
            )
        }

        if (showBatchDeleteDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showBatchDeleteDialog = false },
                title = stringResource(id = R.string.dialog_batch_delete_title, selectedComicIds.size),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_delete),
                        onClick = {
                            showBatchDeleteDialog = false
                            onDeleteSelectedComics()
                        },
                        variant = ButtonVariant.Destructive,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showBatchDeleteDialog = false },
                    )
                },
                content = { Text(text = stringResource(id = R.string.dialog_batch_delete_message, selectedComicIds.size)) },
            )
        }

        if (showBatchClearMetadataDialog) {
            Acerola.Component.Dialog(
                show = true,
                onDismiss = { showBatchClearMetadataDialog = false },
                title = stringResource(id = R.string.dialog_batch_clear_metadata_title, selectedComicIds.size),
                confirmButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_comic_preferences_clear),
                        onClick = {
                            showBatchClearMetadataDialog = false
                            onClearMetadataForSelectedComics()
                        },
                        variant = ButtonVariant.Destructive,
                    )
                },
                dismissButtonContent = {
                    Acerola.Component.DialogButton(
                        text = stringResource(id = R.string.action_cancel),
                        onClick = { showBatchClearMetadataDialog = false },
                    )
                },
                content = { Text(text = stringResource(id = R.string.dialog_batch_clear_metadata_message, selectedComicIds.size)) },
            )
        }

        if (showFilterSheet) {
            Main.Home.Component.HomeFilterSheet(
                sortSettings = sortSettings,
                filterSettings = filterSettings,
                categories = allCategories,
                onDismiss = { showFilterSheet = false },
                onSortChange = onUpdateSortSettings,
                onFilterChange = onUpdateFilterSettings,
            )
        }
    }
}

// Desktop home toolbar: "Filter and sort" (with the active filter count) and "Browse another
// device" as secondary pills, then layout and sync-all as round buttons. Scrolls sideways on narrow
// screens instead of squeezing the labels.
@Composable
private fun HomeToolbar(
    layout: HomeLayoutType,
    activeFilterCount: Int,
    onFilterClick: () -> Unit,
    onBrowseRemoteClick: () -> Unit,
    onToggleLayout: () -> Unit,
    onSyncAllClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .horizontalScroll(rememberScrollState())
                .padding(horizontal = SpacingTokens.Large),
        horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box {
            Acerola.Component.Button(
                text = stringResource(id = R.string.action_home_filter),
                onClick = onFilterClick,
                variant = ButtonVariant.Secondary,
                icon = Icons.Default.Tune,
            )
            if (activeFilterCount > 0) {
                Text(
                    text = activeFilterCount.toString(),
                    style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Black),
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier =
                        Modifier
                            .align(Alignment.TopEnd)
                            .size(20.dp)
                            .background(MaterialTheme.colorScheme.primary, CircleShape)
                            .wrapContentSize(Alignment.Center),
                )
            }
        }
        Acerola.Component.Button(
            text = stringResource(id = R.string.action_home_browse_remote),
            onClick = onBrowseRemoteClick,
            variant = ButtonVariant.Secondary,
            icon = Icons.Rounded.PhoneAndroid,
        )
        Acerola.Component.IconButton(
            icon = if (layout == HomeLayoutType.GRID) Icons.AutoMirrored.Filled.ViewList else Icons.Default.GridView,
            contentDescription = stringResource(id = R.string.description_icon_home_change_layout),
            onClick = onToggleLayout,
        )
        Acerola.Component.IconButton(
            icon = Icons.Default.Sync,
            contentDescription = stringResource(id = R.string.description_icon_home_sync_all_peer),
            onClick = onSyncAllClick,
        )
    }
}

@Composable
private fun EmptyState(
    isIndexing: Boolean,
    hasFolder: Boolean,
    onQuickSync: () -> Unit,
    onFolderSelected: (Uri?) -> Unit,
) {
    val launcher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocumentTree(),
            onResult = { uri ->
                onFolderSelected(uri)
            },
        )

    val infiniteTransition = rememberInfiniteTransition(label = "sync_spin")
    val angle by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec =
            infiniteRepeatable(
                animation = tween(durationMillis = 1000, easing = LinearEasing),
                repeatMode = RepeatMode.Restart,
            ),
        label = "spin_angle",
    )

    Box(
        modifier =
            Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(SpacingTokens.Large),
        contentAlignment = Alignment.Center,
    ) {
        // Acerola standard empty state: mascot + title + one phrase + ONE action (rescan when a
        // folder is already chosen, pick one otherwise).
        Acerola.Component.EmptyState(
            title = stringResource(id = R.string.description_text_home_empty_state),
            hint = stringResource(id = R.string.description_text_home_empty_subtitle),
        ) {
            Acerola.Component.ButtonSurface(
                onClick = { if (hasFolder) onQuickSync() else launcher.launch(null) },
                variant = ButtonVariant.Primary,
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Small)) {
                    Icon(
                        imageVector = if (hasFolder) Icons.Default.Refresh else Icons.Default.FolderOpen,
                        contentDescription = null,
                        modifier =
                            Modifier
                                .size(18.dp)
                                .graphicsLayer { rotationZ = if (hasFolder && isIndexing) angle else 0f },
                    )
                    Text(
                        text =
                            stringResource(
                                id = if (hasFolder) R.string.action_home_quick_sync else R.string.action_home_select_folder,
                            ),
                        style = MaterialTheme.typography.labelLarge,
                    )
                }
            }
        }
    }
}

// Loading: skeleton in the shape of the cover grid, never a blank screen or a loose spinner.
@Composable
private fun LoadingSkeleton() {
    val pulse by rememberInfiniteTransition(label = "skeleton_pulse").animateFloat(
        initialValue = 0.5f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(animation = tween(durationMillis = 900), repeatMode = RepeatMode.Reverse),
        label = "skeleton_alpha",
    )

    Column(
        modifier = Modifier.fillMaxSize().padding(SpacingTokens.Medium),
        verticalArrangement = Arrangement.spacedBy(SpacingTokens.Medium),
    ) {
        repeat(3) {
            Row(horizontalArrangement = Arrangement.spacedBy(SpacingTokens.Medium)) {
                repeat(3) {
                    Box(
                        modifier =
                            Modifier
                                .weight(1f)
                                .aspectRatio(2f / 3f)
                                .graphicsLayer { alpha = pulse }
                                .clip(AcerolaShapes.Tile)
                                .background(AcerolaExtendedTheme.colors.surface),
                    )
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun EmptyStatePreview() {
    AcerolaTheme {
        EmptyState(
            isIndexing = false,
            hasFolder = false,
            onQuickSync = {},
            onFolderSelected = {},
        )
    }
}

@Preview(name = "Light — empty library", showBackground = true)
@Preview(name = "Dark — empty library", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun HomeScreenContentEmptyPreview() {
    AcerolaTheme {
        HomeScreenContent(
            uiState = HomeUiState(comics = null),
            allCategories = emptyList(),
            sortSettings = HomeSortPreference(type = ComicSortType.TITLE, direction = SortDirection.ASCENDING),
            filterSettings = FilterSettings(),
            searchQuery = "",
            isSearchExpanded = false,
            selectedComicIds = emptySet(),
            pairedPeers = emptyList(),
            isSelectionMode = false,
            areAllSelectedHidden = false,
            selectedCategoryCounts = emptyMap(),
            lastRead = null,
            onAction = {},
            onQuickSyncLibrary = {},
            onSelectFolder = {},
            onUpdateSearchQuery = {},
            onSetSearchExpanded = {},
            onClearComicSelection = {},
            onSelectAllComics = {},
            onToggleComicSelection = {},
            onLoadPairedPeers = {},
            onSyncAllWithPeer = {},
            onBrowseRemoteLibrary = { _, _ -> },
            onHideManga = {},
            onDeleteComic = {},
            onClearMetadata = {},
            onSetMangaCategory = { _, _ -> },
            onSyncComicWithPeer = { _, _, _ -> },
            onSetSelectedComicsCategory = {},
            onHideSelectedComics = {},
            onUnhideSelectedComics = {},
            onDeleteSelectedComics = {},
            onClearMetadataForSelectedComics = {},
            onUpdateSortSettings = {},
            onUpdateFilterSettings = {},
        )
    }
}
