import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BOOKMARKS_COMMANDS } from '#lib/contracts/bookmarks/bookmarks.commands.ts';
import { _resetBookmarksState } from '#lib/hooks/store/use-bookmarks.svelte.ts';
import ConfigPage from '../+page.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

const { mockInvoke, mockListen } = vi.hoisted(() => ({
	mockInvoke: vi.fn().mockResolvedValue(undefined),
	mockListen: vi.fn().mockResolvedValue(vi.fn())
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: mockInvoke
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: mockListen
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

// use-theme.svelte.ts instantiates LazyStore at the module top level (outside any hook) —
// it must stay mocked here like in the global setup.ts, since this file's vi.mock replaces
// the whole module instead of extending the global mock.
vi.mock('@tauri-apps/plugin-store', () => ({
	load: mockStoreLoad,
	LazyStore: class {
		constructor() {
			return {
				get: vi.fn().mockResolvedValue(null),
				set: vi.fn().mockResolvedValue(undefined)
			};
		}
	}
}));

describe('config +page (accordion)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		_resetBookmarksState();
		// Bookmarks is now always open (flat) — `AcerolaBookmarkManager` mounts right away (not
		// only after expanding a category) and calls `loadBookmarks()` in `onMount`, so it needs
		// a list-shaped response here, otherwise `bookmarks` becomes `undefined`.
		mockInvoke.mockImplementation((cmd: string) => {
			if (
				cmd === BOOKMARKS_COMMANDS.getCategories ||
				cmd === BOOKMARKS_COMMANDS.getAllComicCategories
			) {
				return Promise.resolve([]);
			}
			return Promise.resolve(undefined);
		});
		mockListen.mockResolvedValue(vi.fn());
		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		});
	});

	// Bookmarks is always open (flat, no click) — only Files/Library/Metadata collapse by
	// default. Same UI/UX mix as the Network screen (not every section needs a click to show
	// its content).
	it('renders the flat categories already open and the collapsible ones collapsed by default', () => {
		render(ConfigPage);

		expect(screen.getByText(/^configuração dos arquivos$/i)).toBeInTheDocument();
		expect(screen.getByText(/^biblioteca$/i)).toBeInTheDocument();
		expect(screen.getByText(/^configuração de metadados$/i)).toBeInTheDocument();
		expect(screen.getByText(/^marcadores$/i)).toBeInTheDocument();

		// There is no theme switching section anymore — only the light/dark mode in the header.
		expect(screen.queryByText(/^aparência$/i)).not.toBeInTheDocument();
		// Library is still collapsible — content hidden until clicked.
		expect(screen.queryByText(/^templates de nomenclatura$/i)).not.toBeInTheDocument();
	});

	it('expands a collapsible category inline instead of navigating', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^configuração de metadados$/i));

		expect(await screen.findByText(/sincronização com mangadex/i)).toBeInTheDocument();
		expect(mockGoto).not.toHaveBeenCalled();

		// aria-expanded on the header changes synchronously — removing the content itself goes
		// through transition:slide, whose outro may not complete in jsdom (no real layout), so
		// the reliable signal here is the attribute, not the content.
		await user.click(screen.getByText(/^configuração de metadados$/i));

		expect(screen.getByText(/^configuração de metadados$/i).closest('button')).toHaveAttribute(
			'aria-expanded',
			'false'
		);
	});

	it('keeps a flat section visible alongside an expanded collapsible category', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^biblioteca$/i));

		// Bookmarks (flat) never stopped being visible; Library (collapsible) now is too.
		expect(screen.getByText(/^marcadores$/i)).toBeInTheDocument();
		expect(await screen.findByText(/^templates de nomenclatura$/i)).toBeInTheDocument();
	});

	it('still navigates to the templates route from within library', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByText(/^biblioteca$/i));
		await user.click(await screen.findByText(/^templates de nomenclatura$/i));

		expect(mockGoto).toHaveBeenCalledWith('/config/templates');
	});
});
