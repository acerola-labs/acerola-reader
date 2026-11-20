<script module lang="ts">
	import { Button, type ButtonProps } from '$lib/components/ui/button';

	export type AcerolaButtonProps = {
		events?: {
			onClick?: ButtonProps['onclick'];
		};
		ui?: Omit<ButtonProps, 'children' | 'class' | 'onclick'> & {
			class?: string;
		};
	};
</script>

<script lang="ts">
	import { cn } from '$lib/utils/cn.utils';
	import type { Snippet } from 'svelte';

	let {
		children,
		events,
		ui
	}: AcerolaButtonProps & {
		children?: Snippet;
	} = $props();

	const uiProps = $derived.by(() => {
		const { class: _class, ...rest } = ui ?? {};
		return rest;
	});
</script>

<Button
	class={cn(
		'font-black enabled:cursor-pointer disabled:pointer-events-auto disabled:cursor-not-allowed',
		ui?.class,
		// Acerola standard: a button is always a pill, whatever the call site passes.
		'rounded-full'
	)}
	onclick={events?.onClick}
	{...uiProps}
>
	{@render children?.()}
</Button>
