import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './tests/e2e',
	timeout: 10_000,
	workers: 1,
	// NOTE: the first spec to hit a new route pays for Vite's on-demand compilation
	// (cold dev server) and may exceed the `expect` timeout on that first attempt only —
	// retrying on CI fixes it without masking real failures (reported as "flaky", not silent).
	retries: process.env.CI ? 2 : 0,
	expect: {
		timeout: 5_000
	},
	use: {
		baseURL: 'http://127.0.0.1:1420',
		trace: 'retain-on-failure'
	},
	webServer: {
		command: 'npm run dev',
		url: 'http://127.0.0.1:1420',
		reuseExistingServer: !process.env.CI,
		timeout: 120_000
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	]
});
