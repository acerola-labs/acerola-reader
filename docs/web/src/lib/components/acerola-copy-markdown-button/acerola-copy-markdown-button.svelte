<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import ClipboardCopyIcon from '@lucide/svelte/icons/clipboard-copy';
	import XIcon from '@lucide/svelte/icons/x';

	let {
		raw,
		url,
		label = 'Copy page as Markdown',
		copiedLabel = 'Copied',
		failedLabel = "Couldn't copy"
	}: {
		raw: string;
		url: string;
		label?: string;
		copiedLabel?: string;
		failedLabel?: string;
	} = $props();

	type Status = 'idle' | 'copied' | 'failed';
	let status = $state<Status>('idle');
	let resetTimeout: ReturnType<typeof setTimeout> | undefined;

	function resetAfter(next: Status, ms: number) {
		status = next;
		clearTimeout(resetTimeout);
		resetTimeout = setTimeout(() => (status = 'idle'), ms);
	}

	async function copy() {
		try {
			await navigator.clipboard.writeText(`<!-- URL: ${url} -->\n\n${raw}`);
			resetAfter('copied', 2000);
		} catch (error) {
			// navigator.clipboard.writeText rejects in an insecure context or without
			// permission — without this catch the promise fails silently and the button
			// never gives the user any error feedback.
			console.error('Failed to copy page as Markdown', error);
			resetAfter('failed', 3000);
		}
	}

	// Same pill as the brand copy button (AcerolaCode): surface at rest, the state in its soft tone.
	const views: Record<Status, { icon: typeof ClipboardCopyIcon; text: string; class: string }> =
		$derived({
			idle: {
				icon: ClipboardCopyIcon,
				text: label,
				class: 'bg-surface text-muted-foreground hover:text-foreground'
			},
			copied: { icon: CheckIcon, text: copiedLabel, class: 'bg-success-soft text-success' },
			failed: {
				icon: XIcon,
				text: failedLabel,
				class: 'bg-destructive-soft text-destructive'
			}
		});
	const view = $derived(views[status]);
	const Icon = $derived(view.icon);
</script>

<button
	type="button"
	class="flex shrink-0 items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-black transition-all duration-200 ease-brand active:scale-95 {view.class}"
	onclick={copy}
	aria-label={view.text}
	title={view.text}
>
	<Icon size={13} />
	<span>{view.text}</span>
</button>
