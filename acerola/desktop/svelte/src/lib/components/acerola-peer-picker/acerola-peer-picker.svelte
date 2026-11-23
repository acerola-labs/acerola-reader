<script module lang="ts">
	import type { PairedPeerPayload } from '#lib/contracts/network/network.payloads.ts';

	export type AcerolaPeerPickerProps = {
		state: {
			open: boolean;
		};
		data: {
			peers: PairedPeerPayload[];
			/** Local nickname (set by the user on this device) to override the default
			 *  `deviceName` — same priority as `usePeerConnection().peerLabel`. */
			nicknameFor?: (peerId: string) => string | null | undefined;
		};
		events: {
			onOpenChange: (open: boolean) => void;
			onSelect: (peer: PairedPeerPayload) => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaDialog from '#lib/components/acerola-dialog/acerola-dialog.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import { shortId } from '#lib/utils/connection-code.util.ts';
	import MonitorIcon from '@lucide/svelte/icons/monitor';

	let { state, data, events }: AcerolaPeerPickerProps = $props();

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `events` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings down
	// the suite even with every assert passing (same pattern as
	// acerola-network-my-device-card.svelte).
	let safeEvents = $derived(events ?? { onOpenChange: () => {}, onSelect: () => {} });
	let safeData = $derived(data ?? { peers: [] });
</script>

<AcerolaDialog
	state={{ open: state?.open ?? false }}
	data={{ title: m['pages.network.peerPicker.title']() }}
	events={{ onOpenChange: safeEvents.onOpenChange }}
	ui={{ contentClass: 'sm:max-w-sm' }}
>
	{#if safeData.peers.length === 0}
		<AcerolaEmptyState
			data={{ title: m['pages.network.peers.empty'](), icon: MonitorIcon }}
			ui={{ class: 'py-4' }}
		/>
	{:else}
		<!-- List rows (brand book): rounded-tile on bg-surface, round soft icon, font-black name. -->
		<div class="flex flex-col gap-2">
			{#each safeData.peers as peer (peer.peerId)}
				<button
					type="button"
					class="flex items-center gap-3 rounded-tile bg-surface p-3 text-left transition-[filter] duration-200 ease-brand hover:brightness-110 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					onclick={() => safeEvents.onSelect(peer)}
				>
					<span
						class="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary"
					>
						<MonitorIcon size={18} />
					</span>
					<span class="truncate text-sm font-black tracking-tight text-foreground">
						{safeData.nicknameFor?.(peer.peerId) ??
							peer.deviceName ??
							shortId(peer.peerId)}
					</span>
				</button>
			{/each}
		</div>
	{/if}
</AcerolaDialog>
