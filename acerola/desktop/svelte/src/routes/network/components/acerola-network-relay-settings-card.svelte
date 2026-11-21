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
	import ServerIcon from '@lucide/svelte/icons/server';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils/cn.util.ts';
	import { autoAnimateList } from '#lib/utils/auto-animate.util.ts';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import AcerolaSettingRow from '#lib/components/acerola-setting-row/acerola-setting-row.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';
	import AcerolaSwitch from '#lib/components/acerola-switch/acerola-switch.svelte';

	let { data, events }: NetworkRelaySettingsCardProps = $props();

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

<AcerolaSettingSection
	data={{ id: 'connection', title: m['pages.network.relaySettings.title'](), lead: summary }}
>
	<!-- Acerola relay: a plain switch. -->
	<AcerolaSettingRow
		data={{
			title: m['pages.network.relaySettings.useAcerolaRelay'](),
			description: m['pages.network.relaySettings.useAcerolaRelayDesc']({
				url: safeData.acerolaRelayUrl
			})
		}}
	>
		{#snippet icon()}<ServerIcon size={18} />{/snippet}
		{#snippet control()}
			<AcerolaSwitch
				state={{ checked: safeData.useAcerolaRelay }}
				events={{ onCheckedChange: (checked) => toggleAcerolaRelay(checked) }}
				ui={{
					disabled: safeData.useIrohPublicNetwork || restarting,
					'aria-label': m['pages.network.relaySettings.useAcerolaRelay']()
				}}
			/>
		{/snippet}
	</AcerolaSettingRow>

	<!-- Custom relays: the source turns on by itself once it has a URL; the button only opens
	     the list, so "manage" isn't confused with "enable". -->
	<AcerolaSettingRow
		data={{
			title: m['pages.network.relaySettings.customRelays.title'](),
			description: customRelaysSubtitle
		}}
		ui={{ tone: safeData.customRelayUrls.length > 0 ? 'primary' : 'neutral' }}
	>
		{#snippet icon()}<NetworkIcon size={18} />{/snippet}
		{#snippet control()}
			<AcerolaButton
				events={{ onClick: () => (customExpanded = !customExpanded) }}
				ui={{ variant: 'secondary', class: 'bg-card', 'aria-expanded': customExpanded }}
			>
				{m['pages.network.relaySettings.customRelays.manage']()}
				<ChevronDownIcon
					size={16}
					class={cn('transition-transform duration-200', customExpanded && 'rotate-180')}
				/>
			</AcerolaButton>
		{/snippet}

		{#if customExpanded}
			<div transition:slide={{ duration: 200 }} class="grid gap-2">
				<div class="grid gap-2" use:autoAnimateList>
					{#each safeData.customRelayUrls as url (url)}
						<div class="flex items-center gap-3 rounded-full bg-card py-1 pr-1 pl-4">
							<span class="min-w-0 flex-1 truncate text-sm text-foreground"
								>{url}</span
							>
							<AcerolaButtonIcon
								events={{ onClick: () => removeCustomUrl(url) }}
								ui={{
									tone: 'destructive',
									class: 'size-9',
									disabled: safeData.useIrohPublicNetwork || restarting,
									title: m['pages.network.relaySettings.customRelays.remove'](),
									'aria-label':
										m['pages.network.relaySettings.customRelays.remove']()
								}}
							>
								<Trash2Icon size={14} />
							</AcerolaButtonIcon>
						</div>
					{/each}
				</div>

				<form
					class="flex items-center gap-2"
					onsubmit={(event) => {
						event.preventDefault();
						submitCustomUrl();
					}}
				>
					<AcerolaInput
						state={{ value: customUrlDraft }}
						events={{
							onValueChange: (value) => {
								customUrlDraft = value;
								customUrlError = false;
							}
						}}
						ui={{
							placeholder:
								m['pages.network.relaySettings.customRelays.addPlaceholder'](),
							'aria-label':
								m['pages.network.relaySettings.customRelays.addPlaceholder'](),
							class: 'flex-1 bg-card',
							disabled: safeData.useIrohPublicNetwork || restarting
						}}
					/>
					<AcerolaButton
						ui={{
							variant: 'contrast',
							type: 'submit',
							disabled:
								!customUrlDraft.trim() ||
								safeData.useIrohPublicNetwork ||
								restarting
						}}
					>
						<PlusIcon size={16} />
						{m['pages.network.relaySettings.customRelays.addButton']()}
					</AcerolaButton>
				</form>
				{#if customUrlError}
					<p class="px-1 text-xs font-semibold text-destructive">
						{m['pages.network.relaySettings.invalidUrl']()}
					</p>
				{/if}
			</div>
		{/if}
	</AcerolaSettingRow>

	<!-- Iroh public network: the switch turns it on (exclusive with the other sources, locked
	     without a ticket); the ticket is managed apart so "enable" isn't "change credential". -->
	<AcerolaSettingRow
		data={{
			title: m['pages.network.relaySettings.useIrohPublicNetwork'](),
			description: irohSubtitle
		}}
	>
		{#snippet icon()}<GlobeIcon size={18} />{/snippet}
		{#snippet control()}
			<AcerolaButton
				events={{ onClick: () => (ticketExpanded = !ticketExpanded) }}
				ui={{ variant: 'secondary', class: 'bg-card', 'aria-expanded': ticketExpanded }}
			>
				<KeyRoundIcon size={16} />
				{m['pages.network.relaySettings.irohServicesTicket.manage']()}
			</AcerolaButton>
			<AcerolaSwitch
				state={{ checked: safeData.useIrohPublicNetwork }}
				events={{ onCheckedChange: (checked) => toggleIrohPublicNetwork(checked) }}
				ui={{
					disabled: !safeData.hasIrohServicesTicket || restarting,
					'aria-label': m['pages.network.relaySettings.useIrohPublicNetwork']()
				}}
			/>
		{/snippet}

		{#if ticketExpanded}
			<form
				transition:slide={{ duration: 200 }}
				class="grid gap-2"
				onsubmit={(event) => {
					event.preventDefault();
					submitTicket();
				}}
			>
				<p class="px-1 text-xs font-semibold text-muted-foreground">
					{safeData.hasIrohServicesTicket
						? m['pages.network.relaySettings.irohServicesTicket.configured']()
						: m['pages.network.relaySettings.irohServicesTicket.notConfigured']()}
				</p>
				<div class="flex items-center gap-2">
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
							placeholder:
								m['pages.network.relaySettings.irohServicesTicket.placeholder'](),
							'aria-label':
								m['pages.network.relaySettings.irohServicesTicket.placeholder'](),
							class: 'flex-1 bg-card',
							disabled: ticketSaving
						}}
					/>
					{#if safeData.hasIrohServicesTicket}
						<AcerolaButtonIcon
							events={{ onClick: removeTicket }}
							ui={{
								tone: 'destructive',
								disabled: ticketSaving,
								title: m[
									'pages.network.relaySettings.irohServicesTicket.removeButton'
								](),
								'aria-label':
									m[
										'pages.network.relaySettings.irohServicesTicket.removeButton'
									]()
							}}
						>
							<Trash2Icon size={16} />
						</AcerolaButtonIcon>
					{/if}
					<AcerolaButton
						ui={{
							variant: 'contrast',
							type: 'submit',
							disabled: !ticketDraft.trim() || ticketSaving
						}}
					>
						{safeData.hasIrohServicesTicket
							? m['pages.network.relaySettings.irohServicesTicket.replaceButton']()
							: m['pages.network.relaySettings.irohServicesTicket.saveButton']()}
					</AcerolaButton>
				</div>
				{#if ticketError}
					<p class="px-1 text-xs font-semibold text-destructive">
						{m['pages.network.relaySettings.irohServicesTicket.invalid']()}
					</p>
				{/if}
				<p class="px-1 text-xs text-muted-foreground">
					{m['pages.network.relaySettings.irohServicesTicket.help']()}
				</p>
			</form>
		{/if}
	</AcerolaSettingRow>

	<!-- Restart the P2P node by hand. -->
	<AcerolaSettingRow
		data={{
			title: m['pages.network.relaySettings.restart.button'](),
			description: restartError
				? m['pages.network.relaySettings.restart.error']()
				: m['pages.network.relaySettings.restart.description']()
		}}
		ui={{ tone: restartError ? 'destructive' : 'neutral' }}
	>
		{#snippet icon()}<RefreshCwIcon
				size={18}
				class={cn(restarting && 'animate-spin')}
			/>{/snippet}
		{#snippet control()}
			<AcerolaButton
				events={{ onClick: restart }}
				ui={{ variant: 'secondary', class: 'bg-card', disabled: restarting }}
			>
				{restarting
					? m['pages.network.relaySettings.restart.restarting']()
					: m['pages.network.relaySettings.restart.button']()}
			</AcerolaButton>
		{/snippet}
	</AcerolaSettingRow>
</AcerolaSettingSection>

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
