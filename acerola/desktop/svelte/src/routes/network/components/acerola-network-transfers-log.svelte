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
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import AcerolaSettingRow, {
		type AcerolaSettingRowTone
	} from '#lib/components/acerola-setting-row/acerola-setting-row.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';

	let { data, events }: NetworkTransfersLogProps = $props();

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `data` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings
	// down the suite even with every assert passing.
	let entries = $derived(data?.entries ?? []);

	type EntryMessageByStatus = Partial<
		Record<TransferLogEntry['status'], (entry: TransferLogEntry) => string>
	>;

	// Grouped by kind and then by status (instead of a switch over the concatenated
	// "kind:status" string) to make visible which combinations actually exist —
	// not every kind has "progress", for example.
	const messageByKind = $derived<Record<TransferLogEntry['kind'], EntryMessageByStatus>>({
		history: {
			started: (entry) =>
				m['pages.network.transfers.historyStarted']({
					peer: data.peerLabel(entry.message)
				}),
			complete: (entry) =>
				m['pages.network.transfers.historyComplete']({
					peer: data.peerLabel(entry.message)
				}),
			error: (entry) => m['pages.network.transfers.historyError']({ msg: entry.message })
		},
		historyEntry: {
			started: (entry) =>
				m['pages.network.transfers.historyEntryStarted']({
					peer: data.peerLabel(entry.message)
				}),
			complete: (entry) =>
				m['pages.network.transfers.historyEntryComplete']({
					peer: data.peerLabel(entry.message)
				}),
			error: (entry) => m['pages.network.transfers.historyEntryError']({ msg: entry.message })
		},
		files: {
			started: (entry) =>
				m['pages.network.transfers.filesStarted']({ peer: data.peerLabel(entry.message) }),
			progress: (entry) =>
				m['pages.network.transfers.filesProgress']({ item: entry.message }),
			complete: (entry) =>
				m['pages.network.transfers.filesComplete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.filesError']({ msg: entry.message })
		},
		comic: {
			started: (entry) =>
				m['pages.network.transfers.comicStarted']({ peer: data.peerLabel(entry.message) }),
			progress: (entry) =>
				m['pages.network.transfers.comicProgress']({ item: entry.message }),
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

	// Status is state, so it carries color (soft background + the tone); in progress stays neutral.
	function entryTone(entry: TransferLogEntry): AcerolaSettingRowTone {
		if (entry.status === 'error') return 'destructive';
		if (entry.status === 'complete') return 'success';
		return 'neutral';
	}
</script>

<AcerolaSettingSection
	data={{
		id: 'transfers',
		title: m['pages.network.transfers.title'](),
		lead: m['pages.network.transfers.lead']()
	}}
>
	{#snippet actions()}
		<AcerolaButtonIcon
			events={{ onClick: () => events?.onRefresh?.() }}
			ui={{
				variant: 'secondary',
				title: m['pages.network.transfers.refresh'](),
				'aria-label': m['pages.network.transfers.refresh']()
			}}
		>
			<RefreshCwIcon size={16} />
		</AcerolaButtonIcon>

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
				<span
					class="flex h-11 items-center gap-2 rounded-full bg-destructive-soft px-4 text-sm font-black text-destructive transition-[filter] hover:brightness-110"
				>
					<Trash2Icon size={16} />
					{m['pages.network.transfers.clear.button']()}
				</span>
			</AcerolaAlertDialog>
		{/if}
	{/snippet}

	{#if entries.length === 0}
		<AcerolaEmptyState
			data={{ title: m['pages.network.transfers.empty'](), icon: ArrowLeftRightIcon }}
			ui={{ class: 'py-6' }}
		/>
	{:else}
		<div class="grid max-h-[28rem] gap-2 overflow-y-auto">
			{#each entries as entry (entry.id)}
				<AcerolaSettingRow
					data={{
						title: describeEntry(entry),
						description: new Date(entry.timestamp).toLocaleTimeString()
					}}
					ui={{ tone: entryTone(entry) }}
				>
					{#snippet icon()}
						{#if entry.status === 'error'}
							<AlertCircleIcon size={18} />
						{:else if entry.status === 'complete'}
							<CheckIcon size={18} />
						{:else if entry.status === 'started'}
							<RefreshCwIcon size={18} class="animate-spin" />
						{:else}
							<ArrowRightIcon size={18} />
						{/if}
					{/snippet}
				</AcerolaSettingRow>
			{/each}
		</div>
	{/if}
</AcerolaSettingSection>
