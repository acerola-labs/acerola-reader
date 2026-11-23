export const APP_ORIGIN =
	process.platform === 'win32' ? 'http://tauri.localhost' : 'tauri://localhost';
const APP_READY_SELECTOR = '[data-tauri-drag-region]';
const APP_ORIGINS = ['http://tauri.localhost', 'https://tauri.localhost', 'tauri://localhost'];

type AppState = {
	href: string;
	pathname: string;
	readyState: DocumentReadyState;
	bodyChildCount: number;
	hasAppLayout: boolean;
};

async function getAppState() {
	return browser.execute((readySelector) => {
		return {
			href: window.location.href,
			pathname: window.location.pathname,
			readyState: document.readyState,
			bodyChildCount: document.body?.children.length ?? 0,
			hasAppLayout: Boolean(document.querySelector(readySelector))
		} satisfies AppState;
	}, APP_READY_SELECTOR);
}

export async function waitForTauriReady() {
	await browser.waitUntil(
		async () =>
			browser.execute(() => {
				const tauriWindow = window as typeof window & {
					__TAURI_INTERNALS__?: unknown;
				};
				return Boolean(tauriWindow.__TAURI_INTERNALS__);
			}),
		{
			timeout: 5_000,
			interval: 100,
			timeoutMsg: 'window.__TAURI_INTERNALS__ did not become available in the WebView.'
		}
	);
}

export async function navigateTo(route: string) {
	// WebView2/Tauri need the custom origin; a relative URL breaks on about:blank.
	const normalizedRoute = route.startsWith('/') ? route : `/${route}`;
	const fullUrl = `${APP_ORIGIN}${normalizedRoute}`;

	await browser.url(fullUrl);

	await browser.waitUntil(
		async () => {
			const state = await getAppState();
			return state.pathname === normalizedRoute;
		},
		{
			timeout: 5_000,
			interval: 100,
			timeoutMsg: `WebView did not navigate to ${normalizedRoute}.`
		}
	);
}

export async function navigateToWithState(route: string, state: Record<string, unknown>) {
	const normalizedRoute = route.startsWith('/') ? route : `/${route}`;
	const fullUrl = `${APP_ORIGIN}${normalizedRoute}`;

	await browser.execute(
		({ targetUrl, pageState }) => {
			const currentState = window.history.state ?? {};
			const fallbackIndex = Date.now();
			const currentHistoryIndex = Number(currentState['sveltekit:history']) || fallbackIndex;
			const currentNavigationIndex =
				Number(currentState['sveltekit:navigation']) || fallbackIndex;

			const nextState = {
				...currentState,
				'sveltekit:history': currentHistoryIndex + 1,
				'sveltekit:navigation': currentNavigationIndex + 1,
				'sveltekit:states': pageState
			};

			window.history.pushState(nextState, '', targetUrl);
			window.dispatchEvent(new PopStateEvent('popstate', { state: nextState }));
		},
		{ targetUrl: fullUrl, pageState: state }
	);

	await browser.waitUntil(
		async () => {
			const currentState = await getAppState();
			return currentState.pathname === normalizedRoute;
		},
		{
			timeout: 5_000,
			interval: 100,
			timeoutMsg: `WebView did not navigate to ${normalizedRoute} with page state.`
		}
	);
}

export async function setStoreValues(entries: Record<string, unknown>) {
	const state = await getAppState();
	if (!APP_ORIGINS.some((origin) => state.href.startsWith(origin))) {
		await navigateTo('/home');
	}

	await browser.execute(async (items) => {
		const tauriWindow = window as typeof window & {
			__TAURI_INTERNALS__?: {
				invoke: (cmd: string, payload?: unknown) => Promise<unknown>;
			};
		};
		if (tauriWindow.__TAURI_INTERNALS__) {
			const rid = await tauriWindow.__TAURI_INTERNALS__.invoke('plugin:store|load', {
				path: 'settings.json',
				options: {}
			});
			for (const [k, v] of Object.entries(items)) {
				await tauriWindow.__TAURI_INTERNALS__.invoke('plugin:store|set', {
					rid,
					key: k,
					value: v
				});
			}
			await tauriWindow.__TAURI_INTERNALS__.invoke('plugin:store|save', { rid });
		}
	}, entries);
}

export async function setStoreValue(key: string, value: unknown) {
	return setStoreValues({ [key]: value });
}

export async function getStoreValue(key: string) {
	return browser.execute(async (k) => {
		try {
			const tauriWindow = window as typeof window & {
				__TAURI_INTERNALS__?: {
					invoke: (cmd: string, payload?: unknown) => Promise<unknown>;
				};
			};
			if (tauriWindow.__TAURI_INTERNALS__) {
				const rid = await tauriWindow.__TAURI_INTERNALS__.invoke('plugin:store|load', {
					path: 'settings.json',
					options: {}
				});
				const res = await tauriWindow.__TAURI_INTERNALS__.invoke('plugin:store|get', {
					rid,
					key: k
				});
				return Array.isArray(res) ? res[0] : res;
			}
			return null;
		} catch (err: any) {
			return null;
		}
	}, key);
}

export async function ensureOnboardingCompleted() {
	await waitForTauriReady();

	const alreadyCompleted = await getStoreValue('onboarding_completed');
	if (alreadyCompleted) return;

	await setStoreValue('onboarding_completed', true);
	// See the matching comment in waitForAppReady: without a real reload, the
	// in-memory $state of use-onboarding.svelte.ts does not pick up the new
	// value. navigateTo('/home') does not work here because, if the webview is
	// already on /home, navigating to the same URL is a no-op — refresh() forces
	// an actual reload.
	await browser.refresh();
	await waitForTauriReady();
}

export async function waitForAppReady(skipOnboardingComplete = false) {
	// __TAURI_INTERNALS__ is already available on about:blank; load the real route first.
	await waitForTauriReady();

	const state = await getAppState();
	if (!state.hasAppLayout && !APP_ORIGINS.some((origin) => state.href.startsWith(origin))) {
		await navigateTo('/home');
	}

	if (!skipOnboardingComplete) {
		const alreadyCompleted = await getStoreValue('onboarding_completed');

		if (!alreadyCompleted) {
			await setStoreValue('onboarding_completed', true);

			// INFO: use-onboarding.svelte.ts reads the store only once, on module
			// boot (checkStatus() runs a single time when the module is imported) —
			// writing the value to the store after the app has loaded does not
			// change the $state already in memory. navigateTo('/home') does not
			// work here because the webview is already on /home at this point
			// (line above) — navigating to the same URL is a no-op for WebView2,
			// the module never re-evaluates. refresh() forces an actual reload.
			await browser.refresh();
			await waitForTauriReady();
		}
	}

	try {
		await browser.waitUntil(
			async () => {
				const currentState = await getAppState();
				return currentState.readyState === 'complete' && currentState.hasAppLayout;
			},
			{
				timeout: 5_000,
				interval: 100,
				timeoutMsg: 'App DOM did not render after __TAURI_INTERNALS__ became available.'
			}
		);
	} catch {
		const currentState = await getAppState();
		throw new Error(
			`App DOM did not render after __TAURI_INTERNALS__ became available. State: ${JSON.stringify(
				currentState
			)}`
		);
	}
}

export async function getTitle() {
	return browser.execute(() => document.title);
}

export async function getPathname() {
	return browser.execute(() => window.location.pathname);
}

export async function expectPathname(pathname: string) {
	await browser.waitUntil(async () => (await getPathname()) === pathname, {
		timeout: 5_000,
		interval: 100,
		timeoutMsg: `Current route did not settle on ${pathname}.`
	});
}

export function xpathLiteral(value: string) {
	if (!value.includes("'")) return `'${value}'`;
	if (!value.includes('"')) return `"${value}"`;

	return `concat(${value
		.split("'")
		.map((part) => `'${part}'`)
		.join(`, "'", `)})`;
}

export function exactTextSelector(text: string) {
	return `//*[normalize-space()=${xpathLiteral(text)}]`;
}

export function containsTextSelector(text: string) {
	return `//*[contains(normalize-space(), ${xpathLiteral(text)})]`;
}

export async function waitForText(text: string, timeout = 5_000) {
	return firstDisplayed(exactTextSelector(text), timeout);
}

export async function waitForTextContaining(text: string, timeout = 5_000) {
	return firstDisplayed(containsTextSelector(text), timeout);
}

export async function isTextDisplayed(text: string) {
	const elements = await browser.$$(exactTextSelector(text));

	for (const element of elements) {
		if (await element.isDisplayed().catch(() => false)) return true;
	}

	return false;
}

export async function clickText(text: string, timeout = 5_000) {
	const element = await waitForText(text, timeout);
	await element.click();
	return element;
}

/**
 * The navigation dock (AcerolaDock) opens in `hover` mode: it stays collapsed
 * into a small pill until the mouse passes over it — the `<a href>` links only
 * exist in the DOM after that (see acerola-dock.svelte:33-48). Call this
 * before any selector that depends on the dock items.
 */
export async function expandDock(timeout = 5_000) {
	const dockArea = await firstDisplayed('[role="navigation"]', timeout);
	await dockArea.moveTo();

	await browser.waitUntil(
		async () => {
			const links = await browser.$$('[role="navigation"] a[href]');
			return (await links.length) > 0;
		},
		{
			timeout,
			interval: 100,
			timeoutMsg: 'Navigation dock did not expand after hover.'
		}
	);
}

export async function firstDisplayed(selector: string, timeout = 5_000) {
	let lastCount = 0;

	await browser.waitUntil(
		async () => {
			const elements = await browser.$$(selector);
			lastCount = await elements.length;

			for (const element of elements) {
				if (await element.isDisplayed().catch(() => false)) return true;
			}

			return false;
		},
		{
			timeout,
			interval: 100,
			timeoutMsg: `No visible element for selector "${selector}" (${lastCount} found).`
		}
	);

	const elements = await browser.$$(selector);
	for (const element of elements) {
		if (await element.isDisplayed().catch(() => false)) return element;
	}

	throw new Error(`No visible element for selector "${selector}".`);
}

export async function invokeTauriCommand<T = unknown>(command: string, args?: unknown) {
	return browser.execute(
		async ({ command: commandName, args: commandArgs }) => {
			const tauriWindow = window as typeof window & {
				__TAURI_INTERNALS__?: {
					invoke: (cmd: string, payload?: unknown) => Promise<unknown>;
				};
			};

			if (!tauriWindow.__TAURI_INTERNALS__) {
				throw new Error('window.__TAURI_INTERNALS__ unavailable.');
			}

			return tauriWindow.__TAURI_INTERNALS__.invoke(commandName, commandArgs);
		},
		{ command, args }
	) as Promise<T>;
}
