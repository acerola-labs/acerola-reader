import { defineConfig, devices } from '@playwright/test';

// IMPORTANT: the webServer below runs the production server (`node build`), NOT
// `npm run dev` or `npm run preview`. This is deliberate:
//   - `search-dialog.svelte` shows an "unavailable" placeholder whenever
//     `import.meta.env.DEV` is true, skipping Pagefind entirely in dev mode.
//   - The Pagefind index (`pagefind.js` + data files) is built into
//     `build/client/pagefind` (`pagefind --site build/prerendered --output-path
//     build/client/pagefind`, part of the `build` script) — the static directory the
//     Node server serves in production.
//   - `vite preview` serves the adapter-agnostic `.svelte-kit/output/client`, so it
//     404s on `/pagefind/*` even after a real build — only `node build` serves exactly
//     what production does, pagefind included.
// So a real `npm run build` must happen before `npx playwright test` for the search
// spec to have anything real to hit.
export default defineConfig({
	testDir: './tests/e2e',
	// The home page runs a WebGL background (`faulty-terminal.svelte`) and the search-dialog
	// loads the Pagefind WASM/index on demand on the first query — under headless Chromium's
	// virtual GPU in CI that easily exceeds 10s, and on busier runners it goes past 20s
	// (search-dialog.spec.ts gives those two assertions its own margin). The default
	// per-assertion timeout stays short (5s); only the whole-test ceiling is generous.
	timeout: 90_000,
	workers: 1,
	expect: {
		timeout: 5_000
	},
	retries: process.env.CI ? 2 : 0,
	use: {
		baseURL: 'http://127.0.0.1:5173',
		trace: 'retain-on-failure',
		// The home page uses a WebGL background effect (`faulty-terminal.svelte`, via `ogl`) that
		// emits "GPU stall"/driver warnings under headless Chromium's virtual GPU and can
		// occasionally crash the page (most visible on `page.reload()`) — forcing SwiftShader
		// (software rendering) gives a stable WebGL context instead of relying on the host's
		// GPU driver, which headless doesn't expose properly.
		launchOptions: {
			args: ['--use-gl=swiftshader', '--enable-webgl', '--ignore-gpu-blocklist']
		}
	},
	webServer: {
		command: 'node build',
		env: { HOST: '127.0.0.1', PORT: '5173' },
		url: 'http://127.0.0.1:5173',
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
