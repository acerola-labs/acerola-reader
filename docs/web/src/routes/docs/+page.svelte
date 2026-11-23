<script lang="ts">
	import { getSidebar } from '#lib/content/docs.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, localizeHref } from '#lib/paraglide/runtime.js';

	const groups = $derived(getSidebar(getLocale()));
</script>

<svelte:head>
	<title>{m['nav.docs']()} — {m['site.name']()}</title>
</svelte:head>

<div class="doc-content max-w-none">
	<h1 class="mb-8 font-heading text-4xl font-semibold">{m['nav.docs']()}</h1>

	{#each groups as group (group.section)}
		<section class="mb-10">
			<h2 class="mb-3 text-lg font-semibold text-muted-foreground">{group.section}</h2>
			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
				{#each group.docs as doc (doc.slug)}
					<a
						href={localizeHref(`/docs/${doc.slug}`)}
						class="rounded-inner border border-border bg-card p-5 transition-colors hover:bg-accent"
					>
						<span class="block font-heading text-lg font-semibold"
							>{doc.frontmatter.title}</span
						>
						{#if doc.frontmatter.description}
							<span class="mt-1 block text-sm text-muted-foreground"
								>{doc.frontmatter.description}</span
							>
						{/if}
					</a>
				{/each}
			</div>
		</section>
	{/each}
</div>
