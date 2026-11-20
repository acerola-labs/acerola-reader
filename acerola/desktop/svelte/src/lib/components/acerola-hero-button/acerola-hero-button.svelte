<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaHeroButtonProps = {
		data?: {
			title?: string;
			description?: string;
		};
		events?: {
			onClick?: (event: MouseEvent) => void;
		};
		ui?: {
			class?: string;
			/** Overrides the icon circle's default background/color (`bg-muted text-foreground`) —
			 *  for cases like a status (success/error) where the tinted background is the icon's
			 *  own colored outline, not a generic `bg-muted`. */
			iconClass?: string;
		};
	};

	export type AcerolaHeroButtonSnippets = {
		icon?: Snippet;
		action?: Snippet;
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import * as Item from '#lib/components/ui/item/index.ts';
	import { cn } from '#lib/utils/cn.util.ts';

	let { data, events, ui, icon, action }: AcerolaHeroButtonProps & AcerolaHeroButtonSnippets =
		$props();
</script>

<Item.Root
	class={cn(
		'group flex min-w-0 items-center justify-between rounded-tile border border-border bg-card p-6 transition-colors',
		events?.onClick
			? 'cursor-pointer transition-transform active:scale-[0.98]'
			: '',
		ui?.class
	)}
	onclick={events?.onClick}
>
	<div class="flex min-w-0 flex-1 items-center gap-4">
		{#if icon}
			<Item.Media
				class={cn(
					'flex h-12 w-12 shrink-0 items-center justify-center rounded-inner bg-muted text-foreground transition-[filter] group-hover:brightness-110',
					ui?.iconClass
				)}
			>
				{@render icon()}
			</Item.Media>
		{/if}

		<Item.Content class="min-w-0 flex-1 text-left">
			<Item.Title class="block w-full truncate text-lg font-bold text-foreground">
				{data?.title ?? m['components.heroButton.default.title']()}
			</Item.Title>

			{#if data?.description}
				<Item.Description class="block w-full truncate text-sm text-muted-foreground"
					>{data.description}</Item.Description
				>
			{/if}
		</Item.Content>
	</div>

	{#if action}
		<Item.Actions class="ml-4 shrink-0">
			{@render action()}
		</Item.Actions>
	{/if}
</Item.Root>
