<script lang="ts">
	import { Toaster as Sonner, type ToasterProps as SonnerProps } from 'svelte-sonner';
	import { mode } from 'mode-watcher';
	import { fade } from 'svelte/transition';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import OctagonXIcon from '@lucide/svelte/icons/octagon-x';
	import InfoIcon from '@lucide/svelte/icons/info';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import { toastIconEnter } from '#lib/utils/toast-icon-motion.util.ts';

	let { toastOptions, class: className, ...restProps }: SonnerProps = $props();
</script>

<Sonner
	theme={mode.current}
	class="toaster group {className ?? ''}"
	toastOptions={{
		classes: {
			// Brand book `.ac-toast`: rounded-card, no border, bg-card + shadow-float; each type takes
			// its soft background and its tone as text.
			toast: 'group toast flex items-center gap-3.5 rounded-card! border-0! bg-card! px-4.5! py-3.5! text-sm text-card-foreground! shadow-float! data-[type=success]:bg-success-soft! data-[type=success]:text-success! data-[type=error]:bg-destructive-soft! data-[type=error]:text-destructive! data-[type=warning]:bg-warning-soft! data-[type=warning]:text-warning! data-[type=info]:bg-info-soft! data-[type=info]:text-info!',
			title: 'font-black text-sm tracking-tight',
			description: 'text-[13px] opacity-80 leading-relaxed mt-0.5',
			actionButton:
				'rounded-full! bg-foreground! text-background! font-black text-xs px-3.5 h-8 transition-[filter] hover:brightness-110',
			cancelButton:
				'rounded-full! bg-surface! text-foreground! font-black text-xs px-3.5 h-8 transition-[filter] hover:brightness-110',
			closeButton:
				'rounded-full! border-0! bg-surface! text-muted-foreground hover:text-foreground transition-colors',
			...toastOptions?.classes
		},
		...toastOptions
	}}
	style="--normal-bg: var(--color-card); --normal-text: var(--color-card-foreground); --normal-border: transparent; --border-radius: var(--radius-card);"
	{...restProps}
>
	{#snippet loadingIcon()}
		<span use:toastIconEnter={'loading'} out:fade={{ duration: 120 }}>
			<Loader2Icon class="size-4 animate-spin" />
		</span>
	{/snippet}
	{#snippet successIcon()}
		<span use:toastIconEnter={'success'} out:fade={{ duration: 120 }}>
			<CircleCheckIcon class="size-4" />
		</span>
	{/snippet}
	{#snippet errorIcon()}
		<span use:toastIconEnter={'error'} out:fade={{ duration: 120 }}>
			<OctagonXIcon class="size-4" />
		</span>
	{/snippet}
	{#snippet infoIcon()}
		<span use:toastIconEnter={'info'} out:fade={{ duration: 120 }}>
			<InfoIcon class="size-4" />
		</span>
	{/snippet}
	{#snippet warningIcon()}
		<span use:toastIconEnter={'warning'} out:fade={{ duration: 120 }}>
			<TriangleAlertIcon class="size-4" />
		</span>
	{/snippet}
</Sonner>
