<script module lang="ts">
	import type { TransferLogEntry } from '#lib/hooks/store/use-network-sync.svelte.ts';

	export type NetworkTransfersLogProps = {
		data: {
			entries: TransferLogEntry[];
			peerLabel: (peerId: string) => string;
		};
		events?: {
			onRefresh?: () => void;
			onClear?: () => void;
		};
	};
</script>

<script lang="ts">
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import CheckIcon from '@lucide/svelte/icons/check';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ArrowLeftRightIcon from '@lucide/svelte/icons/arrow-left-right';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaAccordionCard from '#lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaHeroButton from '#lib/components/acerola-hero-button/acerola-hero-button.svelte';

	let { data, events }: NetworkTransfersLogProps = $props();

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `data` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings
	// down the suite even with every assert passing.
	let entries = $derived(data?.entries ?? []);

	// Closed by default, expands inline — same component and same idea as the Relay
	// Settings card right above on this screen, instead of a separate dialog.
	let expanded = $state(false);

	type EntryMessageByStatus = Partial<
		Record<TransferLogEntry['status'], (entry: TransferLogEntry) => string>
	>;

	// Grouped by kind and then by status (instead of a switch over the concatenated
	// "kind:status" string) to make visible which combinations actually exist —
	// not every kind has "progress", for example.
	const messageByKind = $derived<Record<TransferLogEntry['kind'], EntryMessageByStatus>>({
		history: {
			started: (entry) =>
				m['pages.network.transfers.historyStarted']({ peer: data.peerLabel(entry.message) }),
			complete: (entry) =>
				m['pages.network.transfers.historyComplete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.historyError']({ msg: entry.message })
		},
		historyEntry: {
			started: (entry) =>
				m['pages.network.transfers.historyEntryStarted']({ peer: data.peerLabel(entry.message) }),
			complete: (entry) =>
				m['pages.network.transfers.historyEntryComplete']({
					peer: data.peerLabel(entry.message)
				}),
			error: (entry) => m['pages.network.transfers.historyEntryError']({ msg: entry.message })
		},
		files: {
			started: (entry) =>
				m['pages.network.transfers.filesStarted']({ peer: data.peerLabel(entry.message) }),
			progress: (entry) => m['pages.network.transfers.filesProgress']({ item: entry.message }),
			complete: (entry) =>
				m['pages.network.transfers.filesComplete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.filesError']({ msg: entry.message })
		},
		comic: {
			started: (entry) =>
				m['pages.network.transfers.comicStarted']({ peer: data.peerLabel(entry.message) }),
			progress: (entry) => m['pages.network.transfers.comicProgress']({ item: entry.message }),
			complete: (entry) =>
				m['pages.network.transfers.comicComplete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.comicError']({ msg: entry.message })
		}
	});

	function describeEntry(entry: TransferLogEntry): string {
		const base = messageByKind[entry.kind][entry.status]?.(entry) ?? entry.message;
		// No dedicated toast/notification for this (see `use-network-sync.svelte.ts`) — the
		// conflict only needs to show here, on the session line that was going to be shown
		// anyway, one total per session instead of one notification per chapter.
		if (entry.status === 'complete' && entry.conflicts) {
			return `${base} ${m['pages.network.transfers.conflictsSuffix']({ count: entry.conflicts })}`;
		}
		return base;
	}

	// Summary shown in the closed accordion header — the most recent entry
	// (`entries[0]`, see `use-network-sync.svelte.ts`) or the empty text.
	let summary = $derived(
		entries.length > 0 ? describeEntry(entries[0]) : m['pages.network.transfers.empty']()
	);

	// SOLID icon circle background tinted by status (the bg outlines the icon, which sits in
	// a neutral foreground color on top) — not AcerolaHeroButton's generic default
	// `bg-muted`.
	function iconBackgroundClass(entry: TransferLogEntry): string {
		if (entry.status === 'error') return 'bg-destructive text-destructive-foreground';
		if (entry.status === 'complete') return 'bg-chart-3 text-primary-foreground';
		return 'bg-muted text-muted-foreground';
	}
</script>

<AcerolaAccordionCard
	data={{ title: m['pages.network.transfers.title'](), description: summary }}
	state={{ expanded }}
	events={{ onToggle: () => (expanded = !expanded) }}
	ui={{ iconClass: 'bg-accent-hero text-accent-hero-foreground' }}
>
	{#snippet icon()}
		<ArrowLeftRightIcon size={20} />
	{/snippet}

	<div class="flex items-center justify-end gap-1">
		<AcerolaButton
			events={{ onClick: () => events?.onRefresh?.() }}
			ui={{
				variant: 'ghost',
				size: 'sm',
				class: 'h-8 gap-1.5 bg-accent-hero/15 px-2.5 text-accent-hero hover:bg-accent-hero/25'
			}}
		>
			<RefreshCwIcon size={14} />
			{m['pages.network.transfers.refresh']()}
		</AcerolaButton>

		{#if entries.length > 0}
			<AcerolaAlertDialog
				data={{
					title: m['pages.network.transfers.clear.title'](),
					description: m['pages.network.transfers.clear.desc'](),
					cancelText: m['pages.network.transfers.clear.cancel'](),
					actionText: m['pages.network.transfers.clear.confirm']()
				}}
				ui={{ variant: 'destructive' }}
				events={{ onAction: () => events?.onClear?.() }}
			>
				<AcerolaButton
					ui={{
						variant: 'ghost',
						size: 'sm',
						class: 'h-8 gap-1.5 bg-destructive/15 px-2.5 text-destructive hover:bg-destructive/25'
					}}
				>
					<Trash2Icon size={14} />
					{m['pages.network.transfers.clear.button']()}
				</AcerolaButton>
			</AcerolaAlertDialog>
		{/if}
	</div>

	{#if entries.length === 0}
		<p class="p-4 text-center text-sm text-muted-foreground">
			{m['pages.network.transfers.empty']()}
		</p>
	{:else}
		<div class="max-h-[28rem] space-y-2 overflow-y-auto pr-1">
			{#each entries as entry (entry.id)}
				<AcerolaHeroButton
					data={{
						title: describeEntry(entry),
						description: new Date(entry.timestamp).toLocaleTimeString()
					}}
					ui={{ iconClass: iconBackgroundClass(entry) }}
				>
					{#snippet icon()}
						{#if entry.status === 'error'}
							<AlertCircleIcon size={20} />
						{:else if entry.status === 'complete'}
							<CheckIcon size={20} />
						{:else if entry.status === 'started'}
							<RefreshCwIcon size={20} class="animate-spin" />
						{:else}
							<ArrowRightIcon size={20} />
						{/if}
					{/snippet}
				</AcerolaHeroButton>
			{/each}
		</div>
	{/if}
</AcerolaAccordionCard>
