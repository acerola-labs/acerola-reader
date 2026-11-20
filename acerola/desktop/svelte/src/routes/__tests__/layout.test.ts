import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { NETWORK_COMMANDS } from '#lib/contracts/network/network.commands.ts';
import { NETWORK_EVENTS } from '#lib/contracts/network/network.events.ts';
import RootLayout from '../+layout.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

vi.mock('$app/stores', async () => {
	const { writable: makeWritable } = await import('svelte/store');
	const store = makeWritable({ url: new URL('http://localhost/home') });
	return { page: store };
});

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// Map by event name (not a generic mock that always returns an empty unlisten) — to really
// simulate `sync:files:started`/`sync:files:complete` arriving and prove the global header
// sync indicator (`headerSync`/`headerPeers`, see `+layout.svelte`) reacts to them. Same
// pattern as `setupListeners()` in `use-network-sync.test.ts`.
const { mockListenCallbacks } = vi.hoisted(() => ({
	mockListenCallbacks: new Map<string, (event: { payload: unknown }) => void>()
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: (event: string, callback: (event: { payload: unknown }) => void) => {
		mockListenCallbacks.set(event, callback);
		return Promise.resolve(vi.fn());
	}
}));

const { mockMinimize, mockToggleMaximize, mockCloseWin } = vi.hoisted(() => ({
	mockMinimize: vi.fn(),
	mockToggleMaximize: vi.fn(),
	mockCloseWin: vi.fn()
}));

vi.mock('@tauri-apps/api/window', () => ({
	getCurrentWindow: () => ({
		theme: vi.fn().mockResolvedValue('light'),
		setTheme: vi.fn().mockResolvedValue(undefined),
		onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() }),
		minimize: mockMinimize,
		toggleMaximize: mockToggleMaximize,
		close: mockCloseWin
	})
}));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

// useOnboarding's checkStatus() runs a single time, on module import (top-level, not inside
// the hook) — that's why the LazyStore must already return "completed" here in the vi.mock
// factory (hoisted before any import), not in beforeEach: by then it is too late.
vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn().mockResolvedValue({
		get: vi.fn().mockResolvedValue(undefined),
		set: vi.fn().mockResolvedValue(undefined),
		delete: vi.fn().mockResolvedValue(undefined),
		save: vi.fn().mockResolvedValue(undefined)
	}),
	LazyStore: class {
		constructor() {
			return {
				// only "onboarding_completed" needs to be true — the same LazyStore is reused
				// by use-theme.svelte.ts ("theme"/"mode" keys), which breaks if it gets `true`
				// as a theme/mode value instead of undefined (falls back to the default).
				get: vi.fn((key: string) =>
					Promise.resolve(key === 'onboarding_completed' ? true : undefined)
				),
				set: vi.fn().mockResolvedValue(undefined)
			};
		}
	}
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
		get_package_family_name: 'No package identity',
		// `headerPeers`/`headerSync` (global header sync indicator) call `startListening()` in
		// `onMount` — without these defaults, `usePeerConnection::loadPairedPeers` got
		// `undefined` back and broke trying to iterate (`for...of undefined`), causing an
		// unhandled rejection in the other tests of this file (same fixtures as
		// `use-peer-connection.test.ts`, to match the real shape each hook expects).
		[NETWORK_COMMANDS.getLocalId]: 'local-id',
		[NETWORK_COMMANDS.getLocalAddr]: { id: { id: 'local-id', device_id: null }, addrs: [1, 2] },
		[NETWORK_COMMANDS.getLocalDeviceInfo]: { name: 'Desktop', os: 'windows', version: '1.0' },
		[NETWORK_COMMANDS.getRelayInfo]: {
			acerolaRelayUrl: 'relay-a',
			useAcerolaRelay: true,
			useIrohPublicNetwork: false,
			customRelayUrls: [],
			hasIrohServicesTicket: false
		},
		[NETWORK_COMMANDS.getPairedPeers]: [],
		[NETWORK_COMMANDS.getSyncHistoryLog]: []
	};
	mockInvoke.mockImplementation((cmd: string) =>
		Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined))
	);
}

function childrenSnippet() {
	return createRawSnippet(() => ({
		render: () => `<div data-testid="page-content">Conteúdo da página</div>`
	}));
}

describe('root +layout', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockListenCallbacks.clear();
		setupInvokeMock();
	});

	it('renders the sidebar, header and page content once onboarding is complete', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });

		expect(await screen.findByTestId('page-content')).toBeInTheDocument();
		expect(screen.getByText('Acerola')).toBeInTheDocument();
	});

	it('calls the native window controls', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		await screen.findByTestId('page-content');
		await waitFor(() => expect(mockInvoke).toHaveBeenCalled());

		await user.click(screen.getByRole('button', { name: /minimize|minimizar/i }));
		expect(mockMinimize).toHaveBeenCalled();

		await user.click(screen.getByRole('button', { name: /maximize|maximizar/i }));
		expect(mockToggleMaximize).toHaveBeenCalled();

		await user.click(screen.getByRole('button', { name: /close|fechar/i }));
		expect(mockCloseWin).toHaveBeenCalled();
	});

	it('opens the command palette on ctrl+k and navigates to a section', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		await screen.findByTestId('page-content');

		await user.keyboard('{Control>}k{/Control}');

		const historyItem = await screen.findByText(/histórico|history/i);
		await user.click(historyItem);

		expect(mockGoto).toHaveBeenCalledWith('/history');
	});

	// Regression for the real reported problem: a files sync running with NO visual hint
	// anywhere in the UI — the user only found out by looking at the backend log. The global
	// header indicator (`headerSync.activeSession()`) must show as soon as the session starts
	// (`sync:files:started`) and vanish as soon as it ends (`sync:files:complete`), regardless
	// of which page is mounted — that's why it lives in `+layout.svelte`, not on a specific
	// screen.
	it('shows a global sync indicator in the header while a files session is active, and hides it once it completes', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });
		await screen.findByTestId('page-content');

		expect(screen.queryByText(/sincronizando|syncing/i)).not.toBeInTheDocument();

		// `startListening()` registers the listeners in series (each `await listen(...)`) after
		// having already awaited `loadPersistedLog()` — without this, firing the event right
		// after `findByTestId('page-content')` risks the callback not being in the Map yet,
		// and the test would wait for a text that never arrives.
		await waitFor(() => expect(mockListenCallbacks.has(NETWORK_EVENTS.filesStarted)).toBe(true));
		mockListenCallbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-1' });

		expect(await screen.findByText(/sincronizando|syncing/i)).toBeInTheDocument();

		mockListenCallbacks.get(NETWORK_EVENTS.filesComplete)?.({ payload: 'peer-1' });

		await waitFor(() => {
			expect(screen.queryByText(/sincronizando|syncing/i)).not.toBeInTheDocument();
		});
	});

	it('clicking the global sync indicator navigates to the network page', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });
		await screen.findByTestId('page-content');

		await waitFor(() => expect(mockListenCallbacks.has(NETWORK_EVENTS.comicStarted)).toBe(true));
		mockListenCallbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-2' });
		const indicator = await screen.findByRole('button', { name: /sincronizando|syncing/i });

		await user.click(indicator);

		expect(mockGoto).toHaveBeenCalledWith('/network');
	});
});
