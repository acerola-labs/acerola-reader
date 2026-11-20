<script module lang="ts">
	export type NetworkRelaySettingsCardData = {
		acerolaRelayUrl: string;
		useAcerolaRelay: boolean;
		useIrohPublicNetwork: boolean;
		customRelayUrls: string[];
		hasIrohServicesTicket: boolean;
	};

	export type NetworkRelaySettingsCardProps = {
		data: NetworkRelaySettingsCardData | undefined;
		events: {
			onToggleAcerolaRelay: (value: boolean) => Promise<void>;
			onToggleIrohPublicNetwork: (value: boolean) => Promise<void>;
			onAddCustomRelayUrl: (url: string) => Promise<void>;
			onRemoveCustomRelayUrl: (url: string) => Promise<void>;
			onSetIrohServicesTicket: (ticket: string) => Promise<void>;
			onClearIrohServicesTicket: () => Promise<void>;
			onRestart: () => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import { slide } from 'svelte/transition';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import WifiIcon from '@lucide/svelte/icons/wifi';
	import ServerIcon from '@lucide/svelte/icons/server';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import CheckIcon from '@lucide/svelte/icons/check';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import AcerolaAccordionCard from '#lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaToggleCard from '#lib/components/acerola-toggle-card/acerola-toggle-card.svelte';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils/cn.util.ts';
	import { autoAnimateList } from '#lib/utils/auto-animate.util.ts';
	import { checkBadgePop } from '#lib/utils/check-badge-motion.util.ts';

	let { data, events }: NetworkRelaySettingsCardProps = $props();

	// Closed by default — most users never need to touch this (the Acerola relay comes on by
	// default), so it isn't worth taking up open space on the Network screen.
	let expanded = $state(false);
	let customUrlDraft = $state('');
	let customUrlError = $state(false);
	let ticketDraft = $state('');
	let ticketError = $state(false);
	let ticketSaving = $state(false);
	// The "custom relays" and "Iroh ticket" cards hide their configuration by default (less
	// always-visible text) — they expand on demand when clicked, in a toggle independent of
	// enabling/disabling the source itself (see the 3 cards in the template below).
	let customExpanded = $state(false);
	// Pre-expanded while no ticket is saved yet — without it, a first-time user would have to
	// guess they need to click something to see the ticket paste field (only read on the
	// first render, like `expanded` above).
	let ticketExpanded = $state(untrack(() => !data?.hasIrohServicesTicket));

	// Shared by EVERY action that restarts the P2P node underneath (relay toggle, custom URL
	// add/remove, manual button) — the two things missing before: (1) visual feedback that a
	// restart is in progress (the toggles had NONE, unlike the manual button, which already
	// had `restarting`) and (2) a reentrancy lock — a double click on the same control, or
	// touching another one while a previous restart hasn't finished, is now ignored instead
	// of firing a second node rebuild in parallel (the real root cause: two concurrent
	// rebuilds registered the SAME identity on the relay at once, and the relay silently
	// dropped one of them).
	let restarting = $state(false);
	let restartError = $state(false);

	async function runRestartingAction(action: () => Promise<void>) {
		if (restarting) return;
		restarting = true;
		restartError = false;
		try {
			await action();
		} catch {
			restartError = true;
		} finally {
			restarting = false;
		}
	}

	// Switching the active relay source (Acerola's own / Iroh public network) rebuilds the P2P
	// node with a different `RelayMap` — paired peers that don't converge on the same relay
	// stay unreachable until they do. Without a warning, the user has no way to know this
	// before clicking.
	let pendingSwitch = $state<(() => Promise<void>) | null>(null);

	function confirmSwitch(action: () => Promise<void>) {
		pendingSwitch = action;
	}

	let safeData = $derived(
		data ?? {
			acerolaRelayUrl: '',
			useAcerolaRelay: false,
			useIrohPublicNetwork: false,
			customRelayUrls: [],
			hasIrohServicesTicket: false
		}
	);

	const activeSourceCount = $derived(
		(safeData.useAcerolaRelay ? 1 : 0) + safeData.customRelayUrls.length
	);

	// Mirrors `RelaySettings::resolve` in the backend: the Iroh public network is exclusive
	// with the other sources, and no active source falls back to mDNS-only — they don't combine.
	const summary = $derived.by(() => {
		if (safeData.useIrohPublicNetwork) {
			return m['pages.network.relaySettings.summaryIrohPublic']();
		}
		if (activeSourceCount === 0) {
			return m['pages.network.relaySettings.summaryMdnsOnly']();
		}
		return m['pages.network.relaySettings.summaryActive']({ count: activeSourceCount });
	});

	// Subtitle of the custom relays card — folds the count text in instead of a separate
	// paragraph next to the list itself (which only shows when expanded).
	const customRelaysSubtitle = $derived(
		safeData.customRelayUrls.length === 0
			? m['pages.network.relaySettings.customRelays.empty']()
			: m['pages.network.relaySettings.customRelays.count']({
					count: safeData.customRelayUrls.length
				})
	);

	// Subtitle of the Iroh public network card — folds in the old "disabled until a ticket is
	// configured" paragraph (`use_iroh_public_network_disabled_hint`) instead of a separate
	// warning, since it only makes sense exactly when there is no ticket.
	const irohSubtitle = $derived(
		safeData.hasIrohServicesTicket
			? m['pages.network.relaySettings.useIrohPublicNetworkDesc']()
			: m['pages.network.relaySettings.useIrohPublicNetworkDisabledHint']()
	);

	function isValidUrl(value: string): boolean {
		try {
			const url = new URL(value);
			return url.protocol === 'http:' || url.protocol === 'https:';
		} catch {
			return false;
		}
	}

	function toggleAcerolaRelay(value: boolean) {
		confirmSwitch(() => events.onToggleAcerolaRelay(value));
	}

	function toggleIrohPublicNetwork(value: boolean) {
		confirmSwitch(() => events.onToggleIrohPublicNetwork(value));
	}

	function submitCustomUrl() {
		const trimmed = customUrlDraft.trim();
		if (!trimmed) return;
		if (!isValidUrl(trimmed)) {
			customUrlError = true;
			return;
		}
		customUrlError = false;
		customUrlDraft = '';
		runRestartingAction(() => events.onAddCustomRelayUrl(trimmed));
	}

	function removeCustomUrl(url: string) {
		runRestartingAction(() => events.onRemoveCustomRelayUrl(url));
	}

	async function submitTicket() {
		const trimmed = ticketDraft.trim();
		if (!trimmed || ticketSaving) return;

		ticketSaving = true;
		try {
			await events.onSetIrohServicesTicket(trimmed);
			ticketDraft = '';
			ticketError = false;
		} catch {
			ticketError = true;
		} finally {
			ticketSaving = false;
		}
	}

	async function removeTicket() {
		if (ticketSaving) return;
		ticketSaving = true;
		try {
			await events.onClearIrohServicesTicket();
		} finally {
			ticketSaving = false;
		}
	}

	function restart() {
		runRestartingAction(() => events.onRestart());
	}
</script>

<AcerolaAccordionCard
	data={{ title: m['pages.network.relaySettings.title'](), description: summary }}
	state={{ expanded }}
	events={{ onToggle: () => (expanded = !expanded) }}
	ui={{ iconClass: 'bg-accent-hero text-accent-hero-foreground' }}
>
	{#snippet icon()}
		<WifiIcon size={20} />
	{/snippet}

	<div class="space-y-3">
		<!-- Card 1: Acerola relay — a pure toggle, clicking the whole row turns it on/off. -->
		<AcerolaToggleCard
			data={{
				title: m['pages.network.relaySettings.useAcerolaRelay'](),
				subtitle: m['pages.network.relaySettings.useAcerolaRelayDesc']({
					url: safeData.acerolaRelayUrl
				})
			}}
			state={{ active: safeData.useAcerolaRelay }}
			events={{ onClick: () => toggleAcerolaRelay(!safeData.useAcerolaRelay) }}
			ui={{
				disabled: safeData.useIrohPublicNetwork || restarting,
				iconClass: 'bg-chart-5 text-primary-foreground'
			}}
		>
			{#snippet icon()}
				<ServerIcon size={18} />
			{/snippet}
		</AcerolaToggleCard>

		<!-- Card 2: custom relays — the row only expands/collapses the list, it isn't what
		     enables the source (it turns on by itself once it has at least 1 URL, see
		     `activeSourceCount`). Stays clickable even with the Iroh public network on (only the
		     actions inside lock), so what is already configured can always be checked. -->
		<AcerolaToggleCard
			data={{
				title: m['pages.network.relaySettings.customRelays.title'](),
				subtitle: customRelaysSubtitle
			}}
			state={{ active: safeData.customRelayUrls.length > 0, expanded: customExpanded }}
			events={{ onClick: () => (customExpanded = !customExpanded) }}
			ui={{
				class: safeData.useIrohPublicNetwork ? 'opacity-50' : undefined,
				iconClass: 'bg-chart-2 text-primary-foreground'
			}}
		>
			{#snippet icon()}
				<NetworkIcon size={18} />
			{/snippet}

			{#snippet children()}
				<div class="space-y-2" use:autoAnimateList>
					{#each safeData.customRelayUrls as url (url)}
						<div
							class="flex items-center justify-between gap-3 rounded-inner border border-border bg-background/50 p-3"
						>
							<span class="min-w-0 flex-1 truncate text-sm text-foreground">{url}</span>
							<AcerolaButtonIcon
								events={{ onClick: () => removeCustomUrl(url) }}
								ui={{
									variant: 'ghost',
									tone: 'destructive',
									class: 'size-8',
									disabled: safeData.useIrohPublicNetwork || restarting,
									title: m['pages.network.relaySettings.customRelays.remove'](),
									'aria-label': m['pages.network.relaySettings.customRelays.remove']()
								}}
							>
								<Trash2Icon size={14} />
							</AcerolaButtonIcon>
						</div>
					{/each}
				</div>

				<div class="flex items-center gap-2">
					<AcerolaInput
						state={{ value: customUrlDraft }}
						events={{
							onValueChange: (value) => {
								customUrlDraft = value;
								customUrlError = false;
							}
						}}
						ui={{
							placeholder: m['pages.network.relaySettings.customRelays.addPlaceholder'](),
							class: 'flex-1',
							disabled: safeData.useIrohPublicNetwork || restarting
						}}
					/>
					<AcerolaButton
						events={{ onClick: submitCustomUrl }}
						ui={{
							size: 'sm',
							disabled: !customUrlDraft.trim() || safeData.useIrohPublicNetwork || restarting
						}}
					>
						<PlusIcon size={14} />
						{m['pages.network.relaySettings.customRelays.addButton']()}
					</AcerolaButton>
				</div>
				{#if customUrlError}
					<p class="text-xs text-destructive">
						{m['pages.network.relaySettings.invalidUrl']()}
					</p>
				{/if}
			{/snippet}
		</AcerolaToggleCard>

		<!-- Card 3: Iroh public network — the whole row turns it on/off (exclusive with the other
		     two, locked without a configured ticket); managing the ticket is a separate action
		     inside the card, so "enable the source" isn't confused with "change the credential". -->
		<div
			class={cn(
				'overflow-hidden rounded-inner border-2 transition-all duration-200',
				safeData.useIrohPublicNetwork
					? ' bg-primary/5'
					: 'border-border/60 bg-card'
			)}
		>
			<button
				type="button"
				onclick={() => toggleIrohPublicNetwork(!safeData.useIrohPublicNetwork)}
				disabled={!safeData.hasIrohServicesTicket || restarting}
				class="flex w-full items-center gap-3 p-4 text-left enabled:cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
			>
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-inner bg-chart-4 text-primary-foreground"
				>
					<GlobeIcon size={18} />
				</div>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-semibold text-foreground">
						{m['pages.network.relaySettings.useIrohPublicNetwork']()}
					</p>
					<p class="truncate text-xs text-muted-foreground">{irohSubtitle}</p>
				</div>
				{#if safeData.useIrohPublicNetwork}
					<div
						use:checkBadgePop
						class="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground"
					>
						<CheckIcon size={12} strokeWidth={3} />
					</div>
				{/if}
			</button>

			<div class="border-t border-border/60 p-3">
				<button
					type="button"
					onclick={() => (ticketExpanded = !ticketExpanded)}
					class="flex w-full cursor-pointer items-center gap-2 text-xs font-semibold text-muted-foreground hover:text-foreground"
				>
					<KeyRoundIcon size={12} />
					{safeData.hasIrohServicesTicket
						? m['pages.network.relaySettings.irohServicesTicket.configured']()
						: m['pages.network.relaySettings.irohServicesTicket.notConfigured']()}
				</button>

				{#if ticketExpanded}
					<div transition:slide={{ duration: 200 }} class="mt-3 space-y-2">
						<!-- Field on its own full-width row — sharing the row with the button
						     squeezed the field so much that the placeholder (much longer than any
						     relay URL) wrapped onto several lines. -->
						<AcerolaInput
							state={{ value: ticketDraft }}
							events={{
								onValueChange: (value) => {
									ticketDraft = value;
									ticketError = false;
								}
							}}
							ui={{
								type: 'password',
								placeholder: m['pages.network.relaySettings.irohServicesTicket.placeholder'](),
								class: 'w-full',
								disabled: ticketSaving
							}}
						/>

						{#if ticketError}
							<p class="text-xs text-destructive">
								{m['pages.network.relaySettings.irohServicesTicket.invalid']()}
							</p>
						{/if}

						<div class="flex items-center justify-end gap-2">
							{#if safeData.hasIrohServicesTicket}
								<AcerolaButtonIcon
									events={{ onClick: removeTicket }}
									ui={{
										variant: 'ghost',
										tone: 'destructive',
										class: 'size-8',
										disabled: ticketSaving,
										title: m['pages.network.relaySettings.irohServicesTicket.removeButton'](),
										'aria-label':
											m['pages.network.relaySettings.irohServicesTicket.removeButton']()
									}}
								>
									<Trash2Icon size={14} />
								</AcerolaButtonIcon>
							{/if}
							<AcerolaButton
								events={{ onClick: submitTicket }}
								ui={{ size: 'sm', disabled: !ticketDraft.trim() || ticketSaving }}
							>
								{safeData.hasIrohServicesTicket
									? m['pages.network.relaySettings.irohServicesTicket.replaceButton']()
									: m['pages.network.relaySettings.irohServicesTicket.saveButton']()}
							</AcerolaButton>
						</div>

						<p class="text-xs text-muted-foreground italic">
							{m['pages.network.relaySettings.irohServicesTicket.help']()}
						</p>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<div
		class="flex items-center justify-between gap-4 rounded-inner border border-border bg-background/50 p-3"
	>
		<div class="min-w-0">
			<p class="text-sm font-semibold text-foreground">
				{m['pages.network.relaySettings.restart.button']()}
			</p>
			{#if restartError}
				<p class="mt-1 text-xs text-destructive">
					{m['pages.network.relaySettings.restart.error']()}
				</p>
			{/if}
		</div>
		<AcerolaButton
			events={{ onClick: restart }}
			ui={{
				size: 'sm',
				variant: 'outline',
				disabled: restarting,
				title: m['pages.network.relaySettings.restart.description']()
			}}
		>
			<RefreshCwIcon size={14} class={cn(restarting && 'animate-spin')} />
			{restarting
				? m['pages.network.relaySettings.restart.restarting']()
				: m['pages.network.relaySettings.restart.button']()}
		</AcerolaButton>
	</div>
</AcerolaAccordionCard>

<AcerolaAlertDialog
	state={{ open: pendingSwitch !== null }}
	data={{
		title: m['pages.network.relaySettings.switchConfirm.title'](),
		description: m['pages.network.relaySettings.switchConfirm.desc'](),
		cancelText: m['pages.network.relaySettings.switchConfirm.cancel'](),
		actionText: m['pages.network.relaySettings.switchConfirm.action']()
	}}
	events={{
		onAction: () => {
			const action = pendingSwitch;
			pendingSwitch = null;
			if (action) runRestartingAction(action);
		},
		onCancel: () => (pendingSwitch = null)
	}}
/>
