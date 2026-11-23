<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaCopyMarkdownButton from './acerola-copy-markdown-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaCopyMarkdownButton',
		component: AcerolaCopyMarkdownButton,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Button with an icon and visible label that copies a documentation page’s Markdown source (plus a comment with the URL) to the clipboard, to paste into an AI and ask for a summary. Shows success or failure feedback for a few seconds.'
				}
			}
		},
		argTypes: {
			raw: {
				description: 'Full Markdown source of the page (with frontmatter).',
				control: 'text'
			},
			url: {
				description:
					'Canonical URL of the page, included as a comment before the Markdown.',
				control: 'text'
			},
			label: { description: 'Accessible button text before copying.', control: 'text' },
			copiedLabel: {
				description: 'Accessible button text right after copying.',
				control: 'text'
			},
			failedLabel: {
				description: 'Accessible button text when copying fails.',
				control: 'text'
			}
		}
	});
</script>

<Story
	name="Default"
	args={{
		raw: '---\ntitle: Primeiros passos\nsection: Primeiros passos\n---\n\n# Primeiros passos\n\nConteúdo de exemplo.',
		url: 'https://docs.reader.acerola.app/docs/getting-started'
	}}
	parameters={{
		docs: {
			description: {
				story: 'Default state, ready to copy the current page’s Markdown.'
			}
		}
	}}
/>

<Story
	name="CustomLabels"
	args={{
		raw: '---\ntitle: Contribuindo\n---\n\n# Contribuindo',
		url: 'https://docs.reader.acerola.app/docs/contributing-overview',
		label: 'Copiar como Markdown',
		copiedLabel: 'Copiado'
	}}
	parameters={{
		docs: {
			description: {
				story: 'Custom accessible labels (for example, for use outside the default i18n context).'
			}
		}
	}}
/>

<Story
	name="CustomFailedLabel"
	args={{
		raw: '# Doc',
		url: 'https://docs.reader.acerola.app/docs/doc',
		failedLabel: 'Falha ao copiar'
	}}
	parameters={{
		docs: {
			description: {
				story: 'Custom failure label, shown when the clipboard rejects the copy (e.g. insecure context or permission denied).'
			}
		}
	}}
/>
