<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaApkDownloadButton from './acerola-apk-download-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaApkDownloadButton',
		component: AcerolaApkDownloadButton,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Android APK download badge — same fixed dark chip as the Microsoft Store badge, with the official Android logo, to keep both platform buttons consistent side by side.'
				}
			}
		},
		argTypes: {
			label: {
				description: 'Small caption above "APK".',
				control: 'text'
			}
		}
	});
</script>

<Story name="Default" args={{}} />

<Story name="CustomLabel" args={{ label: 'Baixe o' }} />
