<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import { cn } from '#lib/utils/cn.util.ts';
	import type { Component, Snippet } from 'svelte';

	let {
		title,
		href,
		icon,
		badge,
		class: className,
		children
	}: {
		title: string;
		href?: string;
		icon?: Component;
		badge?: string;
		class?: string;
		children?: Snippet;
	} = $props();

	const isExternal = $derived(href?.startsWith('http') ?? false);
</script>

{#snippet body()}
	<div class="flex items-center gap-3">
		{#if icon}
			{@const Icon = icon}
			<span
				class="flex size-11 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary"
			>
				<Icon size={20} />
			</span>
		{/if}
		<div class="flex min-w-0 flex-1 items-center justify-between gap-2">
			<span class="leading-tight font-black tracking-tight text-foreground">{title}</span>
			{#if badge}
				<span
					class="shrink-0 rounded-full bg-primary-soft px-2.5 py-0.5 text-xs font-black whitespace-nowrap text-primary"
				>
					{badge}
				</span>
			{/if}
		</div>
		{#if href}
			<ArrowRightIcon
				size={18}
				class="shrink-0 text-muted-foreground transition-transform duration-200 ease-brand group-hover:translate-x-1 group-hover:text-foreground"
			/>
		{/if}
	</div>
	{#if children}
		<div class="text-sm leading-relaxed text-muted-foreground [&>p]:my-0">
			{@render children()}
		</div>
	{/if}
{/snippet}

<!-- Acerola card (brand book `.ac-card`): solid block, rounded-card, no border, shadow-lift.
     As a link it rises to shadow-float on hover. -->
{#if href}
	<a
		{href}
		target={isExternal ? '_blank' : undefined}
		rel={isExternal ? 'noopener noreferrer' : undefined}
		data-slot="card"
		class={cn(
			'group grid min-w-0 content-start gap-3 rounded-card bg-card p-5 no-underline shadow-lift transition-all duration-200 ease-brand hover:-translate-y-0.5 hover:shadow-float',
			className
		)}
	>
		{@render body()}
	</a>
{:else}
	<div
		data-slot="card"
		class={cn(
			'grid min-w-0 content-start gap-3 rounded-card bg-card p-5 shadow-lift',
			className
		)}
	>
		{@render body()}
	</div>
{/if}
