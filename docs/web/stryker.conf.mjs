// @ts-check
/** @type {import('@stryker-mutator/api/core').PartialStrykerOptions} */
export default {
	packageManager: 'npm',
	testRunner: 'vitest',
	vitest: {
		configFile: 'vitest.stryker.config.ts'
	},
	coverageAnalysis: 'perTest',
	buildCommand: 'npx svelte-kit sync',
	ignorePatterns: [
		'src/lib/paraglide',
		'coverage',
		'storybook-static',
		'test-results',
		'playwright-report',
		'mutants.out',
		'mutants.out.old',
		'build',
		'.wrangler'
	],
	mutate: [
		'src/lib/hooks/**/*.svelte.ts',
		'src/lib/content/**/*.ts',
		'src/lib/utils.ts',
		'src/lib/constants/**/*.ts',
		'src/routes/**/+page.ts',
		'src/routes/**/+server.ts',
		'src/routes/+layout.ts',
		'!src/**/*.test.ts',
		'!src/**/*.stories.svelte',
		'!src/lib/paraglide/**',
		'!src/lib/components/ui/**'
	],
	reporters: ['html', 'clear-text', 'progress'],
	htmlReporter: {
		fileName: 'mutants.out/mutation-report.html'
	},
	thresholds: {
		// Switched from a placeholder to the real measured baseline (44.31% on the first full
		// run).
		//
		// TODO (not fixed, only documented): the files below showed up mostly as
		// "no coverage" in the report (mutants.out/mutation-report.html) EVEN THOUGH they
		// have tests — same pattern seen on desktop (see acerola/desktop/stryker.conf.mjs),
		// likely a limitation of Stryker's `coverageAnalysis: perTest` with modules called
		// outside a mounted component or with module-level state. The real mutation score of
		// these files is unknown, not necessarily zero — needs dedicated investigation:
		// - src/routes/sitemap.xml/+server.ts (25 mutants, 0 coverage, despite server.test.ts existing)
		// - src/routes/docs/[...slug]/+page.ts (6 mutants, 0 coverage, despite page.test.ts existing)
		// - src/lib/hooks/theme/use-theme.svelte.ts (43 mutants, 21 uncovered — partial)
		high: 60,
		low: 40,
		break: 38
	}
};
