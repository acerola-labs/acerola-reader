<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepWelcome from './acerola-step-welcome.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepWelcome,
		title: 'Pages/Onboarding/AcerolaStepWelcome',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'First onboarding step: welcome to Acerola.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepWelcome onNext={() => {}} />
	</div>
</Story>
