// Global counter that any screen reading `resolveCover`/`resolveBanner` (`artwork.utils.ts`)
// uses as an automatic cache-bust. Without it, each screen had to remember to set its own
// local timestamp every time an operation might have downloaded a new cover (MangaDex/AniList
// sync, batch sync, regenerate cover, P2P sync) — in practice several forgot (e.g. individual
// MangaDex/AniList sync never busted, the Home never busted anywhere), and the old image
// stayed in the WebView cache even with the file already overwritten on disk.
let version = $state(0);

export function bumpArtworkVersion(): void {
	version = Date.now();
}

export function getArtworkVersion(): number {
	return version;
}
