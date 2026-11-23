<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import type { SidebarGroup } from '#lib/content/docs.ts';
	import AcerolaMobileNav from './acerola-mobile-nav.svelte';

	const groups: SidebarGroup[] = [
		{
			section: 'Primeiros passos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'getting-started',
					component: {} as never,
					raw: '',
					frontmatter: {
						title: 'Primeiros passos',
						section: 'Primeiros passos',
						order: 1
					}
				}
			]
		},
		{
			section: 'Conceitos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'architecture',
					component: {} as never,
					raw: '',
					frontmatter: { title: 'Arquitetura', section: 'Conceitos', order: 1 }
				}
			]
		}
	];

	const { Story } = defineMeta({
		title: 'Layout/AcerolaMobileNav',
		component: AcerolaMobileNav,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Full-screen navigation menu (Sheet) used on small screens, with the same `Sidebar` as the desktop side navigation and the `NavControls` in the footer.'
				}
			}
		},
		argTypes: {
			open: { description: 'Controls the menu visibility.', control: 'boolean' },
			groups: { description: 'Sections and documents to display.', control: false },
			activeSlug: { description: 'Slug of the currently open document.', control: 'text' }
		}
	});
</script>

<script lang="ts">
	// `open: true` as the initial arg makes the Sheet start already open — bits-ui's "presence"
	// logic that reveals the content only runs on a real closed→open transition, so the panel
	// never shows up (only the full-screen overlay, blurring the whole Docs tab behind it).
	// Closed by default; a real button in the story triggers the opening transition.
	let open = $state(false);
</script>

<Story
	name="Default"
	args={{ groups, activeSlug: 'architecture' }}
	globals={{ viewport: 'mobile1' }}
	parameters={{
		docs: {
			description: {
				story: 'Closed by default, in a fixed mobile viewport (the component is `md:hidden` — without this the story only shows up by zooming out in the browser). Click the button to open it and see the full navigation and controls.'
			}
		}
	}}
>
	{#snippet template(args: { groups: SidebarGroup[]; activeSlug: string })}
		<div class="flex min-h-[200px] w-full items-center justify-center">
			<button
				type="button"
				onclick={() => (open = true)}
				class="hover:bg-surface-elevated rounded-inner border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground"
			>
				Abrir menu
			</button>
		</div>
		<AcerolaMobileNav {...args} bind:open />
	{/snippet}
</Story>
