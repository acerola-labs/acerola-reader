import { json } from '@sveltejs/kit';
import { resolveLatestApkKey } from '#lib/server/apk-bucket.ts';
import type { RequestHandler } from './$types';

// Cached at the Cloudflare edge for a few minutes: the live check actually runs only once per
// window, per PoP — the rest of the visits in that window reuse the result without any new
// `list` on the R2 bucket.
const CACHE_TTL_SECONDS = 300;

// Checked directly through the R2 binding (`APK_BUCKET`), not over HTTP: a `fetch` from the
// worker to one of its own routes (self-fetch) proved unreliable on the Cloudflare runtime — the
// binding removes that network round-trip and the retry/timeout logic it required.
export const GET: RequestHandler = async ({ platform }) => {
	// `caches.default` (Cloudflare's Cache API) only exists in a real Worker runtime — in
	// tests/local dev it falls straight through to the uncached live check without breaking
	// anything. TS's default `lib dom` doesn't know this Cloudflare-specific extension, hence the
	// local cast instead of pulling in all of @cloudflare/workers-types just for this.
	const edgeCache =
		typeof caches !== 'undefined'
			? (caches as CacheStorage & { default: Cache }).default
			: undefined;
	// Fixed key (doesn't depend on the caller's request): there is only ONE possible result
	// for this route — nothing per-user/per-query-string to vary the cache on.
	const cacheKey = new Request('https://internal-cache-key.acerola.app/api/apk-status');

	if (edgeCache) {
		const cached = await edgeCache.match(cacheKey);
		if (cached) return cached;
	}

	const bucket = platform?.env?.APK_BUCKET;
	const available = bucket !== undefined && (await resolveLatestApkKey(bucket)) !== null;

	const response = json(
		{ available },
		{ headers: { 'Cache-Control': `public, max-age=${CACHE_TTL_SECONDS}` } }
	);

	if (edgeCache) {
		await edgeCache.put(cacheKey, response.clone());
	}

	return response;
};
