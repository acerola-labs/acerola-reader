<script lang="ts">
	import { Dialog as DialogPrimitive } from 'bits-ui';
	import DialogPortal from './dialog-portal.svelte';
	import type { Snippet } from 'svelte';
	import * as Dialog from './index';
	import { cn, type WithoutChildrenOrChild } from '#lib/utils/cn.util.ts';
	import type { ComponentProps } from 'svelte';
	import { Button } from '#lib/components/ui/button/index.ts';
	import XIcon from '@lucide/svelte/icons/x';
	import { m } from '#lib/paraglide/messages.js';

	let {
		ref = $bindable(null),
		class: className,
		portalProps,
		children,
		showCloseButton = true,
		...restProps
	}: WithoutChildrenOrChild<DialogPrimitive.ContentProps> & {
		portalProps?: WithoutChildrenOrChild<ComponentProps<typeof DialogPortal>>;
		children: Snippet;
		showCloseButton?: boolean;
	} = $props();
</script>

<DialogPortal {...portalProps}>
	<Dialog.Overlay />
	<DialogPrimitive.Content
		bind:ref
		data-slot="dialog-content"
		class={cn(
			'fixed top-1/2 left-1/2 z-50 grid w-full max-w-[calc(100%-2rem)] -translate-x-1/2 -translate-y-1/2 gap-5 rounded-sheet bg-card p-6 text-sm text-card-foreground shadow-float duration-200 ease-brand outline-none sm:max-w-sm data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95',
			className
		)}
		{...restProps}
	>
		{@render children?.()}
		{#if showCloseButton}
			<DialogPrimitive.Close data-slot="dialog-close">
				{#snippet child({ props })}
					<Button
						variant="ghost"
						class="absolute top-4 right-4 rounded-full bg-surface"
						size="icon"
						{...props}
					>
						<XIcon />
						<span class="sr-only">{m['components.dialog.close']()}</span>
					</Button>
				{/snippet}
			</DialogPrimitive.Close>
		{/if}
	</DialogPrimitive.Content>
</DialogPortal>
