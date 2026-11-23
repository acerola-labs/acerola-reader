use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use lru::LruCache;

use crate::cmd::events::summary::ChapterDto;

/// The same data source (SQLite) changes cost depending on the access pattern:
/// reopening an already-seen volume/sort/search does not need to run the query
/// again. Mirrors the reader's `LruCache` (`core/services/reader/mod.rs`), except
/// that here the "expensive page" is a query result instead of bytes
/// decoded from a file.
const CHAPTER_CACHE_CAPACITY: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChapterCacheKey {
    pub comic_directory_fk: i64,
    pub volume_id: Option<i64>,
    pub sort_by: String,
    pub search_query: Option<String>,
    // INFO: use-reader-navigation.svelte.ts fetches a specific chapter by
    // position via page=index/page_size=1 (same comic_directory_fk/volume_id/
    // sort_by/search_query as an "everything at once" fetch). Without page/page_size
    // in the key, that call would collide with another pagination's cache and
    // return the wrong chapter.
    pub page: i32,
    pub page_size: i32,
}

#[derive(Clone)]
pub struct ChapterCacheService {
    cache: Arc<Mutex<LruCache<ChapterCacheKey, ChapterDto>>>,
}

impl ChapterCacheService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CHAPTER_CACHE_CAPACITY).unwrap(),
            ))),
        }
    }

    pub fn get(&self, key: &ChapterCacheKey) -> Option<ChapterDto> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }

    pub fn put(&self, key: ChapterCacheKey, value: ChapterDto) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, value);
    }

    /// Called after rescan/deep rescan — the only operations that actually
    /// change which chapters/volumes exist on disk. Metadata sync
    /// (title, description, cover) does not invalidate, because it does not change
    /// chapter identity.
    pub fn invalidate_comic(&self, comic_directory_fk: i64) {
        let mut cache = self.cache.lock().unwrap();

        let stale_keys: Vec<ChapterCacheKey> = cache
            .iter()
            .filter(|(key, _)| key.comic_directory_fk == comic_directory_fk)
            .map(|(key, _)| key.clone())
            .collect();

        for key in stale_keys {
            cache.pop(&key);
        }
    }
}

impl Default for ChapterCacheService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dto() -> ChapterDto {
        use crate::cmd::events::summary::{ChapterPageDto, VolumeViewType};

        ChapterDto {
            archive: ChapterPageDto {
                items: vec![],
                volumes: vec![],
                page_size: 1_000_000,
                page: 0,
                total: 0,
                volume_sections: vec![],
            },
            show_volume_headers: false,
            has_volume_structure: false,
            effective_view_mode: VolumeViewType::Chapter,
        }
    }

    fn key(comic_directory_fk: i64, volume_id: Option<i64>) -> ChapterCacheKey {
        ChapterCacheKey {
            comic_directory_fk,
            volume_id,
            sort_by: "number_asc".to_string(),
            search_query: None,
            page: 0,
            page_size: 1_000_000,
        }
    }

    #[test]
    fn test_different_page_and_page_size_do_not_collide_in_cache() {
        let service = ChapterCacheService::new();

        let full_list_key = key(1, None);
        let single_chapter_key = ChapterCacheKey { page: 5, page_size: 1, ..key(1, None) };

        service.put(full_list_key.clone(), sample_dto());

        assert!(service.get(&full_list_key).is_some());
        assert!(service.get(&single_chapter_key).is_none());
    }

    #[test]
    fn test_get_returns_none_when_no_entry_exists() {
        let service = ChapterCacheService::new();
        assert!(service.get(&key(1, None)).is_none());
    }

    #[test]
    fn test_put_and_get_return_same_value() {
        let service = ChapterCacheService::new();
        let cache_key = key(1, None);

        service.put(cache_key.clone(), sample_dto());

        assert!(service.get(&cache_key).is_some());
    }

    #[test]
    fn test_invalidate_comic_removes_only_entries_for_that_comic() {
        let service = ChapterCacheService::new();
        let key_comic_1 = key(1, None);
        let key_comic_2 = key(2, None);

        service.put(key_comic_1.clone(), sample_dto());
        service.put(key_comic_2.clone(), sample_dto());

        service.invalidate_comic(1);

        assert!(service.get(&key_comic_1).is_none());
        assert!(service.get(&key_comic_2).is_some());
    }
}
