package br.acerola.comic.usecase.chapter

import br.acerola.comic.adapter.contract.gateway.ChapterReadGateway
import br.acerola.comic.adapter.contract.gateway.ChapterSyncStatusGateway
import br.acerola.comic.adapter.contract.gateway.VolumeGateway
import br.acerola.comic.adapter.library.DirectoryEngine
import br.acerola.comic.config.preference.types.ChapterSortPreferenceData
import br.acerola.comic.config.preference.types.SortDirection
import br.acerola.comic.config.preference.types.VolumeViewType
import br.acerola.comic.dto.ChapterDto
import br.acerola.comic.dto.archive.ChapterPageDto
import br.acerola.comic.dto.archive.VolumeChapterGroupDto
import br.acerola.comic.local.translator.ui.toCombinedRegularDto
import br.acerola.comic.local.translator.ui.toCombinedVolumeDto
import br.acerola.comic.service.cache.ChapterCacheHandler
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import javax.inject.Inject

/**
 * UseCase specialized for ComicViewModel.
 * Orchestrates local data and volumes with an LRU cache.
 */
class ObserveCombinedChaptersUseCase
    @Inject
    constructor(
        @param:DirectoryEngine private val volumeGateway: VolumeGateway,
        @param:DirectoryEngine private val localReadGateway: ChapterReadGateway<ChapterPageDto>,
        @param:DirectoryEngine private val localSyncStatusGateway: ChapterSyncStatusGateway,
        private val cacheHandler: ChapterCacheHandler,
    ) {
        val progress: Flow<Int> = localSyncStatusGateway.progress

        val isIndexing: Flow<Boolean> = localSyncStatusGateway.isIndexing

        fun observeCombined(
            comicId: Long,
            remoteId: Long?,
            sort: ChapterSortPreferenceData,
            page: Int,
            pageSize: Int,
            viewMode: VolumeViewType,
            volumeOverrides: Map<Long, VolumeChapterGroupDto>,
        ): Flow<ChapterDto?> {
            val baseCacheKey =
                cacheHandler.generateKey(
                    comicId,
                    sort.type.name,
                    sort.direction == SortDirection.ASCENDING,
                    pageSize,
                    viewMode.name,
                    page,
                    "${remoteId}_${volumeOverrides.hashCode()}",
                )

            val localFlow =
                localReadGateway
                    .observeChapters(comicId, sort.type.name, sort.direction == SortDirection.ASCENDING)
                    .filter { it.pageSize != -1 }

            val volumeFlow =
                volumeGateway
                    .observeVolumeGroups(comicId, pageSize, sort.type.name, sort.direction == SortDirection.ASCENDING)
                    .filter { it.isEmpty() || it.first().totalChapters != -1 }

            return combine(
                localFlow,
                volumeFlow,
                volumeGateway.observeHasRootChapters(comicId),
            ) { localAll, volumeSections, hasRootChapters ->
                val isInitialState = localAll.pageSize == 0 && localAll.items.isEmpty()

                // The key includes a light "snapshot" of the current data (total + sum of lastModified).
                // Without it, the LRU cache answered with the old result even after the database Flow
                // emitted a new chapter list (e.g. after a rescan), since the key depended
                // only on the query parameters (comicId, sort, page...), never on the data itself.
                val dataSignature = "${localAll.total}_${localAll.items.sumOf { it.lastModified }}_${volumeSections.sumOf { it.totalChapters }}"
                val cacheKey = "${baseCacheKey}_$dataSignature"

                val cached = if (!isInitialState) cacheHandler.get(cacheKey) else null
                if (cached != null) return@combine cached

                val hasSubfolders = volumeSections.isNotEmpty()

                val effectiveViewMode =
                    if (hasSubfolders) {
                        if (viewMode == VolumeViewType.COVER_VOLUME) VolumeViewType.COVER_VOLUME else VolumeViewType.VOLUME
                    } else {
                        VolumeViewType.CHAPTER
                    }

                val result =
                    if (effectiveViewMode != VolumeViewType.CHAPTER) {
                        volumeSections.toCombinedVolumeDto(
                            volumeOverrides = volumeOverrides,
                            pageSize = pageSize,
                            effectiveViewMode = effectiveViewMode,
                        )
                    } else {
                        localAll.toCombinedRegularDto(
                            page = page,
                            pageSize = pageSize,
                            hasVolumeStructure = hasSubfolders,
                            effectiveViewMode = effectiveViewMode,
                        )
                    }

                if (!isInitialState) {
                    cacheHandler.put(cacheKey, result)
                }
                result
            }
        }
    }
