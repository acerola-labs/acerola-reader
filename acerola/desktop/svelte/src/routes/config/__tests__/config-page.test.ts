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

describe('config +page', () => {
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

	// No accordions anymore: every section (AcerolaSettingSection) renders open, with its rows
	// (AcerolaSettingRow) visible right away — no click needed to reach a setting.
	it('renders every section open with its settings visible', () => {
		render(ConfigPage);

		expect(screen.getByRole('heading', { name: /^arquivos$/i })).toBeInTheDocument();
		expect(screen.getByRole('heading', { name: /^biblioteca$/i })).toBeInTheDocument();
		expect(screen.getByRole('heading', { name: /^metadados$/i })).toBeInTheDocument();
		expect(screen.getByRole('heading', { name: /^marcadores$/i })).toBeInTheDocument();

		// There is no theme switching section anymore — only the light/dark mode in the header.
		expect(screen.queryByText(/^aparência$/i)).not.toBeInTheDocument();
		// Rows of every section are already there, no expanding.
		expect(screen.getByText(/^templates de nomenclatura$/i)).toBeInTheDocument();
		expect(screen.getByText(/sincronização com mangadex/i)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /escolher pasta/i })).toBeInTheDocument();
	});

	it('does not navigate when just viewing the sections', () => {
		render(ConfigPage);

		expect(mockGoto).not.toHaveBeenCalled();
	});

	it('navigates to the templates route from the library "Open" button', async () => {
		const user = userEvent.setup();
		render(ConfigPage);

		await user.click(screen.getByRole('button', { name: /^abrir$/i }));

		expect(mockGoto).toHaveBeenCalledWith('/config/templates');
	});
});
