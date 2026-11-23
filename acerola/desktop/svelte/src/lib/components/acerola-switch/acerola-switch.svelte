<script module lang="ts">
	import { Switch as SwitchPrimitive } from 'bits-ui';
	import type { WithoutChildrenOrChild } from '#lib/utils/cn.util.ts';

	export type AcerolaSwitchProps = {
		state?: {
			checked?: boolean;
		};
		events?: {
			onCheckedChange?: (checked: boolean) => void;
		};
		ui?: Omit<WithoutChildrenOrChild<SwitchPrimitive.RootProps>, 'checked' | 'class'> & {
			class?: string;
			size?: 'sm' | 'default';
		};
	};
</script>

<script lang="ts">
	import { cn } from '#lib/utils/cn.util.ts';

	let { events, state: control, ui }: AcerolaSwitchProps = $props();

	let localChecked = $state(false);
	const isControlled = $derived(control?.checked !== undefined);
	const currentChecked = $derived(isControlled ? Boolean(control?.checked) : localChecked);

	const restProps = $derived.by(() => {
		const { class: _class, size: _size, ...rest } = ui ?? {};
		return rest;
	});

	function handleCheckedChange(nextChecked: boolean) {
		if (!isControlled) {
			localChecked = nextChecked;
		}
		events?.onCheckedChange?.(nextChecked);
	}
</script>

<SwitchPrimitive.Root
	checked={currentChecked}
	onCheckedChange={handleCheckedChange}
	data-slot="switch"
	data-size={ui?.size ?? 'default'}
	class={cn(
		'peer group/switch relative inline-flex shrink-0 cursor-pointer items-center rounded-full p-0.5 transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60 data-[state=checked]:bg-primary data-[state=unchecked]:bg-muted-foreground/35',
		(ui?.size ?? 'default') === 'default' ? 'h-8 w-14' : 'h-6 w-11',
		ui?.class
	)}
	{...restProps}
>
	<SwitchPrimitive.Thumb
		class={cn(
			// Off: light thumb on a grey track; on: dark thumb on primary. The track must never
			// match the row it sits on (`input` and `surface` are the same color).
			'pointer-events-none block rounded-full shadow-lift ring-0 transition-transform duration-200 ease-brand data-[state=checked]:translate-x-full data-[state=checked]:bg-primary-foreground data-[state=unchecked]:translate-x-0 data-[state=unchecked]:bg-foreground',
			(ui?.size ?? 'default') === 'default' ? 'h-6 w-6' : 'h-5 w-5'
		)}
	/>
</SwitchPrimitive.Root>
