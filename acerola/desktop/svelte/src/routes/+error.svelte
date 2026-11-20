<script lang="ts">
	import { page } from '$app/state';
	import AcerolaErrorScreen from '$lib/components/acerola-error-screen/acerola-error-screen.svelte';
	import { m } from '$lib/paraglide/messages';
	import { errorKind } from '$lib/utils/error.utils';

	// Any status lands here: 404 from the router, 5xx from a load. The mascot is a local asset:
	// the app is open source and does not depend on the brand CDN.
	const kind = $derived(
		errorKind(page.status, typeof navigator === 'undefined' ? true : navigator.onLine)
	);
</script>

<svelte:head>
	<title>{m['error.eyebrow']({ status: page.status })} · Acerola</title>
</svelte:head>

<div class="mx-auto flex min-h-full max-w-6xl items-center px-4 sm:px-6">
	<AcerolaErrorScreen
		data={{
			status: page.status,
			kind,
			mascot: '/brand/acerola-mascot.svg',
			homeHref: '/home'
		}}
		events={{ onRetry: () => location.reload(), onBack: () => history.back() }}
		ui={{ page: true }}
	/>
</div>
