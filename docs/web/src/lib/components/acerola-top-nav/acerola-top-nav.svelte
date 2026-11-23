<script lang="ts">
	import MenuIcon from '@lucide/svelte/icons/menu';
	import SearchIcon from '@lucide/svelte/icons/search';
	import { page } from '$app/state';
	import AcerolaNavControls from '#lib/components/acerola-nav-controls/acerola-nav-controls.svelte';
	import { BRAND_LOGO_URL } from '#lib/constants/site.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { localizeHref } from '#lib/paraglide/runtime.js';

	let {
		onOpenSearch,
		onOpenMobileNav
	}: { onOpenSearch: () => void; onOpenMobileNav: () => void } = $props();

	const inDocs = $derived(page.url.pathname.includes('/docs'));
	const links = $derived([
		{ href: localizeHref('/'), label: m['nav.home'](), current: !inDocs },
		{ href: localizeHref('/docs/getting-started'), label: m['nav.docs'](), current: inDocs }
	]);
</script>

<!-- Acerola topbar (brand book): no border, blurred background, logo + font-black name, pill nav on
     surface with the current item in foreground, round controls. -->
<header class="sticky top-0 z-40 bg-background/85 backdrop-blur-md">
	<div class="mx-auto flex max-w-6xl items-center gap-3 px-4 py-3 sm:px-6">
		<button
			type="button"
			class="flex size-11 shrink-0 items-center justify-center rounded-full bg-surface text-foreground md:hidden"
			aria-label={m['nav.menuToggle']()}
			onclick={onOpenMobileNav}
		>
			<MenuIcon size={18} />
		</button>

		<a href={localizeHref('/')} class="flex shrink-0 items-center gap-2.5">
			<img src={BRAND_LOGO_URL} alt="" class="size-9" />
			<span class="text-xl font-black tracking-tight">{m['site.name']()}</span>
		</a>

		<nav class="ml-4 hidden items-center gap-1 rounded-full bg-surface p-1 md:flex">
			{#each links as link (link.href)}
				<a
					href={link.href}
					aria-current={link.current ? 'page' : undefined}
					class="rounded-full px-4 py-2 text-sm font-bold text-muted-foreground transition-colors hover:text-foreground aria-[current=page]:bg-foreground aria-[current=page]:text-background"
				>
					{link.label}
				</a>
			{/each}
		</nav>

		<button
			type="button"
			class="ml-auto hidden h-11 min-w-0 flex-1 items-center gap-2 rounded-full bg-surface px-4 text-sm text-muted-foreground transition-colors hover:text-foreground md:flex md:max-w-xs"
			onclick={onOpenSearch}
		>
			<SearchIcon size={16} class="shrink-0" />
			<span class="min-w-0 flex-1 truncate text-left">{m['nav.searchPlaceholder']()}</span>
			<kbd
				class="hidden shrink-0 rounded-full bg-background px-2 py-0.5 font-sans text-xs font-bold lg:inline"
			>
				Ctrl K
			</kbd>
		</button>

		<button
			type="button"
			class="ml-auto flex size-11 shrink-0 items-center justify-center rounded-full bg-surface text-foreground md:hidden"
			aria-label={m['nav.searchPlaceholder']()}
			onclick={onOpenSearch}
		>
			<SearchIcon size={18} />
		</button>

		<div class="hidden shrink-0 md:block">
			<AcerolaNavControls />
		</div>
	</div>
</header>
