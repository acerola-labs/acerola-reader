<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { useOnboarding } from '#lib/hooks/onboarding/use-onboarding.svelte.ts';
	import { useLibraryScanner } from '#lib/hooks/store/use-comic-scanner.svelte.ts';
	import { DIRECTORY_SCAN_COMMANDS } from '#lib/contracts/library/library.commands.ts';
	import { useSelectFolder } from '#lib/hooks/store/use-select-folder.svelte.ts';
	import StepWelcome from './steps/acerola-step-welcome.svelte';
	import StepLanguage from './steps/acerola-step-language.svelte';
	import StepFormats from './steps/acerola-step-formats.svelte';
	import StepSettings from './steps/acerola-step-settings.svelte';
	import StepComplete from './steps/acerola-step-complete.svelte';
	import { goto } from '$app/navigation';
	import { fly } from 'svelte/transition';

	const onboarding = useOnboarding();
	const folder = useSelectFolder();

	const scanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.incrementalScan,
		() => folder.folderPath
	);

	const STEPS = [
		{ label: m['onboarding.step.welcome'] },
		{ label: m['onboarding.step.language'] },
		{ label: m['onboarding.step.formats'] },
		{ label: m['onboarding.step.settings'] },
		{ label: m['onboarding.step.complete'] }
	];

	let direction = $state(1);
	let previousStep = $state(0);

	$effect(() => {
		if (onboarding.currentStep !== previousStep) {
			direction = onboarding.currentStep > previousStep ? 1 : -1;
			previousStep = onboarding.currentStep;
		}
	});

	function handleNext() {
		direction = 1;
		onboarding.nextStep();
	}

	function handlePrev() {
		direction = -1;
		onboarding.prevStep();
	}

	async function handleComplete() {
		await onboarding.complete();
		if (folder.folderPath) {
			scanner.start();
		}
		await goto('/home');
	}

	let currentStepIndex = $derived(onboarding.currentStep);
	let progressPercent = $derived((currentStepIndex / (STEPS.length - 1)) * 100);
</script>

<div class="relative flex h-full w-full flex-col overflow-hidden bg-background">
	<!-- Stepper: solid header, steps as pills (current bg-primary, done bg-primary-soft). -->
	<div class="relative z-10 bg-background px-6 py-4">
		<div class="mx-auto flex max-w-4xl flex-col gap-3">
			<div class="flex items-center justify-between">
				<span class="text-xs font-semibold text-muted-foreground">
					{m['onboarding.setupTitle']()}
				</span>
				<span class="text-xs font-black text-muted-foreground tabular-nums">
					{currentStepIndex + 1} / {STEPS.length} ({Math.round(progressPercent)}%)
				</span>
			</div>

			<div class="flex w-full items-center gap-1 rounded-full bg-surface p-1">
				{#each STEPS as step, it (it)}
					{@const isCompleted = it < currentStepIndex}
					{@const isCurrent = it === currentStepIndex}
					<div
						class={[
							'flex h-9 flex-1 items-center justify-center gap-2 rounded-full px-3 text-xs font-bold transition-colors duration-300 ease-brand',
							isCurrent
								? 'bg-primary text-primary-foreground'
								: isCompleted
									? 'bg-primary-soft text-primary'
									: 'text-muted-foreground'
						]}
					>
						<span class="font-black tabular-nums">{it + 1}</span>
						<span class="truncate max-sm:hidden">{step.label()}</span>
					</div>
				{/each}
			</div>

			<div class="h-1 w-full overflow-hidden rounded-full bg-surface">
				<div
					class="h-full rounded-full bg-primary transition-[width] duration-500 ease-brand"
					style="width: {progressPercent}%;"
				></div>
			</div>
		</div>
	</div>

	<!-- Step Content Container with dynamic fly animation -->
	<div class="relative flex-1 overflow-hidden">
		{#key currentStepIndex}
			<div
				class="absolute inset-0 overflow-y-auto"
				in:fly={{ duration: 300, x: direction * 100 }}
			>
				{#if currentStepIndex === 0}
					<StepWelcome onNext={handleNext} />
				{:else if currentStepIndex === 1}
					<StepLanguage onNext={handleNext} onPrev={handlePrev} />
				{:else if currentStepIndex === 2}
					<StepFormats onNext={handleNext} onPrev={handlePrev} />
				{:else if currentStepIndex === 3}
					<StepSettings onNext={handleNext} onPrev={handlePrev} />
				{:else if currentStepIndex === 4}
					<StepComplete onPrev={handlePrev} onComplete={handleComplete} />
				{/if}
			</div>
		{/key}
	</div>
</div>
