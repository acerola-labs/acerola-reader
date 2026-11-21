<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaPopover from './acerola-popover.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaPopover',
		component: AcerolaPopover,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Wrapper for showing floating content anchored to a trigger. The component takes two required snippets, `trigger` and `content`, and controls opening internally with `bind:open`, allowing controlled or uncontrolled use as needed.'
				}
			},
			controls: {
				include: ['state']
			}
		},
		argTypes: {
			state: {
				description: 'Sets whether the popover starts open or closed.',
				control: 'object'
			},
			events: { description: 'Popover open callbacks.', control: 'object' },
			ui: { description: 'Content visual configuration.', control: 'object' },
			trigger: {
				description: 'Snippet that renders the element used as the popover trigger.',
				control: false
			},
			content: {
				description: 'Snippet with the content shown inside the floating layer.',
				control: false
			}
		}
	});
</script>

{#snippet template(args: Record<string, unknown>)}
	<AcerolaPopover {...args}>
		{#snippet trigger()}
			<button
				type="button"
				class="rounded-inner bg-primary px-4 py-2 text-primary-foreground"
			>
				Abrir Popover
			</button>
		{/snippet}
		{#snippet content()}
			<div class="rounded-card bg-card p-4 text-card-foreground shadow-lift">
				Conteudo do Popover
			</div>
		{/snippet}
	</AcerolaPopover>
{/snippet}

<Story
	name="Default"
	args={{ state: { open: false } }}
	parameters={{
		docs: {
			description: {
				story: 'Initially closed. This is the default behavior for context menus, expanded hints and secondary actions revealed on demand.'
			}
		}
	}}
	{template}
/>

<Story
	name="Initially Open"
	args={{ state: { open: true } }}
	parameters={{
		docs: {
			description: {
				story: 'Initially open. Useful for visual inspection, layout validation and cases where the content must show immediately due to a business rule or onboarding.'
			}
		}
	}}
	{template}
/>
