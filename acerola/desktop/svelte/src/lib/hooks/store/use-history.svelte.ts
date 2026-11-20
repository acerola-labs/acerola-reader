import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { HISTORY_COMMANDS } from '#lib/contracts/history/history.commands.ts';
import type { ReadingHistoryPayload } from '#lib/contracts/history/history.payloads.ts';
import { notificationStore } from '#lib/components/acerola-notification/acerola-notification.svelte';
import { m } from '#lib/paraglide/messages.js';

const { notify } = notificationStore;

export function useHistory() {
	let items = $state<ReadingHistoryPayload[]>([]);
	let loading = $state(false);

	async function fetch() {
		if (loading) return;
		loading = true;

		try {
			items = await invoke<ReadingHistoryPayload[]>(HISTORY_COMMANDS.getAll);
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.history.error.load'](), {
				description: errorMessage
			});
			toast.error(errorMessage);
		} finally {
			loading = false;
		}
	}

	async function clear() {
		try {
			await invoke(HISTORY_COMMANDS.clear);
			items = [];
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.history.error.clear'](), {
				description: errorMessage
			});
			toast.error(errorMessage);
		}
	}

	async function updateReading(
		comicId: string,
		chapterId: string,
		lastPage: number,
		isCompleted: boolean
	) {
		try {
			await invoke(HISTORY_COMMANDS.updateReading, {
				comicId,
				chapterId,
				lastPage,
				isCompleted
			});
		} catch (error) {
			console.error('Failed to update history', error);
		}
	}

	async function markChapterRead(comicId: string, chapterId: string) {
		try {
			await invoke(HISTORY_COMMANDS.markChapterRead, { comicId, chapterId });
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.history.error.markRead'](), {
				description: errorMessage
			});
			toast.error(errorMessage);
			throw error;
		}
	}

	async function unmarkChapterRead(comicId: string, chapterId: string) {
		try {
			await invoke(HISTORY_COMMANDS.unmarkChapterRead, { comicId, chapterId });
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.history.error.markUnread'](), {
				description: errorMessage
			});
			toast.error(errorMessage);
			throw error;
		}
	}

	async function markChaptersReadBatch(comicId: string, chapterIds: string[]) {
		try {
			return await invoke<number>(HISTORY_COMMANDS.markChaptersReadBatch, {
				comicId,
				chapterIds
			});
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.history.error.markManyRead'](), {
				description: errorMessage
			});
			toast.error(errorMessage);
			throw error;
		}
	}

	async function unmarkChaptersReadBatch(comicId: string, chapterIds: string[]) {
		try {
			return await invoke<number>(HISTORY_COMMANDS.unmarkChaptersReadBatch, {
				comicId,
				chapterIds
			});
		} catch (error) {
			const errorMessage = error as string;

			notify.error(m['hooks.history.error.markManyUnread'](), {
				description: errorMessage
			});
			toast.error(errorMessage);
			throw error;
		}
	}

	return {
		fetch,
		clear,
		updateReading,
		markChapterRead,
		unmarkChapterRead,
		markChaptersReadBatch,
		unmarkChaptersReadBatch,
		get items() {
			return items;
		},
		get loading() {
			return loading;
		}
	};
}
