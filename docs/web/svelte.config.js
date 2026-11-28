import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', ...mdsvexConfig.extensions],
	preprocess: [vitePreprocess(), mdsvex(mdsvexConfig)],
	compilerOptions: {
		// rehype-pretty-code puts `tabindex="0"` on <pre> so long code blocks are
		// keyboard-scrollable — a deliberate a11y pattern that Svelte's static
		// checker can't tell apart from a genuine misuse, so it warns anyway.
		warningFilter: (warning) => warning.code !== 'a11y_no_noninteractive_tabindex'
	},
	kit: {
		adapter: adapter(),
		alias: {
			$theme: 'src/theme',
			$lib: 'src/lib',
			$routes: 'src/routes'
		},
		prerender: {
			// Each locale under its prefix (paraglide.config.ts). The locale toggle switches via
			// `setLocale()`, not an `<a href>`, so each locale's root is listed, plus the files the
			// crawler never finds through a link; from a root it follows the localized links.
			entries: [
				'*',
				...['pt-br', 'en'].flatMap((locale) => [
					`/${locale}`,
					`/${locale}/sitemap.xml`,
					`/${locale}/llms.txt`
				])
			]
		}
	}
};

export default config;
