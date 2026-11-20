<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { DocEntry } from '#lib/content/docs.ts';
	import AcerolaPrevNextNav from './acerola-prev-next-nav.svelte';

	function doc(slug: string, title: string): DocEntry {
		return {
			locale: 'pt-br',
			slug,
			component: {} as never,
			raw: '',
			frontmatter: { title, section: 'Docs', order: 1 }
		};
	}

	const { Story } = defineMeta({
		title: 'Composites/AcerolaPrevNextNav',
		component: AcerolaPrevNextNav,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Pair of sequential navigation links at the end of a documentation page. Each side is omitted when there is no neighbor (start or end of the docs list).'
				}
			}
		},
		argTypes: {
			prev: { description: 'Previous document, or null if this is the first.', control: false },
			next: { description: 'Next document, or null if this is the last.', control: false }
		}
	});
</script>

<Story
	name="Default"
	args={{
		prev: doc('getting-started', 'Primeiros passos'),
		next: doc('architecture', 'Arquitetura')
	}}
	parameters={{
		docs: { description: { story: 'Both neighbors present, the common case in the middle of the list.' } }
	}}
/>

<Story
	name="First Page"
	args={{ prev: null, next: doc('architecture', 'Arquitetura') }}
	parameters={{
		docs: { description: { story: 'First page of the list: no "Previous" link.' } }
	}}
/>

<Story
	name="Last Page"
	args={{ prev: doc('getting-started', 'Primeiros passos'), next: null }}
	parameters={{
		docs: { description: { story: 'Last page of the list: no "Next" link.' } }
	}}
/>
