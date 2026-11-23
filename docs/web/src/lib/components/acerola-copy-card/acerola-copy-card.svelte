<script lang="ts">
	import type { Component } from 'svelte';
	import CheckIcon from '@lucide/svelte/icons/check';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import MailIcon from '@lucide/svelte/icons/mail';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import { useCopy } from '#lib/hooks/use-copy/use-copy.svelte.ts';
	import { m } from '#lib/paraglide/messages.js';

	let {
		label,
		value,
		href,
		icon,
		copyLabel,
		copiedLabel
	}: {
		label: string;
		value: string;
		href?: string;
		icon?: Component;
		copyLabel?: string;
		copiedLabel?: string;
	} = $props();

	let resolvedCopyLabel = $derived(copyLabel ?? m['copyCard.copy']());
	let resolvedCopiedLabel = $derived(copiedLabel ?? m['copyCard.copied']());

	let IconComponent = $derived(
		icon ??
			(href?.startsWith('mailto:') || value.includes('@')
				? MailIcon
				: href?.startsWith('http')
					? ExternalLinkIcon
					: CopyIcon)
	);

	const clipboard = useCopy(2000);
	const copied = $derived(clipboard.copied === value);
</script>

<!-- Acerola card (solid block, rounded-card, no border) with the brand copy pill (AcerolaCode). -->
<div
	data-slot="card"
	class="flex items-center justify-between gap-4 rounded-card bg-card p-5 shadow-lift"
>
	<div class="flex min-w-0 flex-1 items-center gap-4">
		{#if IconComponent}
			{@const ResolvedIcon = IconComponent}
			<span
				class="flex size-11 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary"
			>
				<ResolvedIcon size={20} />
			</span>
		{/if}

		<div class="grid min-w-0 gap-0.5">
			<span class="text-xs font-semibold text-muted-foreground">{label}</span>
			{#if href}
				<a
					{href}
					class="truncate font-black tracking-tight text-foreground underline-offset-4 hover:text-primary hover:underline"
				>
					{value}
				</a>
			{:else}
				<span class="truncate font-black tracking-tight text-foreground">{value}</span>
			{/if}
		</div>
	</div>

	<button
		type="button"
		onclick={() => clipboard.copy(value)}
		aria-label={copied ? resolvedCopiedLabel : resolvedCopyLabel}
		class="flex shrink-0 items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-black transition-all duration-200 ease-brand active:scale-95 {copied
			? 'bg-success-soft text-success'
			: 'bg-surface text-muted-foreground hover:text-foreground'}"
	>
		{#if copied}
			<CheckIcon size={13} strokeWidth={2.75} />
			<span>{resolvedCopiedLabel}</span>
		{:else}
			<CopyIcon size={13} />
			<span>{resolvedCopyLabel}</span>
		{/if}
	</button>
</div>
