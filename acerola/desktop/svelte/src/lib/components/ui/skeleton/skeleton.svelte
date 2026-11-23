<script lang="ts">
	import { cn, type WithElementRef, type WithoutChildren } from '#lib/utils/cn.util.ts';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: WithoutChildren<WithElementRef<HTMLAttributes<HTMLDivElement>>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="skeleton"
	class={cn('animate-pulse rounded-inner bg-surface', className)}
	{...restProps}
></div>
