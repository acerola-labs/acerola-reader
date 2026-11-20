<script lang="ts">
	import { onMount } from 'svelte';
	import { useComicChapters } from '#lib/hooks/store/use-comic-chapters.svelte.ts';

	let {
		onReady
	}: {
		onReady: (hook: ReturnType<typeof useComicChapters>) => void;
	} = $props();

	const hook = useComicChapters();

	onMount(() => {
		onReady(hook);
	});
</script>
