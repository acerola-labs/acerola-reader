<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { toast } from 'svelte-sonner';
	import AcerolaSonner from './acerola-sonner.svelte';
	import { toastAsync } from '#lib/utils/toast-async.util.ts';

	function fakeInstantAction(): Promise<void> {
		// Resolves immediately on purpose — the case `toastAsync` guards against (`minDurationMs`),
		// so there's always time for the loading -> check animation to be noticed.
		return Promise.resolve();
	}

	function fakeFailingAction(): Promise<void> {
		return Promise.reject(new Error('Simulated failure'));
	}

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaSonner',
		component: AcerolaSonner,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Wrapper of the application notification system based on Sonner. This component must be mounted once in the tree to allow showing global success, error, warning and info toasts fired from anywhere in the interface.'
				}
			}
		}
	});
</script>

{#snippet template()}
	<div class="flex flex-col items-start gap-4">
		<AcerolaSonner />
		<button
			type="button"
			class="rounded-inner bg-primary px-4 py-2 text-primary-foreground"
			onclick={() => toast.success('Toast disparado!')}
		>
			Disparar Toast
		</button>
		<button
			type="button"
			class="rounded-inner bg-secondary px-4 py-2 text-secondary-foreground"
			onclick={() =>
				toastAsync(fakeInstantAction, {
					loading: 'Processando...',
					success: 'Concluído!',
					error: 'Falhou.'
				})}
		>
			Simular operação (loading -&gt; sucesso)
		</button>
		<button
			type="button"
			class="rounded-full bg-destructive px-4 py-2 font-black text-destructive-foreground"
			onclick={() =>
				toastAsync(fakeFailingAction, {
					loading: 'Processando...',
					success: 'Concluído!',
					error: (err) => `Falhou: ${(err as Error).message}`
				}).catch(() => {})}
		>
			Simular operação (loading -&gt; erro)
		</button>
	</div>
{/snippet}

<Story name="Default" {template} />

<Story
	name="Docs"
	parameters={{
		docs: {
			description: {
				story:
					'This example mounts the toaster and exposes a button to fire a success toast. In real use, `AcerolaSonner` must stay mounted in a main layout or shell, while toasts can be called with `toast.success(...)`, `toast.error(...)` and similar variants on action events, form feedback and async operations.'
			}
		}
	}}
	{template}
/>
