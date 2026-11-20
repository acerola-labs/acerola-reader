// Global state mirroring whether a batch metadata sync (MangaDex/AniList, Config screen) is
// running right now — lets `useMetadataSync()` (used on the comic screen) disable the
// individual sync button while the batch runs, instead of the user clicking and only finding
// out via a `SyncInProgress` toast that the command was refused (the lock itself already
// exists in the backend, this is just the UI anticipating the state).
let syncingAll = $state(false);

export function setSyncingAll(value: boolean): void {
	syncingAll = value;
}

export function isSyncingAll(): boolean {
	return syncingAll;
}
