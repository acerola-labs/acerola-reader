<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepComplete from './acerola-step-complete.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepComplete,
		title: 'Pages/Onboarding/AcerolaStepComplete',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Last onboarding step: completion confirmation.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepComplete onPrev={() => {}} onComplete={() => {}} />
	</div>
</Story>
