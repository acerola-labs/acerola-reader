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
		'src-tauri',
		'tests/wdio',
		'svelte/src/lib/paraglide',
		'coverage',
		'test-results',
		'playwright-report',
		'storybook-static',
		'mutants.out',
		'mutants.out.old'
	],
	mutate: [
		'svelte/src/lib/hooks/**/*.svelte.ts',
		'svelte/src/lib/state/**/*.svelte.ts',
		'svelte/src/lib/services/**/*.ts',
		'svelte/src/lib/utils/**/*.ts',
		'!svelte/src/**/*.test.ts',
		'!svelte/src/**/*.browser.test.ts',
		'!svelte/src/**/*.stories.svelte',
		'!svelte/src/lib/paraglide/**',
		'!svelte/src/lib/components/ui/**'
	],
	reporters: ['html', 'clear-text', 'progress'],
	htmlReporter: {
		fileName: 'mutants.out/mutation-report.html'
	},
	thresholds: {
		// Changed from a placeholder to the real measured baseline (49.03% on the first full
		// run) — high/low tuned to reflect the real per-file distribution instead of
		// arbitrary goals. Update: after killing the 168 real survivors mapped in a
		// dedicated work round, the score rose to 71.06%, with 0 survivors in the measured
		// files (hooks/state/services/utils with coverage) — the few remaining mutants are
		// truly equivalent, documented with `// Stryker disable next-line <Mutator>: <reason>`
		// right in each file's source (no threshold or blind suppression). All that is left
		// to get close to the Rust side's rigor is the "no coverage" measurement bug below.
		//
		// TODO (not fixed, only documented): the files below showed up mostly as
		// "no coverage" in the report (mutants.out/mutation-report.html) EVEN THOUGH they
		// have tests — i.e. Stryker's `coverageAnalysis: perTest` cannot link any test to
		// the mutants of these files, so their real mutation score is unknown, not
		// necessarily zero. Suspicion: hooks with module-level state (`$state` outside a
		// function) or called outside a mounted component confuse per-test tracking. Needs
		// dedicated investigation; it is not a Stryker configuration problem per se:
		// - use-comic-summary.svelte.ts (100 mutants, 0 coverage)
		// - use-comic-scanner.svelte.ts (56 mutants, 0 coverage)
		// - use-comic-chapters.svelte.ts (76 mutants, 0 coverage)
		// - use-select-folder.svelte.ts (33 mutants, 0 coverage)
		// - use-history.svelte.ts (57 mutants, 0 coverage)
		// - use-theme.svelte.ts (partial: 8 of 43 mutants without coverage)
		//
		// Solved: use-metadata-language.svelte.ts, use-reader-mode.svelte.ts and
		// use-relay-settings.svelte.ts reached a real 100% mutation score (43/43 killed,
		// 0 no-coverage) after getting tests — this backs the suspicion above: all three
		// keep `$state` only inside the exported function (new instance per call, no
		// module-level singleton), unlike the files still listed as "no coverage" here.
		high: 70,
		low: 50,
		break: 45
	}
};
