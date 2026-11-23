import { expect, test } from '@playwright/test';
import { NETWORK_COMMANDS } from '../../svelte/src/lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '../../svelte/src/lib/contracts/network/network.events';
import {
	e2eLocalAddr,
	e2eLocalDeviceInfo,
	e2eLocalPeerId,
	e2ePairedPeer,
	e2eRelayInfo
} from '../../svelte/tests/mocks/acerola-e2e-data';
import {
	collectConsoleErrors,
	installTauriMocks,
	mockedTauriResponse
} from '../../svelte/tests/mocks/tauri-playwright';

// Replaces the old history-network.spec.ts: that test expected a manual local/relay mode
// switch in `/history` ("Modo: local" text + "Relay" button) that no longer exists in the
// app — `switch_to_local`/`switch_to_relay` are not called by any component today.
// The (read-only) network status and the paired peer list live in `/network`, which had
// no e2e coverage until now.
test.describe('network page', () => {
	let consoleErrors: string[];

	test.beforeEach(async ({ page }) => {
		consoleErrors = collectConsoleErrors(page);

		await installTauriMocks(page, {
			[NETWORK_COMMANDS.getSecurityStatus]: false,
			[NETWORK_COMMANDS.getLocalId]: e2eLocalPeerId,
			[NETWORK_COMMANDS.getLocalAddr]: e2eLocalAddr,
			[NETWORK_COMMANDS.getLocalDeviceInfo]: e2eLocalDeviceInfo,
			[NETWORK_COMMANDS.getRelayInfo]: e2eRelayInfo,
			[NETWORK_COMMANDS.getPairedPeers]: [e2ePairedPeer],
			[NETWORK_COMMANDS.getNetworkStatus]: mockedTauriResponse({
				// `refreshStatus()` ignores the invoke return value — the status only arrives via
				// the `network:status` event, exactly like the real backend behaves (see the
				// comment in `network.events.ts` about this event never being spontaneous).
				events: [{ event: NETWORK_EVENTS.status, payload: { mode: 'local', peers: [] } }]
			})
		});
	});

	test('shows device identity, local mode and paired peer', async ({ page }) => {
		test.setTimeout(10_000);

		await page.goto('/network');

		await expect(page.getByText('Meu PC')).toBeVisible();
		await expect(page.getByText('Rede local')).toBeVisible();

		await expect(page.getByText('Meu Celular')).toBeVisible();
		await expect(page.getByText('Nunca sincronizado')).toBeVisible();

		expect(consoleErrors).toEqual([]);
	});
});
