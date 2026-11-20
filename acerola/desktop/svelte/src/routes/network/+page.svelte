<script lang="ts">
	import { getContext, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { error } from '@tauri-apps/plugin-log';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaRemoteLibraryDialog from '#lib/components/acerola-remote-library-dialog/acerola-remote-library-dialog.svelte';
	import AcerolaNetworkMyDeviceCard from './components/acerola-network-my-device-card.svelte';
	import AcerolaNetworkRelaySettingsCard from './components/acerola-network-relay-settings-card.svelte';
	import AcerolaNetworkPairingCard from './components/acerola-network-pairing-card.svelte';
	import AcerolaNetworkConnectCard from './components/acerola-network-connect-card.svelte';
	import AcerolaNetworkPeerList, {
		type DisplayPeer
	} from './components/acerola-network-peer-list.svelte';
	import AcerolaNetworkTransfersLog from './components/acerola-network-transfers-log.svelte';

	import { usePeerConnection } from '#lib/hooks/store/use-peer-connection.svelte.ts';
	import type { useNetworkSync } from '#lib/hooks/store/use-network-sync.svelte.ts';
	import { useRemoteLibrary } from '#lib/hooks/store/use-remote-library.svelte.ts';
	import { useRelaySettings } from '#lib/hooks/preferences/use-relay-settings.svelte.ts';
	import { NETWORK_COMMANDS } from '#lib/contracts/network/network.commands.ts';
	import { NETWORK_EVENTS } from '#lib/contracts/network/network.events.ts';
	import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';

	import ShieldAlertIcon from '@lucide/svelte/icons/shield-alert';
	import XIcon from '@lucide/svelte/icons/x';
	import NetworkIcon from '@lucide/svelte/icons/network';

	const peers = usePeerConnection();
	// Shared with `+layout.svelte` (never unmounts) via context — doesn't create its own
	// instance. `syncComic()` only resolves via a listener of the instance that called it; a
	// page-only instance (unmounted when navigating to another screen) forcibly rejected that
	// promise with "sync cancelled: listener stopped" even though the real sync kept going in
	// the backend. See `CONTEXT_KEYS.networkSync`.
	const sync = getContext<ReturnType<typeof useNetworkSync>>(CONTEXT_KEYS.networkSync);
	const relay = useRelaySettings();
	const remoteLibrary = useRemoteLibrary();

	let browsingPeerId = $state<string | null>(null);
	let peerPendingRemoval = $state<DisplayPeer | null>(null);
	// Not a toast: while true, the master key protecting identity/peers/trust lacks the OS
	// keyring's protection (see `security:keyring_unavailable` in the backend,
	// `infra::security::MasterKeySource::FallbackFile`) — it must stay visible until the user
	// dismisses it, not vanish on its own like a toast.
	let keyringUnavailable = $state(false);
	let keyringWarningDismissed = $state(false);

	onMount(() => {
		let unlistenKeyringWarning: UnlistenFn | undefined;

		(async () => {
			// `setup_network` (backend) runs in a separate async task and may finish before this
			// screen mounts — a listener alone would miss the event. Query the current state on
			// demand; the listener below covers the case where the backend hasn't finished yet.
			try {
				keyringUnavailable = await invoke<boolean>(NETWORK_COMMANDS.getSecurityStatus);
			} catch (err) {
				error(`failed to query security status: ${err}`);
			}

			unlistenKeyringWarning = await listen(NETWORK_EVENTS.keyringUnavailable, () => {
				keyringUnavailable = true;
			});

			await Promise.all([
				peers.loadLocalInfo(),
				peers.startListening(),
				remoteLibrary.startListening(),
				relay.loadRelayInfo()
			]);
		})();

		return () => {
			unlistenKeyringWarning?.();
			peers.stopListening();
			remoteLibrary.stopListening();
		};
	});

	// Merges the persisted paired peers (survive restarts, `deviceName` comes from
	// `known_peers()` in the backend — see `NetworkServiceApi::paired_peers`) with the ones
	// connected now (`network:status`, fresher but only exists for the few seconds the
	// handshake connection is actually open). Without the paired ones, the sync button would
	// only exist at the exact moment the peer is connected — almost never. `connected` is
	// derived from presence in `status.peers`; when overwriting with the "live" entry, it keeps
	// the already persisted `deviceName` as a fallback for the rare case where `device` hasn't
	// arrived at that exact moment (handshake just opened).
	const uniquePeers = $derived.by(() => {
		const byId = new Map<string, DisplayPeer>();
		for (const peer of peers.pairedPeers) {
			byId.set(peer.peerId, {
				peerId: peer.peerId,
				deviceName: peer.deviceName,
				nickname: peers.peerNicknames[peer.peerId] ?? null,
				connected: false
			});
		}

		for (const peer of peers.status?.peers ?? []) {
			byId.set(peer.peerId, {
				peerId: peer.peerId,
				deviceName: peer.device?.name ?? byId.get(peer.peerId)?.deviceName ?? null,
				nickname: peers.peerNicknames[peer.peerId] ?? null,
				connected: true
			});
		}
		return [...byId.values()];
	});

	function peerStatusLabel(peer: DisplayPeer): string {
		if (peer.connected) return m['pages.network.peers.online']();
		const timestamp = sync.lastSyncedAt(peer.peerId);
		if (!timestamp) return m['pages.network.peers.neverSynced']();
		return m['pages.network.peers.lastSynced']({ when: new Date(timestamp).toLocaleString() });
	}

	async function withSync(
		action: (peerId: string, addrs: number[]) => Promise<unknown>,
		peerId: string
	) {
		const addrs = peers.getKnownAddr(peerId);
		if (!addrs) return;

		try {
			await action(peerId, addrs);
		} catch (err) {
			toast.error(extractErrorMessage(err));
		}
	}

	async function openRemoteLibrary(peerId: string) {
		browsingPeerId = peerId;
		await withSync((id, addrs) => remoteLibrary.queryRemoteLibrary(id, addrs), peerId);
	}

	async function handleSyncRemoteComic(peerId: string, comicName: string) {
		// Comes from remote library browsing — the user can only pick a comic they don't have
		// yet, so it is always a pull.
		await withSync((id, addrs) => sync.syncComic(id, addrs, comicName, 'pull'), peerId);
	}
</script>

<div class="mx-auto w-full max-w-6xl space-y-8 p-6 sm:p-8">
	<!-- Header -->
	<div class="space-y-2">
		<div
			class="inline-flex items-center gap-2 rounded-full border border-border/60 bg-muted/40 px-3 py-1 text-xs font-semibold text-subtext backdrop-blur-xs"
		>
			<NetworkIcon size={13} class="text-primary" />
			<span>{m['pages.network.title']()}</span>
		</div>
		<h1 class="text-3xl font-black tracking-tight text-foreground sm:text-4xl">
			{m['pages.network.title']()}
		</h1>
		<p class="text-sm leading-relaxed font-normal text-subtext sm:text-base">
			{m['pages.network.subtitle']()}
		</p>
	</div>

	{#if keyringUnavailable && !keyringWarningDismissed}
		<div class="flex items-start gap-3 rounded-card bg-destructive-soft p-4">
			<ShieldAlertIcon size={18} class="mt-0.5 shrink-0 text-destructive" />
			<p class="flex-1 text-sm text-foreground">{m['pages.network.keyringUnavailable']()}</p>
			<AcerolaButtonIcon
				events={{ onClick: () => (keyringWarningDismissed = true) }}
				ui={{
					variant: 'ghost',
					tone: 'muted',
					class: 'size-8 shrink-0 rounded-inner p-1',
					title: m['pages.network.dismiss'](),
					'aria-label': m['pages.network.dismiss']()
				}}
			>
				<XIcon size={14} />
			</AcerolaButtonIcon>
		</div>
	{/if}

	<!-- 2 Columns: items on the left, QR Code pinned on the right ignoring the scroll -->
	<div class="grid grid-cols-1 items-start gap-8 lg:grid-cols-12">
		<!-- Left column: interactive items and device list -->
		<div class="flex flex-col gap-8 lg:col-span-7">
			<AcerolaNetworkMyDeviceCard
				data={{
					deviceName: peers.localDeviceInfo?.name,
					localId: peers.localId,
					mode: peers.status?.mode
				}}
				events={{ onRenameDevice: (name) => peers.setDeviceName(name) }}
			/>

			<AcerolaNetworkConnectCard
				data={{ connecting: peers.connecting }}
				events={{ onConnect: (code) => peers.connectWithCode(code) }}
			/>

			<!-- Connected Devices (in its own section with a skeleton when empty) -->
			<AcerolaNetworkPeerList
				data={{
					peers: uniquePeers,
					addrFor: (peerId) => peers.getKnownAddr(peerId),
					statusLabel: peerStatusLabel,
					isSyncing: (peerId, kind) => sync.isSyncing(peerId, kind)
				}}
				events={{
					onSyncHistory: (peerId) =>
						withSync((id, addrs) => sync.syncHistory(id, addrs), peerId),
					onSyncFiles: (peerId) =>
						withSync((id, addrs) => sync.syncFiles(id, addrs), peerId),
					onSyncAll: (peerId) => withSync((id, addrs) => sync.syncAll(id, addrs), peerId),
					onBrowseLibrary: openRemoteLibrary,
					onRemove: (peer) => (peerPendingRemoval = peer),
					onRename: (peerId, nickname) => peers.setPeerNickname(peerId, nickname)
				}}
			/>

			<!-- Relay Settings and History -->
			<div class="space-y-6">
				<AcerolaNetworkRelaySettingsCard
					data={relay.relayInfo}
					events={{
						onToggleAcerolaRelay: (value) => relay.setUseAcerolaRelay(value),
						onToggleIrohPublicNetwork: (value) => relay.setUseIrohPublicNetwork(value),
						onAddCustomRelayUrl: (url) => relay.addCustomRelayUrl(url),
						onRemoveCustomRelayUrl: (url) => relay.removeCustomRelayUrl(url),
						onSetIrohServicesTicket: (ticket) => relay.setIrohServicesTicket(ticket),
						onClearIrohServicesTicket: () => relay.clearIrohServicesTicket(),
						onRestart: async () => {
							await relay.restartP2p();
							toast.success(m['pages.network.relaySettings.restart.success']());
						}
					}}
				/>

				<!-- Transfers -->
				<AcerolaNetworkTransfersLog
					data={{ entries: sync.log, peerLabel: peers.peerLabel }}
					events={{
						onRefresh: async () => {
							try {
								await sync.refreshLog();
							} catch (err) {
								toast.error(extractErrorMessage(err));
							}
						},
						onClear: async () => {
							try {
								await sync.clearLog();
							} catch (err) {
								toast.error(extractErrorMessage(err));
							}
						}
					}}
				/>
			</div>
		</div>

		<!-- Right column: Pairing QR Card pinned, ignoring the scroll -->
		<div class="lg:sticky lg:top-20 lg:col-span-5">
			<AcerolaNetworkPairingCard data={{ code: peers.connectionCode() }} />
		</div>
	</div>
</div>

<!-- Browse remote library -->
<AcerolaRemoteLibraryDialog
	state={{ open: browsingPeerId !== null }}
	data={{
		peerLabel: browsingPeerId ? peers.peerLabel(browsingPeerId) : '',
		comics: browsingPeerId ? remoteLibrary.comicsFor(browsingPeerId) : [],
		isLoading: browsingPeerId ? remoteLibrary.isLoading(browsingPeerId) : false,
		errorMessage: browsingPeerId ? remoteLibrary.errorFor(browsingPeerId) : undefined,
		coverPathFor: (comicName) =>
			browsingPeerId ? remoteLibrary.coverPathFor(browsingPeerId, comicName) : undefined,
		isSyncing: () => (browsingPeerId ? sync.isSyncing(browsingPeerId, 'comic') : false)
	}}
	events={{
		onOpenChange: (open) => {
			if (!open) browsingPeerId = null;
		},
		onSelectComic: (comicName) => {
			if (browsingPeerId) handleSyncRemoteComic(browsingPeerId, comicName);
		}
	}}
/>

<!-- Confirm removal of a paired device -->
<AcerolaAlertDialog
	state={{ open: peerPendingRemoval !== null }}
	data={{
		title: m['pages.network.peers.removeConfirm.title']({
			peer: peerPendingRemoval ? peers.peerLabel(peerPendingRemoval.peerId) : ''
		}),
		description: m['pages.network.peers.removeConfirm.desc'](),
		cancelText: m['pages.network.peers.removeConfirm.cancel'](),
		actionText: m['pages.network.peers.removeConfirm.confirm']()
	}}
	ui={{ variant: 'destructive' }}
	events={{
		onOpenChange: (open) => {
			if (!open) peerPendingRemoval = null;
		},
		onCancel: () => (peerPendingRemoval = null),
		onAction: () => {
			const peerId = peerPendingRemoval?.peerId;
			peerPendingRemoval = null;
			if (peerId) peers.removePeer(peerId);
		}
	}}
/>
