import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';
import { ActiveComicState } from '#lib/state/comic-context.svelte.ts';
import { HOME_COMMANDS } from '#lib/contracts/home/home.commands.ts';
import { HOME_EVENTS } from '#lib/contracts/home/home.events.ts';
import type {
	ComicSummaryPayload,
	ComicSummaryItemPayload
} from '#lib/contracts/home/home.payloads.ts';
import { _resetComicSummaryState } from '#lib/hooks/store/use-comic-summary.svelte.ts';
import { _resetComicSelectionState } from '#lib/hooks/store/use-comic-selection.svelte.ts';
import { _resetBookmarksState } from '#lib/hooks/store/use-bookmarks.svelte.ts';
import HomePage from '../+page.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// Just like on the comic screen: the backend answers `getComicSummarySorted` via an event
// (`home:data`/`home:error`), not through the invoke return — we capture the callbacks by
// event name to simulate the data arriving.
const listenCallbacks = new Map<string, (event: { payload: unknown }) => void>();

vi.mock('@tauri-apps/api/event', () => ({
	listen: (event: string, callback: (event: { payload: unknown }) => void) => {
		listenCallbacks.set(event, callback);
		return Promise.resolve(vi.fn());
	}
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

vi.mock('@tauri-apps/plugin-store', () => ({ load: mockStoreLoad }));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

vi.mock('svelte-sonner', () => ({
	toast: {
		info: vi.fn(),
		loading: vi.fn(() => 'mock-toast-id'),
		success: vi.fn(),
		error: vi.fn()
	}
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}, rejects: string[] = []) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
		get_paired_peers: [],
		get_network_status: undefined,
		get_sync_history_log: []
	};
	mockInvoke.mockImplementation((cmd: string) => {
		if (rejects.includes(cmd)) return Promise.reject(new Error('offline'));
		return Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined));
	});
}

function comic(overrides: Partial<ComicSummaryItemPayload> = {}): ComicSummaryItemPayload {
	return {
		relations: { directoryId: 'dir-1', metadataId: null },
		filesystem: { folderName: 'Acerola' },
		metadata: { title: 'Acerola', externalSync: false, activeSource: 'LOCAL', chapterCount: 3 },
		artwork: { cover: null, banner: null },
		...overrides
	};
}

function summaryPayload(overrides: Partial<ComicSummaryPayload> = {}): ComicSummaryPayload {
	return {
		total: 1,
		fetchedAt: '2026-06-07T12:00:00.000Z',
		comics: [comic()],
		...overrides
	};
}

// `sync`/`p2pSync` now comes from the context shared with `+layout.svelte` (see
// `CONTEXT_KEYS.networkSync`), not from a page-owned instance — without providing that
// context here, `getContext(...)` returns `undefined` and the first access to `sync.log`
// (inside the page's `$effect`) blows up.
function networkSyncStub() {
	return {
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
		stopListening: vi.fn()
	};
}

function renderHomePage(networkSyncOverrides: Record<string, unknown> = {}) {
	const activeComic = new ActiveComicState();
	// Explicit `Map<symbol, unknown>`: without it, TS infers the type from the FIRST tuple
	// (`ActiveComicState`) and rejects the second entry, whose value has a different shape —
	// both sides of `getContext<T>(...)` already cast to the right type on read.
	return render(HomePage, {
		context: new Map<symbol, unknown>([
			[CONTEXT_KEYS.activeComic, activeComic],
			[CONTEXT_KEYS.networkSync, { ...networkSyncStub(), ...networkSyncOverrides }]
		])
	});
}

async function emitSummary(payload: ComicSummaryPayload) {
	await waitFor(() => expect(listenCallbacks.has(HOME_EVENTS.homeData)).toBe(true));
	listenCallbacks.get(HOME_EVENTS.homeData)?.({ payload });
	await tick();
}

describe('home +page', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		listenCallbacks.clear();
		_resetComicSummaryState();
		_resetComicSelectionState();
		_resetBookmarksState();

		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			delete: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined),
			reload: vi.fn().mockResolvedValue(undefined)
		});
	});

	it('shows the empty state when the library has no comics', async () => {
		renderHomePage();

		await emitSummary(summaryPayload({ total: 0, comics: [] }));

		expect(await screen.findByText(/nenhum quadrinho|no comics/i)).toBeInTheDocument();
	});

	it('keeps the filter button visible and shows the filtered-empty message when a filter zeroes the result', async () => {
		const user = userEvent.setup();
		renderHomePage();

		await emitSummary(summaryPayload());
		await screen.findByText('Acerola');

		await user.click(screen.getByRole('button', { name: /filtrar e ordenar|^filter/i }));
		await user.click(await screen.findByText(/mostrar ocultos|show hidden/i));
		await user.click(screen.getByRole('button', { name: /^aplicar$|^apply$/i }));

		await emitSummary(summaryPayload({ total: 0, comics: [] }));

		expect(
			screen.getByRole('button', { name: /filtrar e ordenar|^filter/i })
		).toBeInTheDocument();
		expect(
			await screen.findByText(/nenhum quadrinho encontrado para o filtro|no results.*filter/i)
		).toBeInTheDocument();
	});

	it('renders the comic grid once the summary event arrives', async () => {
		renderHomePage();

		await waitFor(() =>
			expect(mockInvoke).toHaveBeenCalledWith(
				HOME_COMMANDS.getComicSummarySorted,
				expect.objectContaining({ sortBy: 'title', sortOrder: 'asc' })
			)
		);

		await emitSummary(summaryPayload());

		expect(await screen.findByText('Acerola')).toBeInTheDocument();
	});

	it('opens the comic on card click and stores it in the active comic context', async () => {
		const user = userEvent.setup();
		renderHomePage();

		await emitSummary(summaryPayload());

		// The title sits OUTSIDE the clickable <button> (only the image/placeholder is clickable,
		// see acerola-card-image.svelte) — go up to the card and click the button inside it.
		const title = await screen.findByText('Acerola');
		const cardButton = title.closest('.group')?.querySelector('button');
		await user.click(cardButton as HTMLElement);

		expect(mockGoto).toHaveBeenCalledWith('/comic/Acerola');
	});

	it('shows an error toast when the summary fetch fails', async () => {
		const { toast } = await import('svelte-sonner');
		renderHomePage();

		await waitFor(() => expect(listenCallbacks.has(HOME_EVENTS.homeError)).toBe(true));
		listenCallbacks.get(HOME_EVENTS.homeError)?.({
			payload: { errorType: 'Unknown', message: 'falha ao carregar' }
		});

		await waitFor(() => expect(toast.error).toHaveBeenCalledWith('falha ao carregar'));
	});

	it('enters selection mode when clicking a card action button', async () => {
		const user = userEvent.setup();
		renderHomePage();
		await emitSummary(summaryPayload());
		await screen.findByText('Acerola');

		// The card actions button is just the MoreVertical icon (no accessible name) —
		// locate it by the lucide svg instead of by text/role.
		const actionButton = document
			.querySelector('svg.lucide-more-vertical, svg.lucide-ellipsis-vertical')
			?.closest('button');
		expect(actionButton).toBeTruthy();

		await user.click(actionButton as HTMLElement);

		expect(await screen.findByText(/1 selecionado|1 selected/i)).toBeInTheDocument();
	});

	it('does not show a completion toast on mount when sync.log already contains a completed entry', async () => {
		const { toast } = await import('svelte-sonner');
		renderHomePage({
			log: [
				{
					id: 99,
					peerId: 'peer-past',
					kind: 'comic',
					status: 'complete',
					message: 'peer-past',
					timestamp: Date.now(),
					comicName: 'Acerola'
				}
			]
		});

		await tick();
		expect(toast.success).not.toHaveBeenCalled();
	});
});
