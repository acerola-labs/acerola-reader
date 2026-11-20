<script module lang="ts">
	import type { ToggleGroup as ToggleGroupPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';

	export type AcerolaToggleGroupValue = string | string[] | undefined;

	export type AcerolaToggleGroupProps = {
		state?: {
			value?: AcerolaToggleGroupValue;
		};
		events?: {
			onValueChange?: (value: AcerolaToggleGroupValue) => void;
		};
		config?: {
			type?: ToggleGroupPrimitive.RootProps['type'];
		};
		ui?: Pick<ToggleGroupPrimitive.RootProps, 'class'> & {
			variant?: 'default' | 'outline';
			size?: 'default' | 'sm' | 'lg';
			/// Space (on the Tailwind scale, e.g. `4` = 1rem) between items — default `0`
			/// (items joined, round corners only at the group's ends, like a segmented
			/// control). Any value > 0 turns off this "joined" look and each item gets fully
			/// round corners again (see `group-data-[spacing=0]` in the
			/// `toggle-group-item.svelte` classes) — needed whenever items have a real visual gap
			/// between them, otherwise an item is born with one side unrounded.
			spacing?: number;
		};
		children: Snippet;
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import * as ToggleGroup from '#lib/components/ui/toggle-group/index.ts';

	let { children, config, events, state: control, ui }: AcerolaToggleGroupProps = $props();

	let value: AcerolaToggleGroupValue = $state();
	let lastValue: AcerolaToggleGroupValue = $state();

	const type = $derived(config?.type ?? 'single');
	const variant = $derived(ui?.variant ?? 'default');
	const size = $derived(ui?.size ?? 'default');
	const spacing = $derived(ui?.spacing ?? 0);

	$effect(() => {
		if (control?.value === undefined) return;

		const nextValue = control.value;

		if (nextValue !== untrack(() => lastValue)) {
			value = nextValue;
			lastValue = nextValue;
		}
	});

	$effect(() => {
		if (value === lastValue) return;

		lastValue = value;
		events?.onValueChange?.(value);
	});
</script>

{#if type === 'multiple'}
	<ToggleGroup.Root
		bind:value={value as string[] | undefined}
		type="multiple"
		{variant}
		{size}
		{spacing}
		class={ui?.class}
	>
		{@render children()}
	</ToggleGroup.Root>
{:else}
	<ToggleGroup.Root
		bind:value={value as string | undefined}
		type="single"
		{variant}
		{size}
		{spacing}
		class={ui?.class}
	>
		{@render children()}
	</ToggleGroup.Root>
{/if}
