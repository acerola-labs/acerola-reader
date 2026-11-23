<script module lang="ts">
	export type AcerolaSettingsHeaderProps = {
		data: {
			title: string;
			/** Short line above the title (where the screen lives, e.g. "Configurações"). */
			eyebrow?: string;
			lead?: string;
		};
		events?: {
			onBack?: () => void;
		};
		ui?: {
			/** Accessible name of the back button. */
			backLabel?: string;
		};
	};
</script>

<script lang="ts">
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';

	let { data, events, ui }: AcerolaSettingsHeaderProps = $props();
</script>

<!-- Brand book screen header: eyebrow, 44px font-black title and one line of lead; a round back
     button when the screen sits under another one. -->
<header class="grid gap-4">
	{#if events?.onBack}
		<AcerolaButtonIcon
			events={{ onClick: events.onBack }}
			ui={{ variant: 'secondary', title: ui?.backLabel, 'aria-label': ui?.backLabel }}
		>
			<ArrowLeft size={18} />
		</AcerolaButtonIcon>
	{/if}
	<div class="grid gap-2">
		{#if data.eyebrow}
			<p class="text-xs font-semibold text-muted-foreground">{data.eyebrow}</p>
		{/if}
		<h1 class="text-[44px] leading-none font-black tracking-tighter text-foreground">
			{data.title}
		</h1>
		{#if data.lead}
			<p class="text-sm text-muted-foreground">{data.lead}</p>
		{/if}
	</div>
</header>
