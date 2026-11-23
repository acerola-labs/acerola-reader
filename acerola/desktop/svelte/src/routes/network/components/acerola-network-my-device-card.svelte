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
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import XIcon from '@lucide/svelte/icons/x';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { shortId } from '#lib/utils/connection-code.util.ts';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';
	import AcerolaSettingRow from '#lib/components/acerola-setting-row/acerola-setting-row.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';

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
			toast.error(
				m['pages.network.myDevice.rename.error']({ msg: extractErrorMessage(err) })
			);
		} finally {
			savingName = false;
		}
	}
</script>

<AcerolaSettingSection
	data={{
		id: 'this-device',
		title: m['pages.network.myDevice.title'](),
		lead:
			safeData.mode === 'relay'
				? m['pages.network.myDevice.modeRelay']()
				: m['pages.network.myDevice.modeLocal']()
	}}
>
	{#if editingName}
		<form
			class="flex items-center gap-2 rounded-tile bg-surface p-4"
			onsubmit={(event) => {
				event.preventDefault();
				confirmEditingName();
			}}
		>
			<AcerolaInput
				state={{ value: nameDraft }}
				events={{ onValueChange: (value) => (nameDraft = value) }}
				ui={{
					placeholder: m['pages.network.myDevice.rename.placeholder'](),
					'aria-label': m['pages.network.myDevice.rename.placeholder'](),
					class: 'flex-1 bg-card',
					disabled: savingName
				}}
			/>
			<AcerolaButton
				ui={{
					variant: 'contrast',
					type: 'submit',
					disabled: !nameDraft.trim() || savingName
				}}
			>
				<CheckIcon size={16} />
				{m['pages.network.myDevice.rename.save']()}
			</AcerolaButton>
			<AcerolaButtonIcon
				events={{ onClick: cancelEditingName }}
				ui={{
					variant: 'secondary',
					disabled: savingName,
					title: m['pages.network.myDevice.rename.cancel'](),
					'aria-label': m['pages.network.myDevice.rename.cancel']()
				}}
			>
				<XIcon size={16} />
			</AcerolaButtonIcon>
		</form>
	{:else}
		<AcerolaSettingRow
			data={{
				title: safeData.deviceName ?? '…',
				description: safeData.localId ? shortId(safeData.localId) : '…'
			}}
		>
			{#snippet icon()}<MonitorIcon size={18} />{/snippet}
			{#snippet control()}
				{#if events?.onRenameDevice}
					<AcerolaButtonIcon
						events={{ onClick: startEditingName }}
						ui={{
							variant: 'secondary',
							class: 'bg-card',
							title: m['pages.network.myDevice.rename.action'](),
							'aria-label': m['pages.network.myDevice.rename.action']()
						}}
					>
						<PencilIcon size={16} />
					</AcerolaButtonIcon>
				{/if}
				<AcerolaButton
					events={{ onClick: copyLocalId }}
					ui={{ variant: 'secondary', class: 'bg-card', disabled: !safeData.localId }}
				>
					{#if idJustCopied}
						<CheckIcon size={16} class="text-success" />
					{:else}
						<CopyIcon size={16} />
					{/if}
					{m['pages.network.myDevice.copyId']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>
	{/if}
</AcerolaSettingSection>
