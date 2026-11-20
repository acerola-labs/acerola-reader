<script lang="ts">
	import { page } from '$app/state';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils/cn.util.ts';
	import { slidingIndicator } from '#lib/utils/sliding-indicator.util.ts';

	type Heading = { id: string; text: string; depth: 2 | 3 };

	let { containerSelector = '.doc-content' }: { containerSelector?: string } = $props();

	let headings = $state<Heading[]>([]);
	let activeId = $state<string | null>(null);

	$effect(() => {
		// Reads the current route only to force the effect to rerun on every navigation
		// between doc pages — the `.doc-content` container isn't remounted because the
		// Toc lives in the layout, so without this dependency the headings get stuck on
		// the first page visited.
		page.url.pathname;

		const container = document.querySelector(containerSelector);
		if (!container) return;

		activeId = null;

		const nodes = [...container.querySelectorAll('h2, h3')] as HTMLElement[];
		headings = nodes
			.filter((node) => node.id)
			.map((node) => ({
				id: node.id,
				text: node.textContent ?? '',
				depth: node.tagName === 'H3' ? 3 : 2
			}));

		const observer = new IntersectionObserver(
			(observedEntries) => {
				const visible = observedEntries.find((entry) => entry.isIntersecting);
				if (visible) activeId = visible.target.id;
			},
			{ rootMargin: '-96px 0px -70% 0px' }
		);

		for (const node of nodes) observer.observe(node);

		return () => observer.disconnect();
	});
</script>

{#if headings.length > 0}
	<nav class="text-sm">
		<p class="mb-3 text-xs font-semibold text-muted-foreground">
			{m['toc.title']()}
		</p>
		<ul
			use:slidingIndicator={{
				selector: '[aria-current="true"]',
				indicatorClass: 'w-0.5 rounded-full bg-primary',
				track: ['y', 'height']
			}}
			class="flex flex-col gap-1.5 border-l border-border"
		>
			{#each headings as heading (heading.id)}
				<li style={heading.depth === 3 ? 'padding-left: 1rem' : ''}>
					<a
						href="#{heading.id}"
						aria-current={activeId === heading.id ? 'true' : undefined}
						class={cn(
							'block py-0.5 pl-3 transition-colors',
							activeId === heading.id
								? 'font-medium text-primary'
								: 'text-muted-foreground hover:text-foreground'
						)}
					>
						{heading.text}
					</a>
				</li>
			{/each}
		</ul>
	</nav>
{/if}
