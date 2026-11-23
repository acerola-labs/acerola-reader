import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';
import * as m from '#lib/paraglide/messages.js';
import { LIBRARY_EVENTS } from '#lib/contracts/library/library.events.ts';
import { type DirectoryScanCommand } from '#lib/contracts/library/library.commands.ts';
import type { ErrorPayload } from '#lib/contracts/shared/shared.payloads.ts';
import { resolveErrorMessage } from '#lib/contracts/errors/errors.i18n.ts';
import { notificationStore } from '#lib/components/acerola-notification/acerola-notification.svelte';

const { notify, pop } = notificationStore;

export function useLibraryScanner(
	command: DirectoryScanCommand,
	getPath: () => string | undefined
) {
	let progressId: number | undefined;
	// Toast mirroring the same progress — updated in place (same `id`, `toastAsync` style)
	// instead of stacking one toast per event. Without it, a scan running in the background
	// only showed up in the notification bell: anyone not watching it had no way to know the
	// scan had started/finished/failed.
	let toastId: string | number | undefined;
	let scanning = $state(false);

	async function start() {
		const path = getPath();

		if (!path) {
			toast.error(m['hooks.comicScanner.noFolder']());
			return;
		}

		scanning = true;

		// Show the notification/toast IMMEDIATELY on click, not only when the first
		// `scan:progress` arrives from the backend — depending on how long the scan takes to
		// emit that first event, the toast could show up well after the click (sometimes
		// almost together with the completion one), at a moment that made no sense to the user.
		progressId = notify.info(m['hooks.comicScanner.inProgress'](), { duration: 0 });
		toastId = toast.loading(m['hooks.comicScanner.inProgress']());

		const unlistenProgress = await listen(LIBRARY_EVENTS.scanProgress, () => {});

		const unlistenConverting = await listen<string>(LIBRARY_EVENTS.scanConverting, (event) => {
			const msg = event.payload || m['hooks.comicScanner.converting']();

			if (progressId !== undefined) {
				pop(progressId);
			}

			progressId = notify.info(msg, { duration: 0 });
			toastId = toast.loading(msg, { id: toastId });
		});

		const unlisten = await listen(LIBRARY_EVENTS.scanComplete, () => {
			if (progressId !== undefined) {
				pop(progressId);
				progressId = undefined;
			}

			notify.success(m['hooks.comicScanner.success'](), { duration: 0 });
			toast.success(m['hooks.comicScanner.success'](), { id: toastId });
			toastId = undefined;

			scanning = false;

			unlisten();
			unlistenErr();
			unlistenProgress();
			unlistenConverting();
		});

		const unlistenErr = await listen<ErrorPayload>(LIBRARY_EVENTS.scanError, (it) => {
			if (progressId !== undefined) {
				pop(progressId);
				progressId = undefined;
			}

			const description = resolveErrorMessage(it.payload);
			notify.error(m['hooks.comicScanner.error.title'](), {
				description,
				duration: 0
			});
			toast.error(m['hooks.comicScanner.error.title'](), {
				description,
				id: toastId
			});
			toastId = undefined;

			scanning = false;

			unlisten();
			unlistenErr();
			unlistenProgress();
			unlistenConverting();
		});

		await invoke(command, { path });
	}

	return {
		start,
		get scanning() {
			return scanning;
		}
	};
}
