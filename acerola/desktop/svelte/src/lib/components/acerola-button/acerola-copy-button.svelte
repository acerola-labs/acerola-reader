<script module lang="ts">
	export type AcerolaCopyButtonProps = {
		value?: string | null;
		label?: string;
		copiedLabel?: string;
		variant?: 'default' | 'contrast' | 'outline' | 'ghost' | 'secondary';
		size?: 'default' | 'sm' | 'icon' | 'icon-sm';
		class?: string;
		disabled?: boolean;
		onCopy?: () => void;
	};
</script>

<script lang="ts">
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import { Button } from '#lib/components/ui/button/index.ts';
	import { cn } from '#lib/utils/cn.util.ts';
	import { m } from '#lib/paraglide/messages.js';

	let {
		value,
		label,
		copiedLabel,
		variant = 'secondary',
		size = 'default',
		class: className,
		disabled = false,
		onCopy
	}: AcerolaCopyButtonProps = $props();

	let defaultLabel = $derived(label ?? m['components.copyButton.copy']());
	let defaultCopiedLabel = $derived(copiedLabel ?? m['components.copyButton.copied']());

	let copied = $state(false);
	let timeoutId: ReturnType<typeof setTimeout> | undefined;

	async function handleCopy(e: MouseEvent) {
		e.stopPropagation();
		if (!value || copied) return;

		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(value);
			} else {
				const textArea = document.createElement('textarea');
				textArea.value = value;
				document.body.appendChild(textArea);
				textArea.select();
				document.execCommand('copy');
				document.body.removeChild(textArea);
			}
			copied = true;
			onCopy?.();
			clearTimeout(timeoutId);
			timeoutId = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy', err);
		}
	}
</script>

<Button
	type="button"
	{variant}
	size={size === 'icon-sm' ? 'icon' : size}
	disabled={disabled || !value}
	onclick={handleCopy}
	class={cn(
		'relative gap-2 transition-[filter,background-color,color] duration-200 ease-brand',
		size === 'icon-sm' && 'size-9',
		copied && 'bg-success-soft! text-success!',
		className
	)}
	aria-label={copied ? defaultCopiedLabel : defaultLabel}
>
	<!-- Brand book copy pill (AcerolaCode): icon + verb, turning into a check in the success tone. -->
	{#if copied}
		<CheckIcon size={16} class="animate-in duration-200 zoom-in-50" strokeWidth={2.5} />
		{#if copiedLabel}
			<span>{defaultCopiedLabel}</span>
		{:else if label}
			<span>{defaultLabel}</span>
		{/if}
	{:else}
		<CopyIcon size={16} />
		{#if label}
			<span>{defaultLabel}</span>
		{/if}
	{/if}
</Button>
