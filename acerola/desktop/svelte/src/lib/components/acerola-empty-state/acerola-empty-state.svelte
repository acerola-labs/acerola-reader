<script module lang="ts">
	import type { Component, Snippet } from 'svelte';

	export type AcerolaEmptyStateProps = {
		data: {
			title: string;
			hint?: string | null;
			/** A round icon instead of the mascot (small empty states inside a card). */
			icon?: Component<{ size?: number }>;
		};
		ui?: {
			class?: string;
		};
		action?: Snippet;
	};
</script>

<script lang="ts">
	import { cn } from '$lib/utils/cn.utils';
	import { revealIn } from '$lib/utils/motion.utils';

	let { data, ui, action }: AcerolaEmptyStateProps = $props();
</script>

<!-- Acerola standard empty state: mascot (or round icon) + font-black title + one phrase + one action.
     The mascot is a local asset: the app is open source and does not depend on the brand CDN. -->
<div use:revealIn class={cn('flex flex-col items-center gap-2 py-10 text-center', ui?.class)}>
	{#if data.icon}
		<span
			class="mb-1 flex size-14 items-center justify-center rounded-full bg-surface text-muted-foreground"
		>
			<data.icon size={24} />
		</span>
	{:else}
		<img src="/brand/acerola-mascot.svg" alt="" class="mb-1 size-24" />
	{/if}
	<p class="text-base font-black tracking-tight">{data.title}</p>
	{#if data.hint}
		<p class="max-w-72 text-sm text-muted-foreground">{data.hint}</p>
	{/if}
	{#if action}
		<div class="mt-2">{@render action()}</div>
	{/if}
</div>
