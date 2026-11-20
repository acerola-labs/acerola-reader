<script lang="ts" module>
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import type { ThemeModeOption } from '#lib/hooks/theme/use-theme.svelte.ts';
	import type { Component } from 'svelte';

	const MODE_CONFIG: Record<ThemeModeOption, { icon: Component; next: ThemeModeOption }> = {
		light: { icon: SunIcon, next: 'dark' },
		dark: { icon: MoonIcon, next: 'system' },
		system: { icon: MonitorIcon, next: 'light' }
	};
</script>

<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.ts';
	import { useTheme } from '#lib/hooks/theme/use-theme.svelte.ts';
	import { m } from '#lib/paraglide/messages.js';

	const themeCtx = useTheme();

	function nextMode() {
		themeCtx.setMode(MODE_CONFIG[themeCtx.mode].next);
	}
</script>

<Button
	variant="ghost"
	size="icon"
	title={m['themePicker.title']()}
	aria-label={m['themePicker.title']()}
	onclick={nextMode}
>
	{#key themeCtx.mode}
		{@const Icon = MODE_CONFIG[themeCtx.mode].icon}
		<Icon size={17} />
	{/key}
</Button>
