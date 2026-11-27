import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import svg from '@poppanator/sveltekit-svg';
import { defineConfig } from 'vite';
import { paraglide } from './paraglide.config.ts';

export default defineConfig({
	server: {
		host: true
	},
	optimizeDeps: {
		include: ['gsap', 'gsap/ScrollTrigger']
	},
	plugins: [
		tailwindcss(),
		sveltekit(),
		svg({
			includePaths: ['./src/lib/assets/'],
			svgoOptions: {
				multipass: true,
				plugins: ['preset-default']
			}
		}),
		paraglideVitePlugin(paraglide)
	]
});
