import { defineConfig, devices } from '@playwright/test';

// Visual audit only (see audit.spec.ts): one worker, no retries, failures don't stop the walk.
export default defineConfig({
	testDir: '.',
	timeout: 60_000,
	workers: 1,
	reporter: 'list',
	use: {
		baseURL: 'http://127.0.0.1:1420',
		viewport: { width: 1280, height: 1100 }
	},
	webServer: {
		command: 'npm run dev',
		url: 'http://127.0.0.1:1420',
		reuseExistingServer: true,
		timeout: 180_000
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 1100 } }
		}
	]
});
