<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { Button } from '#lib/components/ui/button/index.ts';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';

	let { onPrev, onComplete } = $props<{ onPrev: () => void; onComplete: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="flex max-w-lg flex-col items-center space-y-8 text-center">
		<!-- Celebration Glowing Icon -->
		<div class="relative flex items-center justify-center">
			<div
				class="relative flex size-24 items-center justify-center rounded-tile bg-success-soft text-success shadow-lift"
			>
				<CheckCircleIcon size={48} />
			</div>
		</div>

		<!-- Title & Subtitle -->
		<div class="space-y-2">
			<h1 class="text-4xl font-black tracking-tight text-foreground sm:text-5xl">
				{m['onboarding.complete.title']()}
			</h1>
			<p class="mx-auto max-w-md leading-relaxed text-base text-muted-foreground sm:text-lg">
				{m['onboarding.complete.desc']()}
			</p>
		</div>

		<!-- Checklist Card (VibePrompts Onboarding Checklist pattern) -->
		<div
			class="w-full space-y-3 rounded-tile border border-border/50 bg-card/60 p-5 text-left backdrop-blur-md"
		>
			<div
				class="flex items-center gap-2 border-b border-border/30 pb-3 text-xs font-semibold text-foreground"
			>
				<SparklesIcon size={14} class="text-success" />
				<span>{m['onboarding.complete.checklistTitle']()}</span>
			</div>

			<div class="space-y-2 text-xs">
				<div class="flex items-center gap-2.5 text-foreground/90">
					<CheckCircle2Icon size={15} class="shrink-0 text-success" />
					<span>{m['onboarding.complete.checklistItem1']()}</span>
				</div>
				<div class="flex items-center gap-2.5 text-foreground/90">
					<CheckCircle2Icon size={15} class="shrink-0 text-success" />
					<span>{m['onboarding.complete.checklistItem2']()}</span>
				</div>
				<div class="flex items-center gap-2.5 text-foreground/90">
					<CheckCircle2Icon size={15} class="shrink-0 text-success" />
					<span>{m['onboarding.complete.checklistItem3']()}</span>
				</div>
			</div>
		</div>

		<!-- Note Box -->
		<div
			class="w-full rounded-inner border border-border/40 bg-card/40 p-4 text-left backdrop-blur-sm"
		>
			<p class="text-xs leading-relaxed text-muted-foreground">
				{m['onboarding.complete.note']()}
			</p>
		</div>

		<!-- Action Controls -->
		<div class="flex w-full items-center justify-between pt-2">
			<Button onclick={onPrev} variant="outline" class="rounded-inner px-6">
				{m['onboarding.back']()}
			</Button>
			<Button
				onclick={onComplete}
				size="lg"
				class="group relative inline-flex items-center gap-2 rounded-inner bg-primary px-8 py-6 font-semibold text-base text-primary-foreground shadow-lift transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
			>
				<span>{m['onboarding.complete.button']()}</span>
				<ArrowRightIcon
					size={18}
					class="transition-transform duration-200 group-hover:translate-x-1"
				/>
			</Button>
		</div>
	</div>
</div>
