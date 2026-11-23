<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import * as Command from '#lib/components/ui/command/index.ts';
	import AcerolaCommand from './acerola-command.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaCommand',
		component: AcerolaCommand,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Container for command-palette style interfaces. The wrapper takes `children` as a snippet and organizes state/events in `state` and `events`, suited for local search, quick navigation, keyboard-driven menus and filter-assisted selection.'
				}
			},
			controls: {
				include: ['state']
			}
		},
		argTypes: {
			state: {
				description: 'Current text used by the command search input.',
				control: 'object'
			},
			events: {
				description: 'Callback fired when the search text changes.',
				control: 'object'
			},
			children: {
				description:
					'Snippet with the command internal composition, such as input, list and items.',
				control: false
			}
		}
	});
</script>

{#snippet template(args: Record<string, unknown>)}
	<div class="w-full max-w-md">
		<AcerolaCommand {...args}>
			{#snippet children()}
				<Command.Input placeholder="Pesquisar..." />
				<Command.List>
					<Command.Item value="item1">Item 1</Command.Item>
					<Command.Item value="item2">Item 2</Command.Item>
					<Command.Item value="item3">Item 3</Command.Item>
				</Command.List>
			{/snippet}
		</AcerolaCommand>
	</div>
{/snippet}

<Story
	name="Default"
	args={{ state: { value: '' } }}
	parameters={{
		docs: {
			description: {
				story: 'Base example of a command palette with an empty search field and a short item list. A starting point for local search and internal navigation scenarios.'
			}
		}
	}}
	{template}
/>

<Story
	name="With Search Value"
	args={{ state: { value: 'item' } }}
	parameters={{
		docs: {
			description: {
				story: 'Example with a prefilled initial value. Useful to demonstrate controlled state, filter restoration and screens where the search arrives preconfigured from a previous context.'
			}
		}
	}}
	{template}
/>
