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

	// Same layout as Config and Network: one section per subject, all open, nothing behind a click.
	it('renders every section open', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		for (const title of [
			/^leitura$/i,
			/^metadados$/i,
			/^sincronização de arquivos$/i,
			/^sincronizar com dispositivo$/i,
			/^capa$/i,
			/^zona de risco$/i
		]) {
			expect(screen.getByRole('heading', { name: title })).toBeInTheDocument();
		}

		// Section content is visible right away.
		expect(screen.getByText(/atribua um marcador/i)).toBeInTheDocument();
		expect(screen.getByText(/sincronização externa/i)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Limpar' })).toBeInTheDocument();
	});

	it('hides volume preference when there is no volume structure', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		expect(screen.queryByText('Destaque do volume')).not.toBeInTheDocument();
		expect(screen.queryByRole('radio', { name: 'Capa' })).not.toBeInTheDocument();
	});

	it('displays volume preference when volume structure exists', () => {
		render(AcerolaComicPreferences, {
			props: {
				...defaultProps(),
				data: { hasVolumeStructure: true }
			}
		});

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

	it('shows the mangadex/anilist rows with a sync button while external sync is active', () => {
		render(AcerolaComicPreferences, { props: defaultProps() });

		expect(screen.getByText(/sincronização com mangadex/i)).toBeInTheDocument();
		expect(screen.getByText(/sincronização com anilist/i)).toBeInTheDocument();
	});

	it('toggles external sync through its switch', async () => {
		const user = userEvent.setup();
		const props = defaultProps();
		render(AcerolaComicPreferences, {
			props: { ...props, state: { ...props.state, externalSyncEnabled: false } }
		});

		const toggle = screen.getByRole('switch', { name: /sincronização externa/i });
		expect(toggle).toHaveAttribute('aria-checked', 'false');

		await user.click(toggle);

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

		await user.click(screen.getByRole('radio', { name: 'Banner' }));

		expect(props.events.onVolumeViewModeChange).toHaveBeenCalledWith('banner');
	});
});
