<script module lang="ts">
	export type NetworkConnectCardProps = {
		data: {
			connecting: boolean;
		};
		events: {
			onConnect: (code: string) => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import PlugIcon from '@lucide/svelte/icons/plug';
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { InvalidConnectionCodeError } from '#lib/utils/connection-code.util.ts';
	import { toastAsync } from '#lib/utils/toast-async.util.ts';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';

	let { data, events }: NetworkConnectCardProps = $props();

	let pasteValue = $state('');
	let connectError = $state<string | undefined>(undefined);

	async function handleConnect() {
		connectError = undefined;
		try {
			await toastAsync(() => events.onConnect(pasteValue), {
				loading: m['pages.network.connect.loading'](),
				success: m['pages.network.connect.success'](),
				error: (err) =>
					err instanceof InvalidConnectionCodeError
						? m['pages.network.connect.invalidCode']()
						: m['pages.network.connect.error']()
			});
			pasteValue = '';
		} catch (err) {
			connectError =
				err instanceof InvalidConnectionCodeError
					? m['pages.network.connect.invalidCode']()
					: m['pages.network.connect.error']();
		}
	}
</script>

<AcerolaSettingSection
	data={{
		id: 'connect',
		title: m['pages.network.connect.title'](),
		lead: m['pages.network.connect.lead']()
	}}
>
	<form
		class="flex flex-col gap-2 sm:flex-row"
		onsubmit={(event) => {
			event.preventDefault();
			if (pasteValue.trim() && !data.connecting) handleConnect();
		}}
	>
		<AcerolaInput
			state={{ value: pasteValue }}
			events={{ onValueChange: (value) => (pasteValue = value) }}
			ui={{
				placeholder: m['pages.network.connect.placeholder'](),
				'aria-label': m['pages.network.connect.placeholder'](),
				class: 'flex-1'
			}}
		/>
		<!-- The screen's one primary action. -->
		<AcerolaButton ui={{ type: 'submit', disabled: !pasteValue.trim() || data.connecting }}>
			<PlugIcon size={16} />
			{m['pages.network.connect.button']()}
		</AcerolaButton>
	</form>

	{#if connectError}
		<p
			class="flex items-center gap-2 rounded-tile bg-destructive-soft p-3 text-sm font-semibold text-destructive"
		>
			<AlertCircleIcon size={16} class="shrink-0" />
			{connectError}
		</p>
	{/if}
</AcerolaSettingSection>
