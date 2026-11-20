<script lang="ts" module>
	export type AcerolaAlertDialogProps = {
		state?: {
			open: boolean;
		};
		data?: {
			title?: string;
			description?: string;
			cancelText?: string;
			actionText?: string;
		};
		events?: {
			onOpenChange?: (open: boolean) => void;
			onAction?: () => void;
			onCancel?: () => void;
		};
		ui?: {
			variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
		};
	};
</script>

<script lang="ts">
	import * as AlertDialog from '#lib/components/ui/alert-dialog/index.ts';
	import type { Snippet } from 'svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.ts';
	import { cn } from '#lib/utils/cn.util.ts';

	let {
		state = $bindable({ open: false }),
		data,
		events,
		ui,
		children
	}: AcerolaAlertDialogProps & {
		children?: Snippet;
	} = $props();
</script>

<AlertDialog.Root bind:open={state.open} onOpenChange={events?.onOpenChange}>
	<AlertDialog.Trigger>
		{#if children}
			{@render children()}
		{/if}
	</AlertDialog.Trigger>
	<AlertDialog.Content class="font-dm-sans rounded-inner">
		<AlertDialog.Header>
			<AlertDialog.Title class="text-xl font-bold tracking-tight"
				>{data?.title}</AlertDialog.Title
			>
			<AlertDialog.Description class="opacity-80">
				{data?.description}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer class="mt-6">
			<AlertDialog.Cancel class="rounded-inner" onclick={events?.onCancel}>
				{data?.cancelText ?? 'Cancel'}
			</AlertDialog.Cancel>
			<AlertDialog.Action
				class={cn('rounded-inner', buttonVariants({ variant: ui?.variant ?? 'default' }))}
				onclick={events?.onAction}
			>
				{data?.actionText ?? 'Continue'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
