import { describe, expect, it, vi } from 'vitest';
import { reroute } from './hooks';

// `Reroute` requires `fetch` on the request even though this implementation never uses it
// (deLocalizeUrl only looks at `url`) — stubbed here just to satisfy the type.
const fetchStub = vi.fn() as unknown as typeof fetch;

describe('reroute', () => {
	it('keeps the path unchanged for the default locale', () => {
		const result = reroute({
			url: new URL('http://localhost/docs/getting-started'),
			fetch: fetchStub
		});

		expect(result).toBe('/docs/getting-started');
	});

	it('strips the locale prefix from a localized path', () => {
		const result = reroute({
			url: new URL('http://localhost/en/docs/getting-started'),
			fetch: fetchStub
		});

		expect(result).toBe('/docs/getting-started');
	});
});
