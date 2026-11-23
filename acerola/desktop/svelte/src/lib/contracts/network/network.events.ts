export const NETWORK_EVENTS = {
	status: 'network:status',
	/** Emitted by the `acerola_p2p` lib (not by Acerola) the exact moment a connection
	 *  handshake finishes — `deviceInfoReceived` on the side that connected,
	 *  `deviceInfoExchanged` on the side that accepted. `network:status` is never emitted on
	 *  its own when this happens (only when the `get_network_status` command is called
	 *  explicitly), so without listening to these two the UI stays frozen until a manual
	 *  refresh (F5) even with a peer already connected. */
	deviceInfoReceived: 'rpc:device_info_received',
	deviceInfoExchanged: 'rpc:device_info_exchanged',
	historyStarted: 'sync:history:started',
	historyComplete: 'sync:history:complete',
	historyError: 'sync:history:error',
	filesStarted: 'sync:files:started',
	filesProgress: 'sync:files:progress',
	filesComplete: 'sync:files:complete',
	filesError: 'sync:files:error',
	historyEntryStarted: 'sync:history-entry:started',
	historyEntryComplete: 'sync:history-entry:complete',
	historyEntryError: 'sync:history-entry:error',
	comicStarted: 'sync:comic:started',
	comicProgress: 'sync:comic:progress',
	comicComplete: 'sync:comic:complete',
	comicError: 'sync:comic:error',
	/** Result of `queryRemoteLibrary` — carries `{ peerId, comics }` (see
	 *  `LibraryQueryResultPayload`), emitted by `LibraryBrowseOutbound` in the backend
	 *  (`infra::sync::protocol::library_browse_handler`, ALPN `acerola/browse-library/1`,
	 *  same ALPN used by Android). */
	libraryQueryResult: 'library:query:result',
	libraryQueryError: 'library:query:error',
	/** Result of `queryRemoteCover` (`{ peerId, comicName, status, coverVersion, path }`,
	 *  see `CoverQueryResultPayload`) — `status` is `"not_modified"`, `"changed"` (with `path`
	 *  pointing to the freshly written local cache, resolved via `convertFileSrc`) or
	 *  `"unavailable"`. Emitted by `CoverBrowseOutbound` (`cover_browse_handler.rs`, ALPN
	 *  `acerola/browse-cover/1`). */
	coverQueryResult: 'browse:cover:result',
	coverQueryError: 'browse:cover:error',
	/** Emitted once at startup if the OS keyring is not available (see
	 *  `infra::security::MasterKeySource::FallbackFile` in the backend) — the master key fell
	 *  back to a local file without the OS's extra protection. Not a fatal error (everything
	 *  keeps working, still encrypted with AES-GCM), but the user needs to know. */
	keyringUnavailable: 'security:keyring_unavailable'
} as const;
