import { beforeEach, describe, expect, it, vi } from 'vitest';
import { getApkBucket } from '#lib/server/r2.ts';
import { GET } from './+server';

vi.mock('#lib/server/r2.ts', () => ({ getApkBucket: vi.fn() }));

function withObjects(objects: { key: string }[] | undefined) {
	vi.mocked(getApkBucket).mockReturnValue(
		objects === undefined ? undefined : { list: async () => ({ objects }) }
	);
}

const event = {} as Parameters<typeof GET>[0];

describe('GET /api/apk-latest', () => {
	beforeEach(() => {
		vi.mocked(getApkBucket).mockReset();
	});

	it('redirects to the only APK when latest/ has a single object', async () => {
		withObjects([{ key: 'android/latest/acerola-1.0.15.apk' }]);

		await expect(GET(event)).rejects.toMatchObject({
			status: 302,
			location: 'https://binary.reader.acerola.app/android/latest/acerola-1.0.15.apk'
		});
	});

	it('picks the highest semantic version when more than one APK lingers in latest/', async () => {
		withObjects([
			{ key: 'android/latest/acerola-1.0.9.apk' },
			{ key: 'android/latest/acerola-1.0.15.apk' },
			{ key: 'android/latest/acerola-1.2.0.apk' }
		]);

		await expect(GET(event)).rejects.toMatchObject({
			status: 302,
			location: 'https://binary.reader.acerola.app/android/latest/acerola-1.2.0.apk'
		});
	});

	it('ignores keys that do not match the expected filename pattern', async () => {
		withObjects([
			{ key: 'android/latest/README.txt' },
			{ key: 'android/latest/acerola-1.0.15.apk' }
		]);

		await expect(GET(event)).rejects.toMatchObject({
			status: 302,
			location: 'https://binary.reader.acerola.app/android/latest/acerola-1.0.15.apk'
		});
	});

	it('returns 404 when the latest/ folder is empty', async () => {
		withObjects([]);

		await expect(GET(event)).rejects.toMatchObject({ status: 404 });
	});

	it('returns 500 when the R2 bucket is not configured', async () => {
		withObjects(undefined);

		await expect(GET(event)).rejects.toMatchObject({ status: 500 });
	});
});
