<script module lang="ts">
	export type NetworkPairingCardProps = {
		data: {
			code: string | undefined;
		};
	};
</script>

<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { error } from '@tauri-apps/plugin-log';
	import QRCode from 'qrcode';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ShieldAlertIcon from '@lucide/svelte/icons/shield-alert';
	import AcerolaCopyButton from '#lib/components/acerola-button/acerola-copy-button.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils/cn.util.ts';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';

	let { data }: NetworkPairingCardProps = $props();

	let qrDataUrl = $state<string | undefined>(undefined);
	let qrError = $state(false);
	let showRawCode = $state(false);

	$effect(() => {
		// On story teardown (Storybook + vitest browser mode), this effect runs once more
		// with `data` already undefined before the component is actually destroyed —
		// without the optional chaining here this leaks as an unhandled error and brings
		// down the suite even with every assert passing.
		const code = data?.code;
		if (!code) {
			qrDataUrl = undefined;
			qrError = false;
			return;
		}
		QRCode.toDataURL(code, { margin: 4, width: 280, errorCorrectionLevel: 'L' })
			.then((url) => {
				qrDataUrl = url;
				qrError = false;
			})
			.catch((err) => {
				error(`failed to generate QR code: ${err}`);
				qrDataUrl = undefined;
				qrError = true;
			});
	});

	function handleCopied() {
		toast.success(m['pages.network.copied']());
	}
</script>

<AcerolaSettingSection
	data={{
		id: 'pairing',
		title: m['pages.network.pairing.title'](),
		lead: m['pages.network.pairing.desc']()
	}}
>
	<div class="flex flex-col items-center gap-4 rounded-tile bg-surface p-5">
		<!-- The QR stays black on white whatever the theme, so any camera can read it. -->
		<div class="flex size-64 shrink-0 items-center justify-center rounded-tile bg-on-scrim p-3">
			{#if qrDataUrl}
				<img
					src={qrDataUrl}
					alt={m['pages.network.pairing.desc']()}
					class="h-full w-full rounded-inner"
				/>
			{:else if qrError}
				<span class="px-4 text-center text-xs font-semibold text-destructive">
					{m['pages.network.pairing.qrError']()}
				</span>
			{:else}
				<div
					class="size-full animate-pulse rounded-inner bg-surface"
					role="status"
					aria-label={m['pages.network.pairing.loading']()}
				></div>
			{/if}
		</div>

		<div class="grid w-full grid-cols-2 gap-2">
			<AcerolaCopyButton
				value={data?.code}
				label={m['pages.network.pairing.copy']()}
				copiedLabel={m['pages.network.copied']()}
				disabled={!data?.code}
				onCopy={handleCopied}
				variant="contrast"
				class="w-full"
			/>
			<AcerolaButton
				events={{ onClick: () => (showRawCode = !showRawCode) }}
				ui={{
					variant: 'secondary',
					class: 'w-full bg-card',
					'aria-expanded': showRawCode,
					'aria-controls': 'pairing-raw-code'
				}}
			>
				{showRawCode
					? m['pages.network.pairing.hideCode']()
					: m['pages.network.pairing.showCode']()}
				<ChevronDownIcon
					size={16}
					class={cn('transition-transform duration-200', showRawCode && 'rotate-180')}
				/>
			</AcerolaButton>
		</div>

		{#if showRawCode}
			<!-- The raw code as a selectable block (no field chrome, no scrollbar): one click selects it all. -->
			<p
				id="pairing-raw-code"
				aria-label={m['pages.network.pairing.title']()}
				class="w-full rounded-inner bg-card p-4 text-xs leading-relaxed break-all text-muted-foreground select-all"
			>
				{data?.code ?? ''}
			</p>
		{/if}
	</div>

	<div class="flex items-start gap-3 rounded-tile bg-info-soft p-4 text-info">
		<ShieldAlertIcon size={16} class="mt-0.5 shrink-0" />
		<p class="text-xs leading-relaxed font-semibold">
			{m['pages.network.pairing.securityNote']()}
		</p>
	</div>
</AcerolaSettingSection>
