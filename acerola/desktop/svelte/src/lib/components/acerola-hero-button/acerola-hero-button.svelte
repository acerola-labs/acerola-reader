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

<!-- Brand book list row (same look as AcerolaSettingRow): rounded-tile on bg-surface, round icon
     on bg-card, font-black title, one line of help and the action on the right. -->
<Item.Root
	data-surface="surface"
	class={cn(
		'group flex min-w-0 items-center justify-between gap-4 rounded-tile bg-surface p-4 transition-[filter] duration-200 ease-brand',
		events?.onClick ? 'cursor-pointer hover:brightness-110 active:scale-[0.99]' : '',
		ui?.class
	)}
	onclick={events?.onClick}
>
	<div class="flex min-w-0 flex-1 items-center gap-4">
		{#if icon}
			<Item.Media
				class={cn(
					'flex size-10 shrink-0 items-center justify-center rounded-full bg-card text-foreground',
					ui?.iconClass
				)}
			>
				{@render icon()}
			</Item.Media>
		{/if}

		<Item.Content class="min-w-0 flex-1 text-left">
			<Item.Title
				class="block w-full truncate text-sm font-black tracking-tight text-foreground"
			>
				{data?.title ?? m['components.heroButton.default.title']()}
			</Item.Title>

			{#if data?.description}
				<Item.Description class="block w-full truncate text-xs text-muted-foreground"
					>{data.description}</Item.Description
				>
			{/if}
		</Item.Content>
	</div>

	{#if action}
		<Item.Actions class="flex shrink-0 items-center gap-2">
			{@render action()}
		</Item.Actions>
	{/if}
</Item.Root>
