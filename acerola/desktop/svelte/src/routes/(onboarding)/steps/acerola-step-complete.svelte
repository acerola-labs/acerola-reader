<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';

	let { onPrev, onComplete } = $props<{ onPrev: () => void; onComplete: () => void }>();

	const checklist = $derived([
		m['onboarding.complete.checklistItem1'](),
		m['onboarding.complete.checklistItem2'](),
		m['onboarding.complete.checklistItem3']()
	]);
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="flex max-w-lg flex-col items-center space-y-8 text-center">
		<span
			class="flex size-24 items-center justify-center rounded-full bg-success-soft text-success"
		>
			<CheckCircleIcon size={48} />
		</span>

		<div class="space-y-2">
			<h1 class="text-4xl font-black tracking-tight text-foreground sm:text-5xl">
				{m['onboarding.complete.title']()}
			</h1>
			<p class="mx-auto max-w-md text-base leading-relaxed text-muted-foreground sm:text-lg">
				{m['onboarding.complete.desc']()}
			</p>
		</div>

		<div class="w-full space-y-3 rounded-card bg-card p-5 text-left shadow-lift">
			<div class="flex items-center gap-2 text-sm font-black tracking-tight text-foreground">
				<SparklesIcon size={16} class="text-success" />
				<span>{m['onboarding.complete.checklistTitle']()}</span>
			</div>

			<div class="space-y-2">
				{#each checklist as item (item)}
					<div
						class="flex items-center gap-3 rounded-tile bg-surface p-3 text-sm text-foreground"
					>
						<CheckCircle2Icon size={16} class="shrink-0 text-success" />
						<span>{item}</span>
					</div>
				{/each}
			</div>
		</div>

		<p
			class="w-full rounded-card bg-surface p-5 text-left text-sm leading-relaxed text-muted-foreground"
		>
			{m['onboarding.complete.note']()}
		</p>

		<div class="flex w-full items-center justify-between pt-2">
			<AcerolaButton
				ui={{ variant: 'secondary', class: 'px-6' }}
				events={{ onClick: onPrev }}
			>
				{m['onboarding.back']()}
			</AcerolaButton>
			<AcerolaButton
				ui={{ size: 'lg', class: 'group gap-2 px-8' }}
				events={{ onClick: onComplete }}
			>
				<span>{m['onboarding.complete.button']()}</span>
				<ArrowRightIcon
					size={18}
					class="transition-transform duration-200 ease-brand group-hover:translate-x-1"
				/>
			</AcerolaButton>
		</div>
	</div>
</div>
