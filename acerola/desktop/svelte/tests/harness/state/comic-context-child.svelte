<script lang="ts">
	import { onMount } from 'svelte';
	import { useComicContext, type ActiveComicState } from '#lib/state/comic-context.svelte.ts';

	let {
		onReady
	}: {
		onReady: (context: ActiveComicState) => void;
	} = $props();

	const context = useComicContext();

	onMount(() => {
		onReady(context);
	});
</script>
