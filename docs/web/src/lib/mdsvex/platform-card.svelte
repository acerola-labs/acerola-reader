<script lang="ts">
	import type { Component, Snippet } from 'svelte';

	let {
		title,
		description,
		icon,
		children
	}: {
		title: string;
		description?: string;
		icon?: Component;
		children?: Snippet;
	} = $props();
</script>

<!-- Acerola card (brand book `.ac-card`): solid block, rounded-card, no border, shadow-lift;
     round icon in primary-soft, font-black title, one muted phrase. -->
<article data-slot="card" class="grid content-start gap-4 rounded-card bg-card p-5 shadow-lift">
	{#if icon}
		{@const Icon = icon}
		<span
			class="flex size-11 items-center justify-center rounded-full bg-primary-soft text-primary"
		>
			<Icon size={20} />
		</span>
	{/if}

	<div class="grid gap-1">
		<h3 class="m-0 text-lg font-black tracking-tight text-foreground">{title}</h3>
		{#if description}
			<p class="m-0 text-sm text-pretty text-muted-foreground">{description}</p>
		{/if}
	</div>

	{#if children}
		<div class="text-sm text-muted-foreground [&>p]:my-0">
			{@render children()}
		</div>
	{/if}
</article>
