<script module lang="ts">
	export type NetworkMyDeviceCardProps = {
		data: {
			deviceName: string | null | undefined;
			localId: string | null | undefined;
			mode: 'relay' | 'local' | undefined;
		};
		events?: {
			onRenameDevice?: (name: string) => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import XIcon from '@lucide/svelte/icons/x';
	import AcerolaHeroButton from '#lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { shortId } from '#lib/utils/connection-code.util.ts';

	let { data, events }: NetworkMyDeviceCardProps = $props();

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `data` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings
	// down the suite even with every assert passing.
	let safeData = $derived(
		data ?? {
			deviceName: undefined,
			localId: undefined,
			mode: undefined
		}
	);

	// Copy button visual feedback — besides the toast, the icon itself turns into a check
	// for a moment, so anyone who misses the toast still notices the copy happened.
	let idJustCopied = $state(false);

	async function copyLocalId() {
		if (!safeData.localId) return;
		await navigator.clipboard.writeText(safeData.localId);
		toast.success(m['pages.network.copied']());
		idJustCopied = true;
		setTimeout(() => (idJustCopied = false), 2000);
	}

	// LocalSend-style custom nickname — inline editing instead of a separate dialog, since it
	// is a single field and the Network screen is already the natural place to change it.
	let editingName = $state(false);
	let nameDraft = $state('');
	let savingName = $state(false);

	function startEditingName() {
		nameDraft = safeData.deviceName ?? '';
		editingName = true;
	}

	function cancelEditingName() {
		editingName = false;
	}

	async function confirmEditingName() {
		const trimmed = nameDraft.trim();
		if (!trimmed || savingName || !events?.onRenameDevice) return;

		savingName = true;
		try {
			await events.onRenameDevice(trimmed);
			toast.success(m['pages.network.myDevice.rename.success']());
			editingName = false;
		} catch (err) {
			toast.error(m['pages.network.myDevice.rename.error']({ msg: String(err) }));
		} finally {
			savingName = false;
		}
	}
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<Share2Icon size={16} />
		{m['pages.network.myDevice.title']()}
	</div>

	{#if editingName}
		<div class="flex items-center gap-3 rounded-tile border border-border bg-card p-6">
			<AcerolaInput
				state={{ value: nameDraft }}
				events={{ onValueChange: (value) => (nameDraft = value) }}
				ui={{
					placeholder: m['pages.network.myDevice.rename.placeholder'](),
					class: 'flex-1',
					disabled: savingName
				}}
			/>
			<AcerolaButton
				events={{ onClick: confirmEditingName }}
				ui={{ size: 'sm', disabled: !nameDraft.trim() || savingName }}
			>
				<CheckIcon size={14} />
				{m['pages.network.myDevice.rename.save']()}
			</AcerolaButton>
			<AcerolaButtonIcon
				events={{ onClick: cancelEditingName }}
				ui={{
					variant: 'ghost',
					tone: 'muted',
					disabled: savingName,
					title: m['pages.network.myDevice.rename.cancel'](),
					'aria-label': m['pages.network.myDevice.rename.cancel']()
				}}
			>
				<XIcon size={16} />
			</AcerolaButtonIcon>
		</div>
	{:else}
		<AcerolaHeroButton
			data={{
				title: safeData.deviceName ?? '...',
				description: safeData.localId ? shortId(safeData.localId) : '...'
			}}
			ui={{ iconClass: 'bg-accent-hero text-accent-hero-foreground' }}
		>
			{#snippet icon()}
				<MonitorIcon size={22} />
			{/snippet}

			{#snippet action()}
				{#if events?.onRenameDevice}
					<AcerolaButtonIcon
						events={{ onClick: startEditingName }}
						ui={{
							variant: 'outline',
							tone: 'accent',
							title: m['pages.network.myDevice.rename.action'](),
							'aria-label': m['pages.network.myDevice.rename.action']()
						}}
					>
						<PencilIcon size={14} />
					</AcerolaButtonIcon>
				{/if}
				<AcerolaButton
					events={{ onClick: copyLocalId }}
					ui={{ variant: 'outline', size: 'sm', class: 'gap-2', disabled: !safeData.localId }}
				>
					{#if idJustCopied}
						<CheckIcon size={14} class="text-chart-3" />
					{:else}
						<CopyIcon size={14} />
					{/if}
					{m['pages.network.myDevice.copyId']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaHeroButton>
	{/if}

	<p class="px-2 text-xs text-muted-foreground">
		{safeData.mode === 'relay'
			? m['pages.network.myDevice.modeRelay']()
			: m['pages.network.myDevice.modeLocal']()}
	</p>
</section>
