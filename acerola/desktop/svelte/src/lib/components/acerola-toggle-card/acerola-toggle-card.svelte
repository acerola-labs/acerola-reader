<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaToggleCardProps = {
		data: {
			title: string;
			subtitle?: string;
		};
		state: {
			/** Controls the highlighted border/background and the check badge — "this source/option
			 *  is on now". May stay always `false` in a consumer that only uses the card as a
			 *  simple "drawer" (disclosure), without an on/off notion (see usage in
			 *  `acerola-bookmark-manager.svelte`). */
			active: boolean;
			/** Shows/hides `children` with a slide transition. Independent of `active` — a
			 *  consumer can tie both together (the content shows when the source turns on, see
			 *  the external sync toggle in `acerola-comic-preferences.svelte`) or keep them apart
			 *  (expanding only shows/manages something already active for another reason, see
			 *  the custom relays card). */
			expanded?: boolean;
		};
		events: {
			onClick: () => void;
		};
		ui?: {
			disabled?: boolean;
			class?: string;
			/** Overrides the icon circle's default background/color (`bg-muted text-foreground`) —
			 *  same pattern as AcerolaHeroButton/AcerolaAccordionCard's `ui.iconClass`. */
			iconClass?: string;
		};
	};

	export type AcerolaToggleCardSnippets = {
		icon?: Snippet;
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { slide } from 'svelte/transition';
	import CheckIcon from '@lucide/svelte/icons/check';
	import { cn } from '#lib/utils/cn.util.ts';
	import { checkBadgePop } from '#lib/utils/check-badge-motion.util.ts';

	let {
		data,
		state,
		events,
		ui,
		icon,
		children
	}: AcerolaToggleCardProps & AcerolaToggleCardSnippets = $props();
</script>

<div
	class={cn(
		'overflow-hidden rounded-inner border-2 transition-all duration-200',
		state.active
			? ' bg-primary/5'
			: 'border-border/60 bg-card',
		ui?.class
	)}
>
	<button
		type="button"
		onclick={events.onClick}
		disabled={ui?.disabled}
		class="group flex w-full items-center gap-3 p-4 text-left enabled:cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
	>
		{#if icon}
			<div
				class={cn(
					'flex size-10 shrink-0 items-center justify-center rounded-inner bg-muted text-foreground transition-[filter] group-hover:brightness-110',
					ui?.iconClass
				)}
			>
				{@render icon()}
			</div>
		{/if}
		<div class="min-w-0 flex-1">
			<p class="text-sm font-semibold text-foreground">{data.title}</p>
			{#if data.subtitle}
				<p class="truncate text-xs text-muted-foreground">{data.subtitle}</p>
			{/if}
		</div>
		{#if state.active}
			<div
				use:checkBadgePop
				class="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground"
			>
				<CheckIcon size={12} strokeWidth={3} />
			</div>
		{/if}
	</button>

	{#if state.expanded && children}
		<div transition:slide={{ duration: 200 }} class="space-y-2 border-t border-border/60 p-3">
			{@render children()}
		</div>
	{/if}
</div>
