export type ChapterId = string;

let selectedIds = $state<Set<ChapterId>>(new Set());
let isSelectionMode = $state(false);

// ONLY FOR TESTING
export function _resetChapterSelectionState() {
	selectedIds = new Set();
	isSelectionMode = false;
}

/**
 * Hook to manage multiple chapter selection inside the comic screen.
 * Allows selecting one or more chapters for batch actions (mark read/unread).
 */
export function useChapterSelection() {
	function toggleSelection(id: ChapterId): void {
		const next = new Set(selectedIds);
		if (next.has(id)) {
			next.delete(id);
		} else {
			next.add(id);
		}
		selectedIds = next;
		isSelectionMode = selectedIds.size > 0;
	}

	function selectSingle(id: ChapterId): void {
		selectedIds = new Set([id]);
		isSelectionMode = true;
	}

	function selectAll(ids: ChapterId[]): void {
		selectedIds = new Set(ids);
		isSelectionMode = ids.length > 0;
	}

	function deselectAll(): void {
		selectedIds = new Set();
		isSelectionMode = false;
	}

	function enterSelectionMode(): void {
		isSelectionMode = true;
	}

	function exitSelectionMode(): void {
		selectedIds = new Set();
		isSelectionMode = false;
	}

	function isSelected(id: ChapterId): boolean {
		return selectedIds.has(id);
	}

	return {
		toggleSelection,
		selectSingle,
		selectAll,
		deselectAll,
		enterSelectionMode,
		exitSelectionMode,
		isSelected,
		get selectedIds() {
			return selectedIds;
		},
		get selectedCount() {
			return selectedIds.size;
		},
		get isSelectionMode() {
			return isSelectionMode;
		},
		get selectedIdsArray() {
			return Array.from(selectedIds);
		}
	};
}
