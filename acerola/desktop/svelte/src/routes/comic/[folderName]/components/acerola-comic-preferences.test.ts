import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaComicPreferences from './acerola-comic-preferences.svelte';

describe('AcerolaComicPreferences', () => {
	function defaultProps() {
		return {
			state: {
				volumeViewMode: 'cover' as const,
				bookmarkId: null,
				externalSyncEnabled: true
			},
			events: {
				onVolumeViewModeChange: vi.fn(),
				onBookmarkChange: vi.fn(),
				onExternalSyncChange: vi.fn()
			}
		};
	}

	async function openCategory(user: ReturnType<typeof userEvent.setup>, name: RegExp) {
		await user.click(screen.getByText(name));
	}

	function categoryButton(name: RegExp) {
		return screen.getByText(name).closest('button')!;
	}

	// Sync is always open (flat) — it is the most used category (metadata, files and paired
	// devices); Reading and Advanced stay collapsed by default. Same UI/UX mix as the config/
	// root (Appearance/Bookmarks flat, the rest collapsed).
	it('renders the flat sync category already open and the others collapsed by default', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		expect(screen.getByText(/^leitura$/i)).toBeInTheDocument();
		expect(screen.getByText(/^sincronização$/i)).toBeInTheDocument();
		expect(screen.getByText(/^avançado$/i)).toBeInTheDocument();

		// Sync is flat — the content is already visible, no click needed.
		expect(screen.getByText(/sincronização externa/i)).toBeInTheDocument();
		// Reading and Advanced stay collapsed.
		expect(screen.queryByText(/atribua um marcador/i)).not.toBeInTheDocument();
	});

	it('expands a category inline when clicked', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);

		expect(screen.getByText(/atribua um marcador/i)).toBeInTheDocument();
	});

	it('collapses the category again on a second click', async () => {
		// aria-expanded on the header button changes synchronously — removing the content
		// itself goes through transition:slide, whose outro may not complete in jsdom (no real
		// layout), so the reliable signal here is the attribute, not the content.
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);
		expect(categoryButton(/^leitura$/i)).toHaveAttribute('aria-expanded', 'true');

		await openCategory(user, /^leitura$/i);
		expect(categoryButton(/^leitura$/i)).toHaveAttribute('aria-expanded', 'false');
	});

	it('keeps a collapsible category expanded alongside the always-open sync section', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);

		expect(screen.getByText(/atribua um marcador/i)).toBeInTheDocument();
		expect(screen.getByText(/sincronização externa/i)).toBeInTheDocument();
	});

	it('hides volume preference when there is no volume structure', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, { props: defaultProps() });

		await openCategory(user, /^leitura$/i);

		expect(screen.queryByText('Destaque do volume')).not.toBeInTheDocument();
		expect(screen.queryByRole('radio', { name: 'Capa' })).not.toBeInTheDocument();
	});

	it('displays volume preference when volume structure exists', async () => {
		const user = userEvent.setup();
		render(AcerolaComicPreferences, {
			props: {
				...defaultProps(),
				data: { hasVolumeStructure: true }
			}
		});

		await openCategory(user, /^leitura$/i);

		expect(screen.getByText('Destaque do volume')).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Capa' })).toBeInTheDocument();
		expect(screen.getByRole('radio', { name: 'Banner' })).toBeInTheDocument();
	});

	it('shows the mangadex/anilist sync buttons only while external sync is active', () => {
		render(AcerolaComicPreferences, {
			props: {
				...defaultProps(),
				state: { ...defaultProps().state, externalSyncEnabled: false }
			}
		});

		expect(screen.getByText(/sincronização externa/i)).toBeInTheDocument();
		expect(screen.queryByText(/sincronização com mangadex/i)).not.toBeInTheDocument();
	});

	it('toggles external sync when clicking its card', async () => {
		const user = userEvent.setup();
		const props = defaultProps();
		render(AcerolaComicPreferences, {
			props: { ...props, state: { ...props.state, externalSyncEnabled: false } }
		});

		await user.click(screen.getByText(/sincronização externa/i));

		expect(props.events.onExternalSyncChange).toHaveBeenCalledWith(true);
	});

	it('changes volume highlight when clicking banner', async () => {
		const user = userEvent.setup();
		const props = defaultProps();
		render(AcerolaComicPreferences, {
			props: {
				...props,
				data: { hasVolumeStructure: true }
			}
		});

		await openCategory(user, /^leitura$/i);
		await user.click(screen.getByRole('radio', { name: 'Banner' }));

		expect(props.events.onVolumeViewModeChange).toHaveBeenCalledWith('banner');
	});
});
