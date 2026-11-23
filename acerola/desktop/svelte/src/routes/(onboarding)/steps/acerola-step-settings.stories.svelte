<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaStepSettings from './acerola-step-settings.svelte';

	const { Story } = defineMeta({
		component: AcerolaStepSettings,
		title: 'Pages/Onboarding/AcerolaStepSettings',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Onboarding step to choose the theme and the library folder. Uses the theme and folder selection hooks internally (Tauri calls fail silently outside the real app).'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	<div class="h-[720px] overflow-y-auto bg-surface">
		<AcerolaStepSettings onNext={() => {}} onPrev={() => {}} />
	</div>
</Story>
