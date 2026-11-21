<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaSettingRowTone =
		'neutral' | 'primary' | 'success' | 'warning' | 'destructive';

	export type AcerolaSettingRowProps = {
		data: {
			title: string;
			description?: string | null;
		};
		ui?: {
			/** Tone of the round icon: neutral unless the color means something (state). */
			tone?: AcerolaSettingRowTone;
			class?: string;
		};
		icon?: Snippet;
		/** Control on the right: switch, pill button, select… */
		control?: Snippet;
		/** Extra content under the row (an expanded list, a form). */
		children?: Snippet;
	};
</script>

<script lang="ts">
	import { cn } from '#lib/utils/cn.util.ts';

	let { data, ui, icon, control, children }: AcerolaSettingRowProps = $props();

	const TONE_CLASS: Record<AcerolaSettingRowTone, string> = {
		neutral: 'bg-card text-foreground',
		primary: 'bg-primary-soft text-primary',
		success: 'bg-success-soft text-success',
		warning: 'bg-warning-soft text-warning',
		destructive: 'bg-destructive-soft text-destructive'
	};
</script>

<!-- One setting (brand book list row): rounded-tile on bg-surface, round icon, font-black title,
     one line of help and the control on the right. -->
<div data-surface="surface" class={cn('grid min-w-0 gap-3 rounded-tile bg-surface p-4', ui?.class)}>
	<div class="flex min-w-0 items-center gap-4">
		{#if icon}
			<span
				class={cn(
					'relative flex size-10 shrink-0 items-center justify-center rounded-full',
					TONE_CLASS[ui?.tone ?? 'neutral']
				)}
			>
				{@render icon()}
			</span>
		{/if}
		<div class="min-w-0 flex-1">
			<p class="truncate text-sm font-black tracking-tight text-foreground">{data.title}</p>
			{#if data.description}
				<p class="truncate text-xs text-muted-foreground" title={data.description}>
					{data.description}
				</p>
			{/if}
		</div>
		{#if control}
			<div class="flex shrink-0 items-center gap-2">{@render control()}</div>
		{/if}
	</div>
	{#if children}
		{@render children()}
	{/if}
</div>
