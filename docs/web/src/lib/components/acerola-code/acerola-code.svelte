<!--
	Code panel with copy. Design taken from acerola-reader's docs/web acerola-marketing-feature-code
	(window bar, tabs, copy with a success state, scrollable body), moved onto the standard tokens:
	no colored border, success on success-soft.
-->
<script module lang="ts">
	export type AcerolaCodeTab = { id: string; label: string; code: string };
</script>

<script lang="ts">
	import Check from '@lucide/svelte/icons/check';
	import Copy from '@lucide/svelte/icons/copy';
	import { useCopy } from '#lib/hooks/use-copy/use-copy.svelte.ts';
	import { m } from '#lib/paraglide/messages.js';

	type Props = {
		/** A single snippet (code + label) or several tabs. */
		data: { code?: string; label?: string; tabs?: AcerolaCodeTab[] };
		ui?: { maxHeight?: 'sm' | 'md' | 'none' };
	};

	let { data, ui = {} }: Props = $props();

	const clipboard = useCopy(2000);

	const tabs = $derived<AcerolaCodeTab[]>(
		data.tabs?.length
			? data.tabs
			: [{ id: 'code', label: data.label ?? 'html', code: data.code ?? '' }]
	);
	let selected = $state<string | null>(null);
	const current = $derived(tabs.find((tab) => tab.id === selected) ?? tabs[0]);
	const copied = $derived(clipboard.copied !== null && clipboard.copied === current?.code);
</script>

<div class="code flex min-w-0 flex-col overflow-hidden rounded-tile bg-crust shadow-lift">
	<div class="flex items-center justify-between gap-3 bg-mantle px-3 py-2">
		<div class="flex min-w-0 items-center gap-3">
			<span class="flex shrink-0 items-center gap-1.5 pl-1" aria-hidden="true">
				<span class="size-2.5 rounded-full bg-destructive"></span>
				<span class="size-2.5 rounded-full bg-warning"></span>
				<span class="size-2.5 rounded-full bg-success"></span>
			</span>

			{#if tabs.length > 1}
				<div
					class="flex min-w-0 gap-1 overflow-x-auto"
					role="tablist"
					aria-label={m['code.tabs']()}
				>
					{#each tabs as tab (tab.id)}
						<button
							type="button"
							role="tab"
							aria-selected={tab.id === current?.id}
							onclick={() => (selected = tab.id)}
							class="shrink-0 rounded-full px-2.5 py-1 font-mono text-xs font-bold text-muted-foreground transition-colors hover:text-foreground aria-selected:bg-surface aria-selected:text-foreground"
						>
							{tab.label}
						</button>
					{/each}
				</div>
			{:else}
				<span class="truncate font-mono text-xs font-bold text-muted-foreground">
					{current?.label}
				</span>
			{/if}
		</div>

		<button
			type="button"
			onclick={() => current && clipboard.copy(current.code)}
			aria-label={copied ? m['common.copied']() : m['code.copy']()}
			class="flex shrink-0 items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-black transition-all duration-200 ease-brand active:scale-95 {copied
				? 'bg-success-soft text-success'
				: 'bg-surface text-muted-foreground hover:text-foreground'}"
		>
			{#if copied}
				<Check size={13} strokeWidth={2.75} class="code__pop" />
				<span>{m['common.copied']()}</span>
			{:else}
				<Copy size={13} />
				<span>{m['common.copy']()}</span>
			{/if}
		</button>
	</div>

	<div
		class="code__body overflow-auto p-4 sm:p-5 {ui.maxHeight === 'none'
			? ''
			: ui.maxHeight === 'sm'
				? 'max-h-56'
				: 'max-h-96'}"
	>
		<pre class="m-0 w-max min-w-full font-mono text-sm leading-relaxed text-foreground/90"><code
				>{current?.code ?? ''}</code
			></pre>
	</div>
</div>

<style>
	/* Thin scrollbar in the surface tone (the browser default disappears on the dark background). */
	.code__body {
		scrollbar-width: thin;
		scrollbar-color: var(--surface) transparent;
	}

	/* The check mark pops in, as in the reader's docs/web. */
	.code :global(.code__pop) {
		animation: code-pop 220ms var(--ease-brand);
	}
	@keyframes code-pop {
		from {
			transform: scale(0.5);
			opacity: 0;
		}
	}
</style>
