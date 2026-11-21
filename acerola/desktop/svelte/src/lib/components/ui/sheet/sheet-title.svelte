<script lang="ts">
	import { Dialog as SheetPrimitive } from 'bits-ui';
	import { cn } from '#lib/utils/cn.util.ts';

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: SheetPrimitive.TitleProps = $props();
</script>

<SheetPrimitive.Title
	bind:ref
	data-slot="sheet-title"
	class={cn('pr-12 text-lg leading-tight font-black tracking-tight text-foreground', className)}
	{...restProps}
/>
