<script lang="ts">
	import { Command as CommandPrimitive } from 'bits-ui';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import SearchIcon from '@lucide/svelte/icons/search';
	import SearchXIcon from '@lucide/svelte/icons/search-x';
	import * as Command from '#lib/components/ui/command/index.ts';
	import * as InputGroup from '#lib/components/ui/input-group/index.ts';
	import { m } from '#lib/paraglide/messages.js';

	type PagefindResult = { url: string; excerpt: string; meta: { title?: string } };
	type PagefindApi = {
		search: (query: string) => Promise<{ results: { data: () => Promise<PagefindResult> }[] }>;
	};

	let { open = $bindable(false) }: { open: boolean } = $props();

	let query = $state('');
	let results = $state<PagefindResult[]>([]);
	let status = $state<'idle' | 'loading' | 'unavailable'>('idle');
	// $state (not a plain `let`): the search $effect must rerun when the dynamic import of
	// pagefind.js resolves — without this, a query typed before pagefind finishes loading
	// clears `results` and never retries (no reactive event triggers the rerun), no matter
	// how long the test/assertion timeout waits.
	let pagefind = $state<PagefindApi | null>(null);

	async function ensurePagefind() {
		if (pagefind || status === 'unavailable') return;

		// The Pagefind index only exists in the built output (see `npm run build`), so
		// there's nothing to fetch in dev — skip straight to the "unavailable" state
		// instead of letting Vite's dev-server import analysis choke on a missing file.
		if (import.meta.env.DEV) {
			status = 'unavailable';
			return;
		}

		status = 'loading';
		try {
			// Built from a runtime string (not a literal) so Vite/Rollup can't try to
			// resolve this at build time — the file only exists in the deployed output.
			const pagefindEntry = ['', 'pagefind', 'pagefind.js'].join('/');
			pagefind = (await import(/* @vite-ignore */ pagefindEntry)) as unknown as PagefindApi;
			status = 'idle';
		} catch {
			status = 'unavailable';
		}
	}

	async function runSearch(value: string) {
		if (!pagefind || !value) {
			results = [];
			return;
		}

		const search = await pagefind.search(value);
		const data = await Promise.all(search.results.slice(0, 8).map((result) => result.data()));

		// Pagefind indexes the static build and returns the real file path
		// (e.g. "/docs/architecture.html"), but SvelteKit routes are served without an
		// extension — without this the result link lands on a 404.
		results = data.map((result) => ({
			...result,
			url: result.url.replace(/(?:\/index)?\.html$/, '')
		}));
	}

	$effect(() => {
		if (!open) return;
		ensurePagefind();
	});

	$effect(() => {
		runSearch(query);
	});

	function handleKeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			open = !open;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<Command.Dialog
	bind:open
	shouldFilter={false}
	title={m['search.title']()}
	description={m['search.title']()}
	class="top-1/4 w-full max-w-lg translate-y-0 gap-0 overflow-hidden rounded-inner! p-0 shadow-float ring-1 ring-border sm:max-w-lg"
>
	<div data-slot="command-input-wrapper" class="p-3 pb-2">
		<InputGroup.Root class="h-11 rounded-inner border-border bg-input/50">
			<InputGroup.Addon>
				<SearchIcon class="size-4 shrink-0 opacity-50" />
			</InputGroup.Addon>
			<CommandPrimitive.Input
				bind:value={query}
				data-slot="command-input"
				placeholder={m['search.placeholder']()}
			>
				{#snippet child({ props })}
					<InputGroup.Input {...props} bind:value={query} />
				{/snippet}
			</CommandPrimitive.Input>
		</InputGroup.Root>
	</div>
	<Command.List class="px-2 pb-2">
		{#if status === 'unavailable'}
			<Command.Empty class="flex flex-col items-center gap-2 py-10 text-muted-foreground">
				<SearchXIcon size={20} />
				<span>{m['search.unavailableDev']()}</span>
			</Command.Empty>
		{:else if status === 'loading'}
			<Command.Empty class="flex flex-col items-center gap-2 py-10 text-muted-foreground">
				<Loader2Icon size={20} class="animate-spin" />
				<span>{m['search.loading']()}</span>
			</Command.Empty>
		{:else if !query}
			<Command.Empty class="flex flex-col items-center gap-2 py-10 text-muted-foreground">
				<SearchIcon size={20} />
				<span>{m['search.idle']()}</span>
			</Command.Empty>
		{:else if query && results.length === 0}
			<Command.Empty class="flex flex-col items-center gap-2 py-10 text-muted-foreground">
				<SearchXIcon size={20} />
				<span>{m['search.noResults']()}</span>
			</Command.Empty>
		{:else}
			{#each results as result (result.url)}
				<Command.LinkItem
					href={result.url}
					onclick={() => (open = false)}
					class="rounded-inner py-2"
				>
					<FileTextIcon size={16} class="mt-0.5 shrink-0 self-start" />
					<div class="flex min-w-0 flex-col gap-0.5">
						<span class="truncate font-medium">{result.meta?.title ?? result.url}</span>
						<span class="line-clamp-2 text-xs text-muted-foreground"
							>{@html result.excerpt}</span
						>
					</div>
				</Command.LinkItem>
			{/each}
		{/if}
	</Command.List>
</Command.Dialog>
