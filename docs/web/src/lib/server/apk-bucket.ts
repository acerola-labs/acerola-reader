// Only the method the /api/apk-latest and /api/apk-status resolvers use — avoids pulling in
// all of @cloudflare/workers-types for a single binding.
export interface ApkBucket {
	list(options: { prefix: string }): Promise<{ objects: { key: string }[] }>;
}

// android-release.yml uploads each prod release to `android/releases/acerola-{version}.apk`
// (permanent file) and also to `android/latest/acerola-{version}.apk`, without deleting the
// previous version from that second folder. `latest/` may therefore hold more than one APK for
// a while — always resolve the highest version instead of assuming a fixed key like `latest.apk`.
export const LATEST_PREFIX = 'android/latest/';
const KEY_PATTERN = /^android\/latest\/acerola-(\d+\.\d+\.\d+)\.apk$/;

function compareVersions(a: string, b: string): number {
	const partsA = a.split('.').map(Number);
	const partsB = b.split('.').map(Number);

	for (let i = 0; i < 3; i++) {
		if (partsA[i] !== partsB[i]) return partsA[i] - partsB[i];
	}

	return 0;
}

// Returns the key of the highest-version APK in `android/latest/`, or null if the folder
// is empty (or only holds files that don't match the expected name pattern).
export async function resolveLatestApkKey(bucket: ApkBucket): Promise<string | null> {
	const listed = await bucket.list({ prefix: LATEST_PREFIX });
	const versions = listed.objects
		.map((object) => object.key.match(KEY_PATTERN)?.[1])
		.filter((version): version is string => version !== undefined);

	if (versions.length === 0) return null;

	const latest = versions.sort(compareVersions).at(-1) as string;
	return `${LATEST_PREFIX}acerola-${latest}.apk`;
}
