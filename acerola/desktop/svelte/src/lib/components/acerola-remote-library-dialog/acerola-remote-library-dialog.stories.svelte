<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaRemoteLibraryDialog from './acerola-remote-library-dialog.svelte';
	import type { ComicSummary } from '#lib/contracts/network/network.payloads.ts';

	const comics: ComicSummary[] = [
		{ comicName: 'One Piece', chapterCount: 1090, coverVersion: 1 },
		{ comicName: 'Berserk', chapterCount: 364, coverVersion: 2 },
		{ comicName: 'Vagabond', chapterCount: 327, coverVersion: 1 }
	];

	const { Story } = defineMeta({
		component: AcerolaRemoteLibraryDialog,
		title: 'Composites/AcerolaRemoteLibraryDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Dialog listing the comic library of a remote peer, with loading, error, empty and syncing states.'
				}
			}
		}
	});
</script>

<script lang="ts">
	// AcerolaRemoteLibraryDialog is fully controlled (state.open + events.onOpenChange) and has
	// no trigger of its own — each story needs its own local state and a real button to open
	// it, otherwise the Docs tab shows all 5 variants with the dialog open at the same time.
	let openDefault = $state(false);
	let openLoading = $state(false);
	let openError = $state(false);
	let openEmpty = $state(false);
	let openSyncing = $state(false);
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openDefault = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openDefault }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics,
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openDefault = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Loading" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openLoading = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openLoading }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: true,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openLoading = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Error" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openError = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openError }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: false,
					errorMessage: 'Não foi possível consultar a biblioteca remota.',
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openError = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openEmpty = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openEmpty }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics: [],
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: () => false
				}}
				events={{ onOpenChange: (isOpen) => (openEmpty = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Syncing" asChild>
	{#snippet children()}
		<div class="min-h-[500px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openSyncing = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Dialog
			</button>
			<AcerolaRemoteLibraryDialog
				state={{ open: openSyncing }}
				data={{
					peerLabel: 'Notebook do Trabalho',
					comics,
					isLoading: false,
					coverPathFor: () => undefined,
					isSyncing: (name) => name === 'One Piece'
				}}
				events={{ onOpenChange: (isOpen) => (openSyncing = isOpen), onSelectComic: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
