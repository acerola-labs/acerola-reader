import { json } from '@sveltejs/kit';
import { resolveLatestApkKey } from '#lib/server/apk-bucket.ts';
import { cachedAvailability } from '#lib/server/apk-status-cache.ts';
import { getApkBucket } from '#lib/server/r2.ts';
import type { RequestHandler } from './$types';

// The live check lists the R2 bucket, so it runs at most once per window on this server — the
// rest of the visits in that window reuse the result without any new `list` on the bucket.
const CACHE_TTL_SECONDS = 300;

export const GET: RequestHandler = async () => {
	const available = await cachedAvailability(CACHE_TTL_SECONDS * 1000, async () => {
		const bucket = getApkBucket();
		return bucket !== undefined && (await resolveLatestApkKey(bucket)) !== null;
	});

	return json(
		{ available },
		{ headers: { 'Cache-Control': `public, max-age=${CACHE_TTL_SECONDS}` } }
	);
};
