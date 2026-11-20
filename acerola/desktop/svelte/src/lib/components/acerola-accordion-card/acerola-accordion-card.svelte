<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaAccordionCardProps = {
		data: {
			title: string;
			description?: string;
		};
		state: {
			expanded: boolean;
		};
		events: {
			onToggle: () => void;
		};
		ui?: {
			class?: string;
			/** `false` renders the section always open, with no button/chevron/click — to mix, on
			 *  the same screen, sections worth seeing right away with others dense enough to stay
			 *  hidden by default (same spirit as the Network screen, where only the relay card
			 *  collapses and the rest sits directly on screen). Default `true` (the usual
			 *  behavior, without breaking any existing usage). */
			collapsible?: boolean;
			/** Overrides the icon circle's default background/color (`bg-muted text-foreground`) —
			 *  same pattern as AcerolaHeroButton's `ui.iconClass`. */
			iconClass?: string;
		};
	};

	export type AcerolaAccordionCardSnippets = {
		icon?: Snippet;
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { slide } from 'svelte/transition';
	import { cn } from '#lib/utils/cn.util.ts';

	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';

	let {
		data,
		state,
		events,
		ui,
		icon,
		children
	}: AcerolaAccordionCardProps & AcerolaAccordionCardSnippets = $props();

	const collapsible = $derived(ui?.collapsible ?? true);
</script>

{#snippet header()}
	{#if icon}
		<div
			class={cn(
				'flex h-12 w-12 shrink-0 items-center justify-center rounded-inner bg-muted text-foreground transition-[filter] group-hover:brightness-110',
				ui?.iconClass
			)}
		>
			{@render icon()}
		</div>
	{/if}

	<div class="min-w-0 flex-1">
		<p class="truncate text-lg font-bold text-foreground">{data.title}</p>

		{#if data.description}
			<p class="truncate text-sm text-muted-foreground">{data.description}</p>
		{/if}
	</div>
{/snippet}

<div
	class={cn(
		'overflow-hidden rounded-tile border border-border/60 bg-card/60 backdrop-blur-sm transition-all duration-200',
		ui?.class
	)}
>
	{#if collapsible}
		<button
			type="button"
			aria-expanded={state.expanded}
			onclick={events.onToggle}
			class="group flex w-full min-w-0 cursor-pointer items-center gap-4 p-6 text-left transition-colors hover:bg-muted/20"
		>
			{@render header()}

			<ChevronRightIcon
				class={cn(
					'shrink-0 text-muted-foreground transition-transform duration-200 group-hover:text-foreground',
					state.expanded ? 'rotate-90' : ''
				)}
			/>
		</button>
	{:else}
		<div class="flex w-full min-w-0 items-center gap-4 p-6 text-left">
			{@render header()}
		</div>
	{/if}

	{#if !collapsible}
		<div class="border-t border-border/60 p-4">
			<div class="grid gap-4">
				{@render children?.()}
			</div>
		</div>
	{:else if state.expanded}
		<div transition:slide={{ duration: 200 }} class="border-t border-border/60 p-4">
			<div class="grid gap-4">
				{@render children?.()}
			</div>
		</div>
	{/if}
</div>
