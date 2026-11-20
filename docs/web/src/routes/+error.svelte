<script lang="ts">
	import { page } from '$app/state';
	import AcerolaErrorScreen from '#lib/components/acerola-error-screen/acerola-error-screen.svelte';
	import { BRAND_MASCOT_URL } from '#lib/constants/site.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { localizeHref } from '#lib/paraglide/runtime.js';
	import { errorKind } from '#lib/utils/error.util.ts';

	// Any status lands here: 404 from the router, 5xx from a load.
	const kind = $derived(
		errorKind(page.status, typeof navigator === 'undefined' ? true : navigator.onLine)
	);
</script>

<svelte:head>
	<title>{m['error.eyebrow']({ status: page.status })} · {m['site.name']()}</title>
	<meta name="robots" content="noindex" />
</svelte:head>

<div class="mx-auto flex min-h-svh max-w-6xl items-center px-4 sm:px-6">
	<AcerolaErrorScreen
		data={{
			status: page.status,
			kind,
			mascot: BRAND_MASCOT_URL,
			homeHref: localizeHref('/')
		}}
		events={{ onRetry: () => location.reload(), onBack: () => history.back() }}
		ui={{ page: true }}
	/>
</div>
