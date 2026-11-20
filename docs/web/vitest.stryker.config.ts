import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

// Lean config used only by Stryker (stryker.conf.json) — mutation testing only makes sense
// for the `unit` project (pure logic in .ts/.svelte.ts). The `storybook` project in the main
// vitest.config.ts uses a real browser via Playwright, which would only make each mutant
// slower to run without adding meaningful mutation signal.
export default defineConfig({
	plugins: [sveltekit(), svelteTesting()],
	test: {
		globals: true,
		environment: 'jsdom',
		include: ['src/**/*.test.ts'],
		setupFiles: ['tests/setup.ts']
	}
});
