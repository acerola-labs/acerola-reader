import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkRelaySettingsCard, {
	type NetworkRelaySettingsCardData
} from './acerola-network-relay-settings-card.svelte';

function data(overrides: Partial<NetworkRelaySettingsCardData> = {}): NetworkRelaySettingsCardData {
	return {
		acerolaRelayUrl: 'https://relay.acerola.app',
		useAcerolaRelay: true,
		useIrohPublicNetwork: false,
		customRelayUrls: [],
		hasIrohServicesTicket: false,
		...overrides
	};
}

function events() {
	return {
		onToggleAcerolaRelay: vi.fn().mockResolvedValue(undefined),
		onToggleIrohPublicNetwork: vi.fn().mockResolvedValue(undefined),
		onAddCustomRelayUrl: vi.fn().mockResolvedValue(undefined),
		onRemoveCustomRelayUrl: vi.fn().mockResolvedValue(undefined),
		onSetIrohServicesTicket: vi.fn().mockResolvedValue(undefined),
		onClearIrohServicesTicket: vi.fn().mockResolvedValue(undefined),
		onRestart: vi.fn().mockResolvedValue(undefined)
	};
}

/// Toggling the active relay source (Acerola / public Iroh) opens a confirmation
/// `AcerolaAlertDialog` instead of applying directly — this extra step is needed before
/// checking the handler.
async function confirmSwitch() {
	await fireEvent.click(screen.getByRole('button', { name: /Switch relay|Trocar relay/i }));
}

function acerolaRelaySwitch() {
	return screen.getByRole('switch', { name: /Use Acerola's relay|Usar o relay do Acerola/i });
}

function irohSwitch() {
	return screen.getByRole('switch', {
		name: /Use Iroh Services \(own account\)|Usar a Iroh Services \(conta própria\)/i
	});
}

/// The ticket form sits behind the row's "Ticket" button — pre-expanded only while no ticket
/// is saved yet (see `ticketExpanded` in the component).
async function expandTicketSection() {
	await fireEvent.click(screen.getByRole('button', { name: /^Ticket$/i }));
}

/// The custom relays list opens from the row's "Manage" button, apart from enabling the source.
async function expandCustomRelaysSection() {
	await fireEvent.click(screen.getByRole('button', { name: /Manage|Gerenciar/i }));
}

describe('AcerolaNetworkRelaySettingsCard', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('shows the mDNS-only summary when no relay source is active', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useAcerolaRelay: false }), events: events() }
		});

		expect(screen.getByText(/mDNS|Só rede local/i)).toBeInTheDocument();
	});

	it('shows an active-sources summary count', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: {
				data: data({ customRelayUrls: ['https://relay-a.test.local'] }),
				events: events()
			}
		});

		expect(
			screen.getByText(/2 relay source\(s\) active|2 fonte\(s\) de relay ativa/i)
		).toBeInTheDocument();
	});

	it('shows the iroh public network summary when active', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useIrohPublicNetwork: true }), events: events() }
		});

		expect(
			screen.getByText(/Iroh's public relay network|rede pública de relays do Iroh/i)
		).toBeInTheDocument();
	});

	it('toggles the acerola relay switch', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });

		await fireEvent.click(acerolaRelaySwitch());
		await confirmSwitch();

		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledWith(false);
	});

	it('reflects whether the acerola relay is active on its switch', () => {
		const { unmount } = render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useAcerolaRelay: true }), events: events() }
		});
		expect(acerolaRelaySwitch()).toHaveAttribute('aria-checked', 'true');
		unmount();

		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useAcerolaRelay: false }), events: events() }
		});
		expect(acerolaRelaySwitch()).toHaveAttribute('aria-checked', 'false');
	});

	it('toggles the iroh public network switch when a ticket is configured', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: true }), events: handlers }
		});

		await fireEvent.click(irohSwitch());
		await confirmSwitch();

		expect(handlers.onToggleIrohPublicNetwork).toHaveBeenCalledWith(true);
	});

	it('disables the switches while a toggle is still applying and ignores a second click on it', async () => {
		const handlers = events();
		let resolveToggle: (() => void) | undefined;
		handlers.onToggleAcerolaRelay.mockImplementation(
			() =>
				new Promise<void>((resolve) => {
					resolveToggle = resolve;
				})
		);
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: true }), events: handlers }
		});

		await fireEvent.click(acerolaRelaySwitch());
		await confirmSwitch();

		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledTimes(1);
		expect(acerolaRelaySwitch()).toBeDisabled();
		// The Iroh switch would only be enabled here if not for the `restarting` guard
		// (ticket already configured, so that isn't why it is disabled).
		expect(irohSwitch()).toBeDisabled();

		// A second click while the first change hasn't finished (double click, or touching
		// another relay source) must not fire a second restart in parallel — neither through
		// the `disabled` attribute (natively blocked), nor through the guard in
		// `runRestartingAction` if the click got through some other way.
		await fireEvent.click(acerolaRelaySwitch());
		expect(handlers.onToggleAcerolaRelay).toHaveBeenCalledTimes(1);

		resolveToggle?.();
	});

	it('disables the iroh public network switch without a configured ticket', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: false }), events: events() }
		});

		expect(irohSwitch()).toBeDisabled();
	});

	it('saves an iroh services ticket', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });

		// Without a configured ticket, the section comes pre-expanded — no click needed to open it.
		const input = screen.getByPlaceholderText(/services\.iroh\.computer/i);
		await fireEvent.input(input, { target: { value: 'services-fake-ticket' } });
		await fireEvent.click(screen.getByRole('button', { name: /Save ticket|Salvar ticket/i }));

		expect(handlers.onSetIrohServicesTicket).toHaveBeenCalledWith('services-fake-ticket');
	});

	it('shows an error when saving an invalid ticket fails', async () => {
		const handlers = events();
		handlers.onSetIrohServicesTicket.mockRejectedValueOnce(new Error('invalid ticket'));
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });

		const input = screen.getByPlaceholderText(/services\.iroh\.computer/i);
		await fireEvent.input(input, { target: { value: 'not-a-valid-ticket' } });
		await fireEvent.click(screen.getByRole('button', { name: /Save ticket|Salvar ticket/i }));

		expect(
			await screen.findByText(/[Ii]nvalid ticket|[Tt]icket inválido/i)
		).toBeInTheDocument();
	});

	it('removes a configured iroh services ticket', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ hasIrohServicesTicket: true }), events: handlers }
		});
		// Ticket already configured — the section comes collapsed by default, expand it first.
		await expandTicketSection();

		await fireEvent.click(
			screen.getByRole('button', { name: /Remove ticket|Remover ticket/i })
		);

		expect(handlers.onClearIrohServicesTicket).toHaveBeenCalled();
	});

	it('adds a custom relay url', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCustomRelaysSection();

		const input = screen.getByPlaceholderText(
			/your-relay\.example\.com|seu-relay\.exemplo\.com/i
		);
		await fireEvent.input(input, { target: { value: 'https://relay-a.test.local' } });
		await fireEvent.click(
			screen.getByRole('button', { name: /Add custom relay|Adicionar relay próprio/i })
		);

		expect(handlers.onAddCustomRelayUrl).toHaveBeenCalledWith('https://relay-a.test.local');
	});

	it('rejects an invalid custom relay url without calling the handler', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });
		await expandCustomRelaysSection();

		const input = screen.getByPlaceholderText(
			/your-relay\.example\.com|seu-relay\.exemplo\.com/i
		);
		await fireEvent.input(input, { target: { value: 'not-a-url' } });
		await fireEvent.click(
			screen.getByRole('button', { name: /Add custom relay|Adicionar relay próprio/i })
		);

		expect(handlers.onAddCustomRelayUrl).not.toHaveBeenCalled();
		expect(screen.getByText(/valid URL|URL válida/i)).toBeInTheDocument();
	});

	it('removes a custom relay url', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, {
			props: {
				data: data({ customRelayUrls: ['https://relay-a.test.local'] }),
				events: handlers
			}
		});
		await expandCustomRelaysSection();

		await fireEvent.click(
			screen.getByRole('button', { name: /Remove custom relay|Remover relay próprio/i })
		);

		expect(handlers.onRemoveCustomRelayUrl).toHaveBeenCalledWith('https://relay-a.test.local');
	});

	it('shows how many custom relays are configured once at least one url exists', () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: {
				data: data({ customRelayUrls: ['https://relay-a.test.local'] }),
				events: events()
			}
		});

		expect(screen.getByText(/1 configured|1 configurado/i)).toBeInTheDocument();
	});

	it('disables the custom relay input while the iroh public network is active', async () => {
		render(AcerolaNetworkRelaySettingsCard, {
			props: { data: data({ useIrohPublicNetwork: true }), events: events() }
		});
		await expandCustomRelaysSection();

		expect(
			screen.getByPlaceholderText(/your-relay\.example\.com|seu-relay\.exemplo\.com/i)
		).toBeDisabled();
	});

	it('restarts the p2p module', async () => {
		const handlers = events();
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });

		await fireEvent.click(
			screen.getByRole('button', { name: /Restart P2P module|Reiniciar módulo P2P/i })
		);

		expect(handlers.onRestart).toHaveBeenCalled();
	});

	it('shows an error when restarting fails', async () => {
		const handlers = events();
		handlers.onRestart.mockRejectedValueOnce(new Error('restart failed'));
		render(AcerolaNetworkRelaySettingsCard, { props: { data: data(), events: handlers } });

		await fireEvent.click(
			screen.getByRole('button', { name: /Restart P2P module|Reiniciar módulo P2P/i })
		);

		expect(
			await screen.findByText(/Couldn't restart the P2P module|Não foi possível reiniciar/i)
		).toBeInTheDocument();
	});
});
