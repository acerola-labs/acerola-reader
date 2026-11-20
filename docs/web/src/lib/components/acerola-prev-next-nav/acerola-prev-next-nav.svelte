<script lang="ts">
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import type { DocEntry } from '#lib/content/docs.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { localizeHref } from '#lib/paraglide/runtime.js';
	import { cn } from '#lib/utils/cn.util.ts';

	let {
		prev,
		next,
		position = 'bottom'
	}: { prev: DocEntry | null; next: DocEntry | null; position?: 'top' | 'bottom' } = $props();
</script>

{#if prev || next}
	<div
		class={cn(
			'grid grid-cols-1 gap-3 sm:grid-cols-2',
			position === 'bottom' ? 'mt-14' : 'mb-10'
		)}
	>
		{#if prev}
			<a
				href={localizeHref(`/docs/${prev.slug}`)}
				class="group grid gap-1 rounded-card bg-card p-5 shadow-lift transition-all duration-200 ease-brand hover:-translate-y-0.5 hover:shadow-float"
			>
				<span class="flex items-center gap-1 text-xs font-semibold text-muted-foreground">
					<ArrowLeftIcon size={14} />
					{m['prevNext.previous']()}
				</span>
				<span class="block font-black tracking-tight">{prev.frontmatter.title}</span>
			</a>
		{:else}
			<div></div>
		{/if}

		{#if next}
			<a
				href={localizeHref(`/docs/${next.slug}`)}
				class="group grid gap-1 rounded-card bg-card p-5 text-right shadow-lift transition-all duration-200 ease-brand hover:-translate-y-0.5 hover:shadow-float"
			>
				<span
					class="flex items-center justify-end gap-1 text-xs font-semibold text-muted-foreground"
				>
					{m['prevNext.next']()}
					<ArrowRightIcon size={14} />
				</span>
				<span class="block font-black tracking-tight">{next.frontmatter.title}</span>
			</a>
		{/if}
	</div>
{/if}
