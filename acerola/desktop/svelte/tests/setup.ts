import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/svelte';
import { afterEach, vi } from 'vitest';

// Mock to work around jsdom and radix/bits-ui pointer event issues
if (typeof window !== 'undefined') {
	window.HTMLElement.prototype.hasPointerCapture = vi.fn();
	window.HTMLElement.prototype.releasePointerCapture = vi.fn();
	window.HTMLElement.prototype.scrollIntoView = vi.fn();
	// jsdom already exposes `Element.prototype.animate` (it is not `undefined`), but it returns
	// something incomplete — so the obvious guard `if (!Element.prototype.animate)` never catches
	// this case (it can't be used here). A plain function, NOT `vi.fn()`, for the same reason as
	// `matchMedia` below: `vi.resetAllMocks()` in several files' `beforeEach` would wipe a spy's
	// implementation. `@formkit/auto-animate` treats the return as a real `Animation` (which
	// extends `EventTarget`) and registers `finish` listeners on it — without this, it breaks
	// asynchronously (outside the test body) as soon as a list with `use:autoAnimate` gains/loses
	// an item.
	Element.prototype.animate = function animate() {
		return {
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null,
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		} as unknown as Animation;
	};
	// @ts-ignore: Simple test mock that solves hasPointerCapture
	window.PointerEvent = class PointerEvent extends Event {};

	// jsdom does not implement ResizeObserver/IntersectionObserver — used by actions such as
	// `slidingIndicator` and `AcerolaSectionNav` (scroll-spy). Guarded with `if (!X)` (unlike
	// `animate` above, these two really don't exist in jsdom by default), so as not to step on
	// stubs that a specific test already registers to drive the callbacks manually
	// (e.g. `comic-page.test.ts`).
	if (!window.ResizeObserver) {
		window.ResizeObserver = class ResizeObserver {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		};
	}
	if (!window.IntersectionObserver) {
		// @ts-ignore: minimal stub, enough for code that only calls observe/disconnect
		window.IntersectionObserver = class IntersectionObserver {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		};
	}

	// Mock localStorage
	Object.defineProperty(window, 'localStorage', {
		value: {
			getItem: vi.fn(),
			setItem: vi.fn(),
			removeItem: vi.fn(),
			clear: vi.fn()
		},
		writable: true
	});

	// Mock matchMedia — a plain function, NOT `vi.fn()`: several test files call
	// `vi.resetAllMocks()` in their own `beforeEach`, which would wipe a spy's implementation and
	// return `undefined` from `window.matchMedia(...)` for the rest of that file — breaking any
	// code that reads `.matches` on the result (e.g. `@formkit/auto-animate` checking
	// `prefers-reduced-motion` on mount). No test asserts on this global mock today
	// (`is-mobile.test.ts` registers its own isolated one when it needs to spy on calls).
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: (query: string) => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: vi.fn(), // Deprecated
			removeListener: vi.fn(), // Deprecated
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		})
	});
}

// Mock SvelteKit $app/environment — browser = true for component tests
vi.mock('$app/environment', async () => await import('./mocks/app-environment.ts'));
vi.mock('$app/state', async () => await import('./mocks/app-state.ts'));

// Mock Tauri window — getCurrentWindow does not exist in jsdom
vi.mock('@tauri-apps/api/window', () => ({
	getCurrentWindow: vi.fn(() => ({
		theme: vi.fn().mockResolvedValue('light'),
		setTheme: vi.fn().mockResolvedValue(undefined),
		onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() })
	}))
}));

// Mock Tauri store — default behavior: returns null (no saved value)
vi.mock('@tauri-apps/plugin-store', () => {
	return {
		LazyStore: class {
			constructor() {
				return {
					get: vi.fn().mockResolvedValue(null),
					set: vi.fn().mockResolvedValue(undefined)
				};
			}
		},
		load: vi.fn().mockResolvedValue({
			reload: vi.fn().mockResolvedValue(undefined),
			get: vi.fn().mockResolvedValue(null),
			set: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		})
	};
});

afterEach(async () => {
	cleanup();
	await new Promise((resolve) => setTimeout(resolve, 30));
	vi.clearAllMocks();
});
