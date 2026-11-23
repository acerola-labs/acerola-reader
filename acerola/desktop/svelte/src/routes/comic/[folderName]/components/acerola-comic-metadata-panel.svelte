<script lang="ts" module>
	export type ComicMetadataPanelProps = {
		data: {
			title: string;
			author: string;
			status: string;
			source: string;
			chaptersCount: number;
			description: string;
			cover: string | null;
			bookmarkColor?: number | null;
			bookmarkName?: string | null;
		};
		state?: {
			isResuming?: boolean;
			isLoading?: boolean;
		};
		events: {
			onBack: () => void;
			onReadNow?: () => void;
		};
	};
</script>

<script lang="ts">
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaCardImage from '#lib/components/acerola-card/acerola-card-image.svelte';
	import AcerolaBookmarkRibbon from '#lib/components/acerola-bookmark-ribbon/acerola-bookmark-ribbon.svelte';
	import PlaceholderManga from '#lib/assets/placeholder/placeholder_manga.svg?component';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import Play from '@lucide/svelte/icons/play';
	import { m } from '#lib/paraglide/messages.js';

	let { data, events, state }: ComicMetadataPanelProps = $props();
</script>

<div class="relative z-10 hidden h-full w-100 shrink-0 flex-col bg-mantle select-none lg:flex">
	<div class="flex h-full flex-col p-10">
		<AcerolaButtonIcon
			events={{ onClick: events.onBack }}
			ui={{ tone: 'accent', class: 'group mb-10' }}
		>
			<ArrowLeft size={24} />
		</AcerolaButtonIcon>

		<div
			class="scrollbar-hide flex flex-1 flex-col items-center space-y-6 overflow-y-auto pt-3 text-center"
		>
			<AcerolaCardImage
				data={{
					title: data.title,
					cover: data.cover
				}}
				ui={{
					class: 'w-64 shrink-0 [&_.mt-3]:hidden'
				}}
			>
				{#snippet floatingBadge()}
					{#if data.bookmarkColor != null}
						<AcerolaBookmarkRibbon
							color={data.bookmarkColor}
							name={data.bookmarkName ?? undefined}
							class="-top-1.5 left-5 h-10 w-6"
						/>
					{/if}
				{/snippet}

				{#snippet placeholder()}
					<div class="h-full w-full bg-surface">
						<PlaceholderManga class="h-full w-full" />
					</div>
				{/snippet}
			</AcerolaCardImage>

			<div class="w-full space-y-3 px-4">
				<div class="space-y-1">
					<h1 class="line-clamp-2 text-4xl leading-tight font-black tracking-tighter">
						{data.title}
					</h1>
					<p class="text-lg font-bold text-primary">{data.author}</p>
				</div>

				<div class="flex flex-wrap justify-center gap-2">
					<span
						class="rounded-full bg-surface px-4 py-1.5 text-[11px] font-bold text-foreground"
						>{data.status}</span
					>
					<span
						class="rounded-full bg-surface px-4 py-1.5 text-[11px] font-bold text-foreground"
						>{data.source}</span
					>
				</div>
			</div>

			<div class="w-full space-y-3 rounded-tile bg-surface p-5 text-left">
				<div class="flex items-center justify-between">
					<h3 class="text-sm font-black tracking-tight text-foreground">
						{m['pages.comic.metadata.synopsis']()}
					</h3>
					<span class="text-xs font-semibold text-muted-foreground tabular-nums"
						>{m['pages.comic.metadata.chapters.count']({
							count: data.chaptersCount
						})}</span
					>
				</div>
				<p class="line-clamp-6 text-sm leading-relaxed text-muted-foreground">
					{data.description}
				</p>
			</div>

			<div class="mt-auto w-full space-y-3 pt-4">
				<AcerolaButton
					events={{ onClick: events.onReadNow }}
					ui={{
						size: 'lg',
						class: 'flex w-full items-center justify-center gap-3',
						disabled: state?.isLoading
					}}
				>
					<Play size={20} fill="currentColor" />
					{state?.isResuming
						? m['pages.history.resume']()
						: m['pages.comic.metadata.readNow']()}
				</AcerolaButton>
			</div>
		</div>
	</div>
</div>
