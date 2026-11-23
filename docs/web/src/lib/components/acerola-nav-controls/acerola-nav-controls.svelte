<script lang="ts">
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import AcerolaThemePicker from '#lib/components/acerola-theme-picker/acerola-theme-picker.svelte';
	import GithubIcon from '#lib/icons/github.svelte';
	import { GITHUB_URL } from '#lib/constants/site.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale, locales, setLocale, type Locale } from '#lib/paraglide/runtime.js';

	function toggleLocale() {
		const currentIndex = locales.indexOf(getLocale());
		const next = locales[(currentIndex + 1) % locales.length] as Locale;
		setLocale(next);
	}

	const nextLocale = $derived(locales[(locales.indexOf(getLocale()) + 1) % locales.length]);
</script>

<!-- Brand topbar controls: language pill (shows the language it switches to) and round buttons. -->
<div class="flex items-center gap-2">
	<button
		type="button"
		class="flex h-11 items-center gap-1.5 rounded-full bg-surface px-4 text-sm font-black text-foreground"
		aria-label={m['nav.switchLanguage']()}
		onclick={toggleLocale}
	>
		<LanguagesIcon size={16} />
		<span class="uppercase">{nextLocale === 'pt-br' ? 'PT' : nextLocale}</span>
	</button>
	<AcerolaThemePicker />
	<a
		href={GITHUB_URL}
		target="_blank"
		rel="noreferrer"
		aria-label={m['nav.github']()}
		class="flex size-11 items-center justify-center rounded-full bg-surface text-foreground"
	>
		<GithubIcon size={17} />
	</a>
</div>
