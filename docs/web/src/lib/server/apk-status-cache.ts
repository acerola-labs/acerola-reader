// In-process cache for /api/apk-status: a single result (there is nothing per-request to vary
// on), kept for `ttlMs`. A failed check is not cached, so the next visit retries.
let cached: { available: boolean; expiresAt: number } | undefined;

export async function cachedAvailability(
	ttlMs: number,
	check: () => Promise<boolean>,
	now: () => number = Date.now
): Promise<boolean> {
	if (cached && cached.expiresAt > now()) return cached.available;

	const available = await check();
	cached = { available, expiresAt: now() + ttlMs };
	return available;
}

export function clearAvailabilityCache(): void {
	cached = undefined;
}
