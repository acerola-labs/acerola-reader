import { beforeEach, describe, expect, it, vi } from 'vitest';
import { clearAvailabilityCache } from '#lib/server/apk-status-cache.ts';
import { getApkBucket } from '#lib/server/r2.ts';
import { GET } from './+server';

vi.mock('#lib/server/r2.ts', () => ({ getApkBucket: vi.fn() }));

function withObjects(objects: { key: string }[] | undefined) {
	vi.mocked(getApkBucket).mockReturnValue(
		objects === undefined ? undefined : { list: async () => ({ objects }) }
	);
}

const event = {} as Parameters<typeof GET>[0];

describe('GET /api/apk-status', () => {
	beforeEach(() => {
		vi.mocked(getApkBucket).mockReset();
		clearAvailabilityCache();
	});

	it('reports available when the latest/ folder has a matching APK', async () => {
		withObjects([{ key: 'android/latest/acerola-1.0.15.apk' }]);
		const response = await GET(event);

		expect(await response.json()).toEqual({ available: true });
		expect(response.headers.get('Cache-Control')).toBe('public, max-age=300');
	});

	it('reports unavailable when the latest/ folder is empty', async () => {
		withObjects([]);
		const response = await GET(event);

		expect(await response.json()).toEqual({ available: false });
	});

	it('reports unavailable when the latest/ folder has no key matching the expected filename pattern', async () => {
		withObjects([{ key: 'android/latest/README.txt' }]);
		const response = await GET(event);

		expect(await response.json()).toEqual({ available: false });
	});

	it('reports unavailable when the R2 bucket is not configured', async () => {
		withObjects(undefined);
		const response = await GET(event);

		expect(await response.json()).toEqual({ available: false });
	});

	it('reuses the cached result instead of listing the bucket again', async () => {
		withObjects([{ key: 'android/latest/acerola-1.0.15.apk' }]);
		await GET(event);
		withObjects([]);
		const response = await GET(event);

		expect(await response.json()).toEqual({ available: true });
		expect(getApkBucket).toHaveBeenCalledTimes(1);
	});
});
