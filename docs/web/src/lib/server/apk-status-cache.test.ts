import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cachedAvailability, clearAvailabilityCache } from './apk-status-cache';

describe('cachedAvailability', () => {
	beforeEach(() => {
		clearAvailabilityCache();
	});

	it('runs the check again once the TTL expires', async () => {
		let time = 0;
		const now = () => time;
		const check = vi.fn().mockResolvedValueOnce(true).mockResolvedValueOnce(false);

		expect(await cachedAvailability(1000, check, now)).toBe(true);
		time = 999;
		expect(await cachedAvailability(1000, check, now)).toBe(true);
		time = 1000;
		expect(await cachedAvailability(1000, check, now)).toBe(false);
		expect(check).toHaveBeenCalledTimes(2);
	});

	it('does not cache a failed check', async () => {
		const check = vi
			.fn()
			.mockRejectedValueOnce(new Error('R2 down'))
			.mockResolvedValueOnce(true);

		await expect(cachedAvailability(1000, check)).rejects.toThrow('R2 down');
		expect(await cachedAvailability(1000, check)).toBe(true);
	});
});
