/** Copies text and remembers what was copied for a moment, so the button can say "copied". */
export function useCopy(resetMs = 1400) {
	let copied = $state<string | null>(null);
	let timer: ReturnType<typeof setTimeout> | undefined;

	async function copy(text: string) {
		try {
			await navigator.clipboard.writeText(text);
		} catch {
			return; // no clipboard permission: the text stays visible to copy by hand
		}
		copied = text;
		clearTimeout(timer);
		timer = setTimeout(() => (copied = null), resetMs);
	}

	return {
		get copied() {
			return copied;
		},
		copy
	};
}
