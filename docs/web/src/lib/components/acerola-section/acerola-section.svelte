<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		data: { id: string; eyebrow: string; title: string; lead?: string };
		children: Snippet;
	};

	let { data, children }: Props = $props();
</script>

<section
	id={data.id}
	class="mx-auto max-w-6xl px-4 py-14 sm:px-6"
	aria-labelledby="{data.id}-title"
>
	<p class="text-xs font-semibold text-muted-foreground">{data.eyebrow}</p>
	<h2
		id="{data.id}-title"
		class="mt-1 text-4xl leading-none font-black tracking-tighter sm:text-[44px]"
	>
		{data.title}
	</h2>
	{#if data.lead}
		<p class="mt-3 max-w-2xl text-pretty text-muted-foreground">{data.lead}</p>
	{/if}
	<div class="mt-8">{@render children()}</div>
</section>
