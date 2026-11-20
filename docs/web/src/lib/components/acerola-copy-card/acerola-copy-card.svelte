<script lang="ts">
	import type { Component } from 'svelte';
	import CheckIcon from '@lucide/svelte/icons/check';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import MailIcon from '@lucide/svelte/icons/mail';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import { Button } from '#lib/components/ui/button/index.ts';
	import * as Card from '#lib/components/ui/card/index.ts';
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

	let copied = $state(false);
	// Módulo, não closure por instância — se duas CopyCard copiarem quase ao mesmo
	// tempo, o timeout da primeira não pode apagar o estado "copiado" da segunda.
	let resetTimeout: ReturnType<typeof setTimeout> | undefined;

	async function copy() {
		await navigator.clipboard.writeText(value);
		copied = true;
		clearTimeout(resetTimeout);
		resetTimeout = setTimeout(() => (copied = false), 2000);
	}
</script>

<Card.Root
	class="group relative flex-row items-center justify-between gap-4 overflow-hidden rounded-inner border border-border/70 bg-gradient-to-b from-card/90 via-card/60 to-card/40 p-4 ring-0 backdrop-blur-md transition-all duration-200 hover:shadow-lift"
>
	<div class="flex min-w-0 flex-1 items-center gap-3.5">
		{#if IconComponent}
			{@const ResolvedIcon = IconComponent}
			<div
				class="flex size-11 shrink-0 items-center justify-center rounded-inner border bg-primary/10 text-primary transition-transform duration-200 group-hover:scale-105"
			>
				<ResolvedIcon size={20} />
			</div>
		{/if}

		<Card.Header class="[container-type:normal] min-w-0 gap-0.5 p-0">
			<Card.Title class="text-xs font-bold tracking-wider text-muted-foreground uppercase">
				{label}
			</Card.Title>
			<Card.Description>
				{#if href}
					<a
						{href}
						class="text-sm font-semibold text-foreground underline-offset-4 transition-colors hover:text-primary hover:underline"
					>
						{value}
					</a>
				{:else}
					<span class="text-sm font-semibold text-foreground">{value}</span>
				{/if}
			</Card.Description>
		</Card.Header>
	</div>

	<Button
		variant="outline"
		size="sm"
		class="gap-1.5 rounded-inner border-border/70 bg-background/70 px-3 py-1.5 text-xs font-medium transition-all active:scale-95"
		onclick={copy}
		aria-label={copied ? resolvedCopiedLabel : resolvedCopyLabel}
	>
		{#if copied}
			<CheckIcon size={14} class="animate-in text-success zoom-in-50" strokeWidth={2.5} />
			<span class="text-xs font-semibold text-success">{resolvedCopiedLabel}</span>
		{:else}
			<CopyIcon size={14} class="transition-transform group-hover:scale-105" />
			<span class="text-xs font-semibold">{resolvedCopyLabel}</span>
		{/if}
	</Button>
</Card.Root>
