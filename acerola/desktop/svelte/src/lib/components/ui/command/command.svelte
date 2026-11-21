<script lang="ts">
	import { cn } from '#lib/utils/cn.util.ts';
	import { Command as CommandPrimitive } from 'bits-ui';

	export type CommandRootApi = CommandPrimitive.Root;

	let {
		api = $bindable(null),
		ref = $bindable(null),
		value = $bindable(''),
		class: className,
		...restProps
	}: CommandPrimitive.RootProps & {
		api?: CommandRootApi | null;
	} = $props();
</script>

<CommandPrimitive.Root
	bind:this={api}
	bind:value
	bind:ref
	data-slot="command"
	class={cn(
		'flex size-full flex-col overflow-hidden rounded-sheet bg-card p-2 text-card-foreground',
		className
	)}
	{...restProps}
/>
