<script lang="ts" module>
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import InfoIcon from '@lucide/svelte/icons/info';
	import LightbulbIcon from '@lucide/svelte/icons/lightbulb';
	import OctagonAlertIcon from '@lucide/svelte/icons/octagon-alert';
	import type { Component } from 'svelte';

	export type AcerolaCalloutType = 'note' | 'tip' | 'caution' | 'danger';

	// Acerola notice: rounded-card block, no border, background and text in the tone of the
	// type (`*-soft` + tone). Each variant spells out its classes instead of interpolating
	// tokens, since Tailwind needs the complete, literal class to generate the CSS.
	const CALLOUT_CONFIG: Record<AcerolaCalloutType, { icon: Component; accent: string }> = {
		note: { icon: InfoIcon, accent: ' bg-primary-soft text-primary' },
		tip: { icon: LightbulbIcon, accent: ' bg-success-soft text-success' },
		caution: { icon: CircleAlertIcon, accent: ' bg-warning-soft text-warning' },
		danger: { icon: OctagonAlertIcon, accent: ' bg-destructive-soft text-destructive' }
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		type = 'note',
		title,
		children
	}: { type?: AcerolaCalloutType; title?: string; children: Snippet } = $props();

	const config = $derived(CALLOUT_CONFIG[type]);

	// `alert` is an assertive live region, reserved for urgent content — using it on
	// static notices (note/tip/caution) would make the screen reader interrupt the user
	// on every client-side navigation just to reread an informational notice.
	const role = $derived(type === 'danger' ? 'alert' : undefined);
</script>

<div
	{role}
	class={[
		'my-6 grid grid-cols-[auto_1fr] items-baseline gap-x-2.5 gap-y-1 rounded-card p-5',
		config.accent
	]}
>
	<config.icon size={14} class="translate-y-px" />
	<p class="m-0 text-xs font-semibold tracking-widest uppercase">{title ?? type}</p>
	<div class="col-start-2 text-sm text-foreground [&>p]:my-0">
		{@render children()}
	</div>
</div>
