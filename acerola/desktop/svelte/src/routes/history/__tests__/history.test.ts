import { render, screen, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';
import HistoryPage from '../+page.svelte';

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn(),
		loading: vi.fn()
	}
}));

const { mockInvoke, mockListen } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
	mockListen: vi.fn()
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: any) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// HistoryPage calls peers.startListening()/sync.startListening() on mount, which depend on
// `listen()` (real Tauri IPC, unavailable outside a webview) — without a mock here, the
// rejected promise leaks as an unhandled rejection and brings down the suite even with the
// asserts passing. The `mockResolvedValue` is reapplied on every test (see beforeEach)
// because `vi.resetAllMocks()` wipes the mock config — without reapplying, `listen()`
// resolves `undefined` again, and `unlisten.push(await listen(...))` pushes `undefined` into
// the array, breaking stopListening's `unlisten.forEach((fn) => fn())`.
vi.mock('@tauri-apps/api/event', () => ({
	listen: (...args: unknown[]) => mockListen(...args)
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

vi.mock('#lib/assets/placeholder/placeholder_manga.svg?component', () => ({
	default: () => ''
}));

const mockHistoryData = [
	{
		comicDirectoryId: '1',
		chapterArchiveId: '10',
		lastPage: 5,
		isCompleted: false,
		updatedAt: 1600000000,
		comicName: 'Comic 1',
		comicCover: '/path/to/cover1.jpg',
		chapterName: '1',
		folderName: 'Comic 1',
		chapterPath: '/path/to/chapter1.cbz',
		chapterSort: '1',
		isSpecial: false,
		lastModified: 0
	},
	{
		comicDirectoryId: '2',
		chapterArchiveId: '20',
		lastPage: 12,
		isCompleted: true,
		updatedAt: 1500000000,
		comicName: 'Comic 2',
		comicCover: null,
		chapterName: '2',
		folderName: 'Comic 2',
		chapterPath: '/path/to/chapter2.cbz',
		chapterSort: '2',
		isSpecial: false,
		lastModified: 0
	}
];

// This page's onMount fires history.fetch() along with peers.startListening() and
// sync.startListening(), which call invoke() concurrently (get_paired_peers,
// get_sync_history_log, get_network_status). mockResolvedValueOnce only looks at call
// order, not the command — under real concurrency any of these could "steal" the value
// meant for history_get_all. Routing by command name makes that irrelevant.
function setupInvokeMock(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		get_paired_peers: [],
		get_sync_history_log: []
	};
	mockInvoke.mockImplementation((cmd: string) =>
		Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined))
	);
}

// `sync` now comes from the context shared with `+layout.svelte` (see
// `CONTEXT_KEYS.networkSync`), not from a page-owned instance — without providing that
// context here, `getContext(...)` returns `undefined` and the first access to `sync.log`
// (inside the page's `$effect`) blows up.
function renderHistoryPage(networkSyncOverrides: Record<string, unknown> = {}) {
	return render(HistoryPage, {
		context: new Map([
			[
				CONTEXT_KEYS.networkSync,
				{
					log: [],
					isSyncing: () => false,
					lastSyncedAt: () => undefined,
					activeSession: () => undefined,
					activeProgressMessage: () => undefined,
					syncHistory: vi.fn(),
					syncFiles: vi.fn(),
					syncComic: vi.fn(),
					syncAll: vi.fn(),
					startListening: vi.fn(),
					stopListening: vi.fn(),
					...networkSyncOverrides
				}
			]
		])
	});
}

describe('HistoryPage', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		mockListen.mockResolvedValue(() => {});
	});

	it('renders empty state when there is no history', async () => {
		// Renders the empty state when there is no history
		setupInvokeMock({ history_get_all: [] });
		renderHistoryPage();

		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith('history_get_all', undefined);
		});

		expect(screen.getByText(/Nenhum histórico ainda/i)).toBeInTheDocument();
	});

	it('renders history items successfully', async () => {
		// Renders the history items successfully
		setupInvokeMock({ history_get_all: mockHistoryData });
		renderHistoryPage();

		await waitFor(() => {
			expect(screen.getByText('Comic 1')).toBeInTheDocument();
			expect(screen.getByText('Comic 2')).toBeInTheDocument();
		});
	});

	it('clears history when clicking the button', async () => {
		// Clears the history when clicking the clear button
		const user = userEvent.setup();
		setupInvokeMock({ history_get_all: mockHistoryData });
		renderHistoryPage();

		await waitFor(() => {
			expect(screen.getByText('Comic 1')).toBeInTheDocument();
		});

		// bits-ui AlertDialog.Trigger wraps the child element in its own <button>, so two buttons
		// with the same accessible name are rendered. We take the outermost trigger (index 0).
		const clearButtons = screen.getAllByRole('button', { name: /Limpar Histórico/i });
		await user.click(clearButtons[0]);

		// After opening the dialog, click the confirm action button ("Sim, limpar tudo")
		const confirmButton = await screen.findByRole('button', { name: /Sim, limpar tudo/i });
		await user.click(confirmButton);

		expect(mockInvoke).toHaveBeenCalledWith('history_clear', undefined);

		await waitFor(() => {
			expect(screen.queryByText('Comic 1')).not.toBeInTheDocument();
		});
	});

	it('does not show a completion toast on mount when sync.log already contains a completed entry', async () => {
		const { toast } = await import('svelte-sonner');
		setupInvokeMock({ history_get_all: [] });
		renderHistoryPage({
			log: [
				{
					id: 99,
					peerId: 'peer-history-past',
					kind: 'history',
					status: 'complete',
					message: 'peer-history-past',
					timestamp: Date.now()
				}
			]
		});

		await tick();
		expect(toast.success).not.toHaveBeenCalled();
	});
});
