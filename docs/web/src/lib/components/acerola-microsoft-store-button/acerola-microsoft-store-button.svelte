<script lang="ts">
	import MicrosoftStoreIcon from '#lib/assets/icons/microsoft-store.svg?component';

	let { label = 'Get it from' }: { label?: string } = $props();

	const STORE_URL = 'https://apps.microsoft.com/store/detail/9N9SL3P7F8GC?cid=DevShareMCLPCS';
</script>

<a
	href={STORE_URL}
	target="_blank"
	rel="noopener noreferrer"
	class="group/badge flex w-full items-center justify-center gap-3 rounded-full bg-foreground px-5 py-3 text-background transition-all hover:scale-[1.02] hover:shadow-lift active:scale-[0.98]"
>
	<MicrosoftStoreIcon class="h-7 w-7 shrink-0" />
	<span class="flex flex-col items-start leading-tight">
		<span class="text-[10px] font-medium tracking-wide text-background/60 uppercase">
			{label}
		</span>
		<span class="text-sm font-bold text-background">Microsoft Store</span>
	</span>
</a>
