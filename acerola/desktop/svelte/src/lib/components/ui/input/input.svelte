<script lang="ts" module>
	import type { HTMLInputAttributes, HTMLInputTypeAttribute } from 'svelte/elements';
	import { cn, type WithElementRef } from '#lib/utils/cn.util.ts';

	type InputType = Exclude<HTMLInputTypeAttribute, 'file'>;

	export type InputProps = WithElementRef<
		Omit<HTMLInputAttributes, 'type'> &
			({ type: 'file'; files?: FileList } | { type?: InputType; files?: undefined })
	>;
</script>

<script lang="ts">
	let {
		type,
		value = $bindable(),
		files = $bindable(),
		ref = $bindable(null),
		class: className,
		'data-slot': dataSlot = 'input',
		...restProps
	}: InputProps = $props();
</script>

{#if type === 'file'}
	<input
		bind:this={ref}
		data-slot={dataSlot}
		class={cn(
			'h-11 w-full min-w-0 rounded-full bg-input px-4.5 text-base font-medium text-foreground transition-shadow outline-none file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60 in-data-[surface=surface]:bg-card aria-invalid:ring-2 aria-invalid:ring-destructive md:text-sm',
			className
		)}
		type="file"
		bind:files
		bind:value
		{...restProps}
	/>
{:else}
	<input
		bind:this={ref}
		data-slot={dataSlot}
		class={cn(
			'h-11 w-full min-w-0 rounded-full bg-input px-4.5 text-base font-medium text-foreground transition-shadow outline-none file:inline-flex file:h-6 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60 in-data-[surface=surface]:bg-card aria-invalid:ring-2 aria-invalid:ring-destructive md:text-sm',
			className
		)}
		{type}
		bind:value
		{...restProps}
	/>
{/if}
