<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaSettingSectionProps = {
		data: {
			/** Anchor and `aria-labelledby` target, in English (`files`, `paired-devices`). */
			id: string;
			title: string;
			lead?: string | null;
		};
		ui?: {
			class?: string;
		};
		/** Actions on the right of the title (refresh, clear…). */
		actions?: Snippet;
		children: Snippet;
	};
</script>

<script lang="ts">
	import { cn } from '#lib/utils/cn.util.ts';

	let { data, ui, actions, children }: AcerolaSettingSectionProps = $props();
</script>

<!-- A group of settings (brand book card): rounded-card on bg-card with shadow-lift, a
     text-lg font-black title, one phrase, and the rows (AcerolaSettingRow) below. -->
<section
	aria-labelledby="section-{data.id}"
	class={cn('grid min-w-0 gap-4 rounded-card bg-card p-5 shadow-lift', ui?.class)}
>
	<header class="flex items-start justify-between gap-3">
		<div class="grid min-w-0 gap-0.5">
			<h2 id="section-{data.id}" class="text-lg font-black tracking-tight text-foreground">
				{data.title}
			</h2>
			{#if data.lead}
				<p class="text-sm text-muted-foreground">{data.lead}</p>
			{/if}
		</div>
		{#if actions}
			<div class="flex shrink-0 items-center gap-2">{@render actions()}</div>
		{/if}
	</header>
	<div class="grid min-w-0 gap-2">{@render children()}</div>
</section>
