<script module lang="ts">
	import type { ReaderMode } from '../hooks/use-reader-zoom.svelte';

	export type ReaderFooterProps = {
		data: {
			pageProgressPercent: number;
			pageProgressWidth: string;
			chapterProgressLabel: string;
			modeLabel: string;
			zoomStatusLabel: string;
			chaptersRemainingLabel: string;
			hasNextChapter?: boolean;
			hasPreviousChapter?: boolean;
		};
		state: {
			readingMode: ReaderMode;
		};
		events: {
			onReadingModeChange: (mode: ReaderMode) => void;
			onNextChapter?: () => void;
			onPreviousChapter?: () => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import ReaderModeToggle from './acerola-reader-mode-toggle.svelte';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';

	let { data, events, state }: ReaderFooterProps = $props();
</script>

<footer class="relative z-20 shrink-0 bg-mantle px-4 py-3">
	<div class="flex flex-col gap-3">
		<div
			class="flex items-center justify-between gap-3 text-xs font-semibold text-muted-foreground tabular-nums"
		>
			<span class="shrink-0"
				>{m['pages.reader.progress.readPercent']({
					percent: data.pageProgressPercent
				})}</span
			>
			<span class="hidden min-w-0 truncate md:inline"
				>{data.modeLabel} - {data.zoomStatusLabel}</span
			>
			<span class="min-w-0 truncate text-right">{data.chaptersRemainingLabel}</span>
		</div>

		<div
			role="progressbar"
			aria-valuemin="0"
			aria-valuemax="100"
			aria-valuenow={data.pageProgressPercent}
			class="h-2 overflow-hidden rounded-full bg-surface"
			title={data.chapterProgressLabel}
		>
			<div
				class="h-full rounded-full bg-primary transition-[width] duration-300 ease-out"
				style:width={data.pageProgressWidth}
			></div>
		</div>

		<div class="flex items-center justify-between gap-4">
			<div class="flex items-center gap-2">
				{#if data.hasPreviousChapter && events.onPreviousChapter}
					<button
						class="flex h-11 items-center gap-2 rounded-full bg-surface px-4 text-sm font-black text-foreground transition-[filter,transform] duration-200 ease-brand hover:brightness-110 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none active:scale-[0.98]"
						onclick={events.onPreviousChapter}
					>
						<ChevronLeft size={16} />
						<span class="hidden md:inline"
							>{m['pages.reader.actions.chapter.previous']()}</span
						>
					</button>
				{/if}

				<ReaderModeToggle
					state={{ value: state.readingMode }}
					events={{ onValueChange: events.onReadingModeChange }}
					ui={{ variant: 'mobile' }}
				/>
			</div>

			{#if data.hasNextChapter && events.onNextChapter}
				<button
					class="flex h-11 items-center gap-2 rounded-full bg-surface px-4 text-sm font-black text-foreground transition-[filter,transform] duration-200 ease-brand hover:brightness-110 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none active:scale-[0.98]"
					onclick={events.onNextChapter}
				>
					<span class="hidden md:inline">{m['pages.reader.actions.chapter.next']()}</span>
					<ChevronRight size={16} />
				</button>
			{/if}
		</div>
	</div>
</footer>
