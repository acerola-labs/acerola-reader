/// <reference types="vite/client" />
import type { Preview } from '@storybook/sveltekit';
import { overwriteSetLocale } from '#lib/paraglide/runtime.js';
import './storybook.css';

// The real setLocale() navigates/reloads the page (see src/lib/paraglide/runtime.js) — inside
// the Storybook preview that navigates the iframe to a URL that doesn't exist in the static
// build, breaking the page (AcerolaNavControls/AcerolaTopNav). overwriteSetLocale is paraglide's
// official hook for this, so no component needs to know it's running in Storybook.
overwriteSetLocale(() => {});

export const globalTypes = {
	theme: {
		name: 'Theme',
		defaultValue: 'dark',
		toolbar: {
			icon: 'paintbrush',
			items: [
				{ value: 'dark', title: 'Dark' },
				{ value: 'light', title: 'Light' }
			],
			dynamicTitle: true
		}
	}
};

const preview: Preview = {
	decorators: [
		(Story, context) => {
			const theme = (context.globals.theme as string) ?? 'dark';

			document.documentElement.setAttribute('data-theme', theme);
			document.documentElement.classList.toggle('dark', theme === 'dark');

			return Story();
		}
	],
	parameters: {
		docs: {
			source: {
				type: 'dynamic',
				language: 'svelte'
			}
		},
		controls: {
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i
			}
		},
		a11y: {
			test: 'todo'
		}
	}
};

export default preview;
