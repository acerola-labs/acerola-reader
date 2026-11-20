import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { STORE_KEYS } from '#lib/constants/store-plugin.ts';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useTheme } from './use-theme.svelte';

const getCurrentWindowMock = vi.mocked(getCurrentWindow);

/**
 * Registers a `$app/environment` mock for the module's next dynamic import
 * (used with `vi.resetModules()` + `import()`), since the module runs its
 * theme-loading IIFE at the top level, at import time.
 */
function mockBrowserEnvironment(browserValue: boolean) {
	vi.doMock('$app/environment', () => ({
		browser: browserValue,
		dev: true,
		building: false,
		version: 'test'
	}));
}

/**
 * Registers a `@tauri-apps/plugin-store` mock whose `LazyStore` always
 * returns the SAME instance (instead of a new one on each `new LazyStore()`),
 * so we can hold a direct reference to the `get`/`set` spies even before
 * importing the fresh module.
 */
function mockPersistedStore(values: Record<string, unknown> = {}) {
	const store = {
		get: vi.fn((key: string) => Promise.resolve(values[key] ?? null)),
		set: vi.fn().mockResolvedValue(undefined)
	};

	vi.doMock('@tauri-apps/plugin-store', () => ({
		LazyStore: vi.fn().mockImplementation(function () {
			return store;
		})
	}));

	return store;
}

/** Waits a few microtasks — enough for the module's init IIFE to resolve. */
async function flushMicrotasks() {
	for (let i = 0; i < 4; i++) {
		await Promise.resolve();
	}
}

async function renderHook() {
	let hook: ReturnType<typeof useTheme> | undefined;

	render(HookHarness, {
		props: {
			create: useTheme,
			onReady: (value) => {
				hook = value as ReturnType<typeof useTheme>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

async function flushTheme() {
	await tick();
	await Promise.resolve();
	await tick();
}

describe('useTheme', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		document.documentElement.removeAttribute('data-theme');
		document.documentElement.classList.remove('dark');
	});

	it('applies light mode to the document', async () => {
		const hook = await renderHook();

		await hook.setMode('light');
		await flushTheme();

		expect(hook.mode).toBe('light');
		expect(hook.resolved).toBe('light');
		expect(document.documentElement.getAttribute('data-theme')).toBe('light');
		expect(document.documentElement.classList.contains('dark')).toBe(false);
	});

	it('resolves system mode from current window', async () => {
		const hook = await renderHook();

		await hook.setMode('system');
		await flushTheme();

		expect(hook.mode).toBe('system');
		expect(hook.resolved).toBe('light');
		expect(document.documentElement.classList.contains('dark')).toBe(false);
		// applyTheme() must really have run to set data-theme, not just the internal mode —
		// without this a mutant that removes the applyTheme call goes unnoticed.
		expect(document.documentElement.getAttribute('data-theme')).toBe('light');
	});

	it('applies dark mode directly (not via system), adding the dark class', async () => {
		const hook = await renderHook();

		await hook.setMode('dark');
		await flushTheme();

		expect(hook.resolved).toBe('dark');
		expect(document.documentElement.classList.contains('dark')).toBe(true);
		expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
	});

	it('does not query window.theme() for a non-system mode', async () => {
		const themeQuery = vi.fn().mockResolvedValue('light');
		getCurrentWindowMock.mockReturnValueOnce({
			theme: themeQuery,
			setTheme: vi.fn().mockResolvedValue(undefined),
			onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() })
		} as unknown as ReturnType<typeof getCurrentWindow>);

		const hook = await renderHook();
		// Pin data-theme to dark BEFORE the isolated setMode below — if setMode doesn't
		// really reapply the theme, data-theme stays stuck on this old value.
		await hook.setMode('dark');
		await flushTheme();
		expect(document.documentElement.getAttribute('data-theme')).toBe('dark');

		await hook.setMode('light');
		await flushTheme();

		expect(themeQuery).not.toHaveBeenCalled();
		expect(document.documentElement.getAttribute('data-theme')).toBe('light');
	});

	it('uses the resolved window theme as-is (not a truthy fallback) when it differs from the default', async () => {
		getCurrentWindowMock.mockReturnValueOnce({
			theme: vi.fn().mockResolvedValue('dark'),
			setTheme: vi.fn().mockResolvedValue(undefined),
			onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() })
		} as unknown as ReturnType<typeof getCurrentWindow>);

		const hook = await renderHook();
		await hook.setMode('system');
		await flushTheme();

		// window.theme() resolved 'dark' — it must reflect exactly that, not a fixed
		// 'light' fallback just because the value was truthy.
		expect(hook.resolved).toBe('dark');
	});

	describe('module init (fresh module instance per test)', () => {
		// `mode`/`resolved` and the saved-theme loading IIFE run at the module's top
		// level, at import time — that's why the tests below use `vi.resetModules()` +
		// dynamic `import()` to get a fresh module instance (with its own IIFE, its own
		// `store` and its mocked `browser`), instead of reusing the singleton already
		// loaded statically at the top of the file.

		it('initializes mode and resolved to dark synchronously, and keeps the defaults when nothing was persisted', async () => {
			mockBrowserEnvironment(true);
			const store = mockPersistedStore({});
			vi.resetModules();

			const { useTheme: freshUseTheme } = await import('./use-theme.svelte');
			const hook = freshUseTheme();

			// Synchronous `$state` values right after the import, before any init IIFE
			// microtask runs — proves the declared initial value is 'dark', not just that
			// it eventually converges to 'dark'.
			expect(hook.mode).toBe('dark');
			expect(hook.resolved).toBe('dark');

			await flushMicrotasks();

			// Proves the IIFE really called `store.get` with the right keys
			// (kills the mutant that swaps `Promise.all([...])` for `Promise.all([])`).
			expect(store.get).toHaveBeenCalledWith(STORE_KEYS.mode);

			// `savedMode` resolved `null` (nothing persisted) — must not overwrite the
			// default with `null`.
			expect(hook.mode).toBe('dark');
			expect(hook.resolved).toBe('dark');

			// `applyTheme(mode)` must really have run at the end of the IIFE.
			expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
			expect(document.documentElement.classList.contains('dark')).toBe(true);
		});

		it('overrides mode with the values persisted in the store on module init', async () => {
			mockBrowserEnvironment(true);
			const store = mockPersistedStore({
				[STORE_KEYS.mode]: 'light'
			});
			vi.resetModules();

			const { useTheme: freshUseTheme } = await import('./use-theme.svelte');
			const hook = freshUseTheme();

			await flushMicrotasks();

			expect(store.get).toHaveBeenCalledWith(STORE_KEYS.mode);

			// truthy savedMode → overrides the 'dark' default.
			expect(hook.mode).toBe('light');
			// `applyTheme` ran with the loaded values, not the defaults.
			expect(hook.resolved).toBe('light');
			expect(document.documentElement.getAttribute('data-theme')).toBe('light');
			expect(document.documentElement.classList.contains('dark')).toBe(false);
		});

		it('does not load the persisted theme or apply it when not running in a browser', async () => {
			mockBrowserEnvironment(false);
			const store = mockPersistedStore({
				[STORE_KEYS.mode]: 'light'
			});
			vi.resetModules();

			const { useTheme: freshUseTheme } = await import('./use-theme.svelte');
			const hook = freshUseTheme();

			await flushMicrotasks();

			// `browser && (...)()` must never run outside a browser — nothing is read
			// from the store, even with saved values available.
			expect(store.get).not.toHaveBeenCalled();
			expect(hook.mode).toBe('dark');
			expect(document.documentElement.hasAttribute('data-theme')).toBe(false);

			// `applyTheme` also guards on `browser` internally — calling it
			// (indirectly, via setMode) outside a browser must be a no-op:
			// neither `getCurrentWindow()` nor the DOM may be touched.
			await hook.setMode('light');
			await flushMicrotasks();

			expect(getCurrentWindowMock).not.toHaveBeenCalled();
			expect(document.documentElement.hasAttribute('data-theme')).toBe(false);
			expect(document.documentElement.classList.contains('dark')).toBe(false);
			expect(hook.resolved).toBe('dark');
		});
	});
});
