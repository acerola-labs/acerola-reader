import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
import { STORE_FILE, STORE_KEYS } from '#lib/constants/store-plugin.ts';
import { NETWORK_COMMANDS } from '#lib/contracts/network/network.commands.ts';
import type { RelayInfo } from '#lib/contracts/network/network.payloads.ts';

export function useRelaySettings() {
	let relayInfo = $state<RelayInfo | undefined>(undefined);

	async function loadRelayInfo() {
		relayInfo = await invoke<RelayInfo>(NETWORK_COMMANDS.getRelayInfo);
	}

	/// Every change below persists in `settings.json` AND applies right away
	/// (`apply_relay_settings` rereads disk/vault and RESTARTS the whole P2P module with the new
	/// config — see `NetworkServiceApi::restart` in the backend, LocalSend style) — no need to
	/// close/reopen the app anymore for it to take effect. A restart drops active connections
	/// for a few seconds, but avoids the confusing state mutating the live `Endpoint` could leave.

	async function setUseAcerolaRelay(value: boolean) {
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayUseAcerola, value);
		await store.save();

		if (relayInfo) relayInfo = { ...relayInfo, useAcerolaRelay: value };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function setUseIrohPublicNetwork(value: boolean) {
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayUseIrohPublic, value);
		await store.save();

		if (relayInfo) relayInfo = { ...relayInfo, useIrohPublicNetwork: value };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	/// Unlike the other sources: the ticket doesn't go to `settings.json` (tauri-plugin-store),
	/// it goes to the backend's encrypted vault (same protection as the P2P identity) — that's
	/// why it goes through the Tauri command, not the `store` directly. The value itself never
	/// comes back to the frontend; only `hasIrohServicesTicket` is updated after a successful
	/// save/clear. Propagates the backend error (e.g. malformed ticket) for the caller to
	/// handle (toast).
	async function setIrohServicesTicket(ticket: string) {
		await invoke(NETWORK_COMMANDS.setIrohServicesTicket, { ticket });
		if (relayInfo) relayInfo = { ...relayInfo, hasIrohServicesTicket: true };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function clearIrohServicesTicket() {
		await invoke(NETWORK_COMMANDS.clearIrohServicesTicket);
		if (relayInfo) relayInfo = { ...relayInfo, hasIrohServicesTicket: false };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function addCustomRelayUrl(url: string) {
		const trimmed = url.trim();
		if (!trimmed || !relayInfo || relayInfo.customRelayUrls.includes(trimmed)) return;

		const next = [...relayInfo.customRelayUrls, trimmed];
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayCustomUrls, next);
		await store.save();

		relayInfo = { ...relayInfo, customRelayUrls: next };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function removeCustomRelayUrl(url: string) {
		if (!relayInfo) return;

		const next = relayInfo.customRelayUrls.filter((existing) => existing !== url);
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayCustomUrls, next);
		await store.save();

		relayInfo = { ...relayInfo, customRelayUrls: next };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	/// Manual "Restart" button — the same operation any relay source change already fires
	/// automatically (see doc above), exposed so the user can force a reset without changing
	/// anything (e.g. a connection stuck after the OS switched physical networks, without
	/// having to close and reopen the whole app).
	async function restartP2p() {
		await invoke(NETWORK_COMMANDS.restartP2p);
	}

	return {
		loadRelayInfo,
		restartP2p,
		setUseAcerolaRelay,
		setUseIrohPublicNetwork,
		addCustomRelayUrl,
		removeCustomRelayUrl,
		setIrohServicesTicket,
		clearIrohServicesTicket,
		get relayInfo() {
			return relayInfo;
		},
		/// No relay source active — the natural "everything off" state, not an explicitly
		/// chosen mode (mirrors `RelaySettings::resolve` in the backend).
		get isMdnsOnly() {
			return (
				!!relayInfo &&
				!relayInfo.useAcerolaRelay &&
				!relayInfo.useIrohPublicNetwork &&
				relayInfo.customRelayUrls.length === 0
			);
		}
	};
}
