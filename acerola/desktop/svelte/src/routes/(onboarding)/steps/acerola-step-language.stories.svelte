<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepLanguage from './acerola-step-language.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepLanguage,
		title: 'Pages/Onboarding/AcerolaStepLanguage',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Onboarding step to select the application preferred language.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepLanguage onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
