<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import CheckIcon from '@lucide/svelte/icons/check';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import { useSelectFolder } from '#lib/hooks/store/use-select-folder.svelte.ts';

	import { onMount } from 'svelte';

	const folder = useSelectFolder();

	onMount(() => {
		folder.loadSavedPath();
	});

	$effect(() => {
		folder.loadSavedPath();
	});

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center overflow-y-auto p-6 sm:p-8">
	<div class="w-full max-w-xl space-y-8 text-center sm:text-left">
		<div class="flex flex-col items-center gap-3 sm:flex-row sm:items-center">
			<span
				class="flex size-14 shrink-0 items-center justify-center rounded-full bg-success-soft text-success"
			>
				<FolderIcon size={24} />
			</span>
			<div>
				<h1 class="text-3xl font-black tracking-tight text-foreground sm:text-4xl">
					{m['onboarding.settings.title']()}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground">
					{m['onboarding.settings.folder.desc']()}
				</p>
			</div>
		</div>

		<!-- Folder picker: solid block, primary tone once a folder is chosen. -->
		<div
			class={[
				'flex flex-col items-center rounded-card p-8 text-center transition-colors duration-300 ease-brand',
				folder.folderPath ? 'bg-primary-soft' : 'bg-surface'
			]}
		>
			<span
				class={[
					'mb-4 flex size-16 items-center justify-center rounded-full',
					folder.folderPath
						? 'bg-primary text-primary-foreground'
						: 'bg-card text-muted-foreground'
				]}
			>
				<FolderIcon size={24} />
			</span>

			<h2 class="mb-1 text-xl font-black tracking-tight text-foreground">
				{m['onboarding.settings.folder.title']()}
			</h2>

			<p class="mb-5 max-w-sm text-sm text-muted-foreground">
				{m['onboarding.settings.folder.desc']()}
			</p>

			{#if folder.folderPath}
				<div
					class="mb-5 flex max-w-full items-center gap-2 rounded-full bg-card px-4 py-1.5 text-xs font-black text-primary"
				>
					<CheckIcon size={16} strokeWidth={2.5} />
					<span>{m['onboarding.settings.folder.selected']()}</span>
				</div>
			{/if}

			<AcerolaButton
				ui={{
					variant: folder.folderPath ? 'secondary' : 'default',
					class: 'w-full max-w-md truncate px-6'
				}}
				events={{ onClick: folder.selectFolder }}
			>
				{folder.folderPath || m['onboarding.settings.folder.select']()}
			</AcerolaButton>
		</div>

		<div class="flex items-center justify-between pt-2">
			<AcerolaButton
				ui={{ variant: 'secondary', class: 'px-6' }}
				events={{ onClick: onPrev }}
			>
				{m['onboarding.back']()}
			</AcerolaButton>
			<AcerolaButton
				ui={{ class: 'px-8', disabled: !folder.folderPath }}
				events={{ onClick: onNext }}
			>
				{m['onboarding.next']()}
			</AcerolaButton>
		</div>
	</div>
</div>
