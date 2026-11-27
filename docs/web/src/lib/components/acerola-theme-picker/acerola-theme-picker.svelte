<script lang="ts">
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import { Button } from '#lib/components/ui/button/index.ts';
	import { useTheme } from '#lib/hooks/theme/use-theme.svelte.ts';
	import { m } from '#lib/paraglide/messages.js';

	const themeCtx = useTheme();
</script>

<!-- Shows the theme it switches to, like the brand book: sun on dark, moon on light. -->
<Button
	variant="ghost"
	size="icon"
	title={m['themePicker.title']()}
	aria-label={m['themePicker.title']()}
	onclick={themeCtx.toggle}
>
	{#if themeCtx.mode === 'dark'}<SunIcon size={17} />{:else}<MoonIcon size={17} />{/if}
</Button>
