export type DeviceInfoPayload = {
	name: string;
	os: string;
	version: string;
};

export type ConnectedPeerPayload = {
	peerId: string;
	alpn: string;
	device: DeviceInfoPayload | null;
};

export type NetworkStatusPayload = {
	mode: 'local' | 'relay';
	peers: ConnectedPeerPayload[];
};

/** A peer paired at some point, with its last known address — survives restarts and does
 *  not depend on being connected now (see `NetworkServiceApi::paired_peers` in the backend).
 *  `deviceName` comes from `known_peers()` (persistent) — `null` only when this peer never
 *  answered the identity interview (DeviceInfo). */
export type PairedPeerPayload = {
	peerId: string;
	addrs: number[];
	deviceName: string | null;
};

/** Composable relay configuration, read from `settings.json` (see `RelaySettings` in the
 *  backend). `acerolaRelayUrl` is fixed, only for display even when `useAcerolaRelay` is
 *  `false`. Changing any field only takes effect on the next app start. */
export type RelayInfo = {
	acerolaRelayUrl: string;
	useAcerolaRelay: boolean;
	useIrohPublicNetwork: boolean;
	customRelayUrls: string[];
	/** Only tells WHETHER a ticket for the user's `services.iroh.computer` account has been
	 *  pasted and saved — the value itself is never returned to the frontend (it is a real
	 *  credential, kept in the encrypted vault, not in `settings.json`). */
	hasIrohServicesTicket: boolean;
};

/** Summary of a comic in the remote library (see `library_browse_handler.rs`) — just title +
 *  chapter count, nothing transferred yet. `coverVersion` reuses the peer's
 *  `comic_directory.last_modified` — used to decide whether `queryRemoteCover` needs to
 *  fetch a new cover before firing the request. */
export type ComicSummary = {
	comicName: string;
	chapterCount: number;
	coverVersion: number;
};

/** Payload of the `library:query:result` event. */
export type LibraryQueryResultPayload = {
	peerId: string;
	comics: ComicSummary[];
};

/** Payload of the `browse:cover:result` event (`cover_browse_handler.rs`, ALPN
 *  `acerola/browse-cover/1`). `path` is only filled when `status === 'changed'` — a local
 *  path (`<app_data_dir>/remote_covers/...`) resolved via `convertFileSrc` on the caller's
 *  side (see `resolveArtworkPath`, `artwork.utils.ts`). */
export type CoverQueryResultPayload = {
	peerId: string;
	comicName: string;
	status: 'not_modified' | 'changed' | 'unavailable';
	coverVersion: number | null;
	path: string | null;
};

export type CoverQueryErrorPayload = {
	peerId: string;
	comicName?: string;
	message: string;
};
