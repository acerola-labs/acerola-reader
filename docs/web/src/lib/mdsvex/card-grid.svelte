<script lang="ts">
	import type { Snippet } from 'svelte';

	// Two columns inside the docs text column; the landing asks for three.
	let { columns = 2, children }: { columns?: 2 | 3; children: Snippet } = $props();
</script>

<!-- Acerola card grid (brand book): plain grid, gap-4. -->
<div class={['card-grid my-6 grid gap-4 sm:grid-cols-2', columns === 3 && 'lg:grid-cols-3']}>
	{@render children()}
</div>
