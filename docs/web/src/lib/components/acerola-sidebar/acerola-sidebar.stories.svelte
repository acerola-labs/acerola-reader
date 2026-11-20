<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { SidebarGroup } from '#lib/content/docs.ts';
	import AcerolaSidebar from './acerola-sidebar.svelte';

	function doc(slug: string, title: string, section: string): SidebarGroup['docs'][number] {
		return {
			locale: 'pt-br',
			slug,
			component: {} as never,
			raw: '',
			frontmatter: { title, section, order: 1 }
		};
	}

	const groups: SidebarGroup[] = [
		{
			section: 'Primeiros passos',
			docs: [doc('getting-started', 'Primeiros passos', 'Primeiros passos')]
		},
		{ section: 'Conceitos', docs: [doc('architecture', 'Arquitetura', 'Conceitos')] }
	];

	const { Story } = defineMeta({
		title: 'Layout/AcerolaSidebar',
		component: AcerolaSidebar,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Documentation side navigation, grouped by section. The link whose `slug` matches `activeSlug` gets visual emphasis.'
				}
			}
		},
		argTypes: {
			groups: { description: 'Sections and documents to display.', control: false },
			activeSlug: { description: 'Slug of the currently open document.', control: 'text' }
		}
	});
</script>

<Story
	name="Default"
	args={{ groups, activeSlug: 'architecture' }}
	parameters={{
		docs: {
			description: {
				story: 'Two sections with one document each; the "Arquitetura" item is active.'
			}
		}
	}}
/>
