<script module lang="ts">
	export type AcerolaSelectOption = {
		value: string;
		label: string;
		color?: number;
	};

	export type AcerolaSelectProps = {
		data: {
			options: AcerolaSelectOption[];
		};
		state?: {
			value?: string;
		};
		events?: {
			onValueChange?: (value: string) => void;
		};
		ui?: {
			class?: string;
			placeholder?: string;
		};
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from '#lib/components/ui/select/index.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils/cn.util.ts';

	let { data, state: control, events, ui }: AcerolaSelectProps = $props();

	let value = $state('');
	let lastValue = $state('');

	const placeholder = $derived(ui?.placeholder ?? m['components.select.placeholder']());

	let selectedOption = $derived(data.options.find((it) => it.value === value));
	let selectedLabel = $derived(selectedOption?.label ?? placeholder);

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

<Select type="single" bind:value>
	<SelectTrigger
		class={cn(
			'h-11 w-auto min-w-48 justify-between rounded-full border-0 bg-input px-4.5 font-medium transition-[filter] hover:brightness-110',
			ui?.class
		)}
	>
		<div class="flex items-center gap-2">
			{#if selectedOption?.color != null}
				{@const hex = (selectedOption.color & 0xffffff).toString(16).padStart(6, '0')}
				<div
					class="flex items-center gap-2 rounded-full px-2.5 py-0.5"
					style="background-color: #{hex}26; color: #{hex}"
				>
					<div class="h-2 w-2 rounded-full" style="background-color: #{hex}"></div>
					<span class="text-xs font-black">{selectedLabel}</span>
				</div>
			{:else}
				<span class="text-sm font-medium text-foreground">{selectedLabel}</span>
			{/if}
		</div>
	</SelectTrigger>

	<SelectContent>
		{#each data.options as option (option.value)}
			<SelectItem value={option.value} label={option.label} class="cursor-pointer">
				<div class="flex items-center gap-2 py-0.5">
					{#if option.color != null}
						{@const hex = (option.color & 0xffffff).toString(16).padStart(6, '0')}
						<div
							class="flex items-center gap-2 rounded-inner px-2 py-0.5 transition-colors"
							style="background-color: #{hex}15; color: #{hex}"
						>
							<div
								class="h-2 w-2 rounded-full"
								style="background-color: #{hex}"
							></div>
							<span class="text-xs font-black">{option.label}</span>
						</div>
					{:else}
						<span class="text-sm font-medium">{option.label}</span>
					{/if}
				</div>
			</SelectItem>
		{/each}
	</SelectContent>
</Select>
