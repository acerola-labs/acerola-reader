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
     the platform's logo in a neutral icon block (`.ac-icon-tile`: a logo keeps its own colors),
     font-black title, one muted phrase. -->
<article data-slot="card" class="grid content-start gap-4 rounded-card bg-card p-5 shadow-lift">
	{#if icon}
		{@const Icon = icon}
		<span class="ac-icon-tile">
			<!-- class sizes both a .svg logo (?component) and a Lucide icon. -->
			<Icon class="size-6" aria-hidden="true" />
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
