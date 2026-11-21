<script module lang="ts">
	export type DisplayPeer = {
		peerId: string;
		deviceName: string | null;
		nickname: string | null;
		connected: boolean;
	};

	export type NetworkPeerListProps = {
		data: {
			peers: DisplayPeer[];
			addrFor: (peerId: string) => number[] | undefined;
			statusLabel: (peer: DisplayPeer) => string;
			isSyncing: (peerId: string, kind: 'history' | 'files') => boolean;
		};
		events: {
			onSyncHistory: (peerId: string) => void;
			onSyncFiles: (peerId: string) => void;
			onSyncAll: (peerId: string) => void;
			onBrowseLibrary: (peerId: string) => void;
			onRemove: (peer: DisplayPeer) => void;
			onRename: (peerId: string, nickname: string) => void;
		};
	};
</script>

<script lang="ts">
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MoreVerticalIcon from '@lucide/svelte/icons/more-vertical';
	import HistoryIcon from '@lucide/svelte/icons/history';
	import FolderSyncIcon from '@lucide/svelte/icons/folder-sync';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import UserMinusIcon from '@lucide/svelte/icons/user-minus';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import CheckIcon from '@lucide/svelte/icons/check';
	import XIcon from '@lucide/svelte/icons/x';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { shortId } from '#lib/utils/connection-code.util.ts';
	import { autoAnimateList } from '#lib/utils/auto-animate.util.ts';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import AcerolaSettingRow from '#lib/components/acerola-setting-row/acerola-setting-row.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';

	let { data, events }: NetworkPeerListProps = $props();

	let openPeerMenuId = $state<string | null>(null);
	// Local nickname per peer — inline editing replaces the peer card, same pattern as
	// `acerola-network-my-device-card.svelte` for the device's own name.
	let editingPeerId = $state<string | null>(null);
	let nicknameDraft = $state('');

	function startEditingNickname(peer: DisplayPeer) {
		nicknameDraft = peer.nickname ?? '';
		editingPeerId = peer.peerId;
	}

	function cancelEditingNickname() {
		editingPeerId = null;
	}

	function confirmEditingNickname() {
		if (!editingPeerId) return;
		events.onRename(editingPeerId, nicknameDraft.trim());
		editingPeerId = null;
	}

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `data` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings
	// down the suite even with every assert passing.
	let peers = $derived(data?.peers ?? []);
</script>

{#snippet menuItem(
	label: string,
	Icon: typeof HistoryIcon,
	onClick: () => void,
	opts: { disabled?: boolean; spinning?: boolean; danger?: boolean } = {}
)}
	<button
		type="button"
		class={[
			'flex h-11 w-full items-center gap-3 rounded-full px-4 text-left text-sm font-bold transition-colors duration-200 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60',
			opts.danger
				? 'text-destructive enabled:hover:bg-destructive-soft'
				: 'text-foreground enabled:hover:bg-surface'
		]}
		disabled={opts.disabled}
		onclick={onClick}
	>
		<Icon size={16} class={['shrink-0', opts.spinning && 'animate-spin']} />
		{label}
	</button>
{/snippet}

<AcerolaSettingSection
	data={{
		id: 'paired-devices',
		title: m['pages.network.peers.title'](),
		lead: m['pages.network.peers.lead']()
	}}
>
	<div class="grid gap-2" use:autoAnimateList>
		{#if peers.length === 0}
			<AcerolaEmptyState
				data={{
					title: m['pages.network.peers.emptyTitle'](),
					hint: m['pages.network.peers.empty'](),
					icon: RadioIcon
				}}
				ui={{ class: 'py-6' }}
			/>
		{:else}
			{#each peers as peer (peer.peerId)}
				{@const addrs = data.addrFor(peer.peerId)}
				{@const historySyncing = data.isSyncing(peer.peerId, 'history')}
				{@const filesSyncing = data.isSyncing(peer.peerId, 'files')}
				{@const anySyncing = historySyncing || filesSyncing}

				{#if editingPeerId === peer.peerId}
					<form
						class="grid gap-2 rounded-tile bg-surface p-4"
						onsubmit={(event) => {
							event.preventDefault();
							confirmEditingNickname();
						}}
					>
						<div class="flex items-center gap-2">
							<AcerolaInput
								state={{ value: nicknameDraft }}
								events={{ onValueChange: (value) => (nicknameDraft = value) }}
								ui={{
									placeholder: m['pages.network.peers.rename.placeholder'](),
									'aria-label': m['pages.network.peers.rename.placeholder'](),
									class: 'flex-1 bg-card'
								}}
							/>
							<AcerolaButton ui={{ variant: 'contrast', type: 'submit' }}>
								<CheckIcon size={16} />
								{m['pages.network.peers.rename.save']()}
							</AcerolaButton>
							<AcerolaButtonIcon
								events={{ onClick: cancelEditingNickname }}
								ui={{
									variant: 'secondary',
									class: 'bg-card',
									title: m['pages.network.peers.rename.cancel'](),
									'aria-label': m['pages.network.peers.rename.cancel']()
								}}
							>
								<XIcon size={16} />
							</AcerolaButtonIcon>
						</div>
						<p class="px-1 text-xs text-muted-foreground">
							{m['pages.network.peers.rename.hint']()}
						</p>
					</form>
				{:else}
					<AcerolaSettingRow
						data={{
							title: peer.nickname ?? peer.deviceName ?? shortId(peer.peerId),
							description: addrs
								? data.statusLabel(peer)
								: m['pages.network.peers.offlineHint']()
						}}
						ui={{ tone: peer.connected ? 'success' : 'neutral' }}
					>
						{#snippet icon()}
							<MonitorIcon size={18} />
							<span
								aria-hidden="true"
								class={[
									'absolute right-0 bottom-0 size-3 rounded-full ring-2 ring-surface',
									peer.connected ? 'bg-success' : 'bg-muted-foreground'
								]}
							></span>
						{/snippet}

						{#snippet control()}
							<AcerolaPopover
								state={{ open: openPeerMenuId === peer.peerId }}
								events={{
									onOpenChange: (open) =>
										(openPeerMenuId = open ? peer.peerId : null)
								}}
								ui={{ align: 'end', contentClass: 'w-60 p-2' }}
							>
								{#snippet trigger()}
									<span
										class="flex size-11 items-center justify-center rounded-full bg-card text-foreground transition-[filter] hover:brightness-110"
									>
										<MoreVerticalIcon size={16} />
										<span class="sr-only"
											>{m['pages.network.peers.moreActions']()}</span
										>
									</span>
								{/snippet}

								{#snippet content()}
									<div class="grid gap-0.5">
										{@render menuItem(
											m['pages.network.peers.syncHistory'](),
											historySyncing ? RefreshCwIcon : HistoryIcon,
											() => {
												openPeerMenuId = null;
												events.onSyncHistory(peer.peerId);
											},
											{
												disabled: !addrs || historySyncing,
												spinning: historySyncing
											}
										)}
										{@render menuItem(
											m['pages.network.peers.syncFiles'](),
											filesSyncing ? RefreshCwIcon : FolderSyncIcon,
											() => {
												openPeerMenuId = null;
												events.onSyncFiles(peer.peerId);
											},
											{
												disabled: !addrs || filesSyncing,
												spinning: filesSyncing
											}
										)}
										{@render menuItem(
											m['pages.network.peers.browseLibrary'](),
											BookOpenIcon,
											() => {
												openPeerMenuId = null;
												events.onBrowseLibrary(peer.peerId);
											},
											{ disabled: !addrs }
										)}
										{@render menuItem(
											m['pages.network.peers.rename.action'](),
											PencilIcon,
											() => {
												openPeerMenuId = null;
												startEditingNickname(peer);
											}
										)}
										{@render menuItem(
											m['pages.network.peers.remove'](),
											UserMinusIcon,
											() => {
												openPeerMenuId = null;
												events.onRemove(peer);
											},
											{ danger: true }
										)}
									</div>
								{/snippet}
							</AcerolaPopover>

							<AcerolaButton
								events={{ onClick: () => events.onSyncAll(peer.peerId) }}
								ui={{
									variant: 'secondary',
									class: 'bg-card',
									disabled: !addrs || anySyncing
								}}
							>
								<RefreshCwIcon size={16} class={anySyncing ? 'animate-spin' : ''} />
								{m['pages.network.peers.syncAll']()}
							</AcerolaButton>
						{/snippet}
					</AcerolaSettingRow>
				{/if}
			{/each}
		{/if}
	</div>
</AcerolaSettingSection>
