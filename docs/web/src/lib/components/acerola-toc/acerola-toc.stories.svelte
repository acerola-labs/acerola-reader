<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaToc from './acerola-toc.svelte';

	const { Story } = defineMeta({
		title: 'Composites/AcerolaToc',
		component: AcerolaToc,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Table of contents generated from the `h2`/`h3` headings with an `id` found inside `containerSelector` (`.doc-content` by default). Highlights the currently visible heading using an `IntersectionObserver`. Renders nothing when no heading is found.'
				}
			}
		},
		argTypes: {
			containerSelector: {
				description: 'CSS selector of the container where headings are looked up.',
				control: 'text'
			}
		}
	});
</script>

{#snippet template(args: Record<string, unknown>)}
	<div class="flex gap-8">
		<div class="doc-content max-w-md">
			<h2 id="introducao">Introdução</h2>
			<p>Conteudo de exemplo para demonstrar a tabela de conteudos.</p>
			<h3 id="instalacao">Instalação</h3>
			<p>Mais conteudo de exemplo.</p>
			<h2 id="proximos-passos">Próximos passos</h2>
		</div>
		<AcerolaToc {...args} />
	</div>
{/snippet}

<Story
	name="Default"
	args={{}}
	parameters={{
		docs: {
			description: {
				story: 'Sample headings inside the default `.doc-content` container, rendered alongside.'
			}
		}
	}}
	{template}
/>
