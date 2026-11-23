import { error, redirect } from '@sveltejs/kit';
import { APK_BINARY_BASE_URL } from '#lib/constants/site.ts';
import { resolveLatestApkKey } from '#lib/server/apk-bucket.ts';
import { getApkBucket } from '#lib/server/r2.ts';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async () => {
	const bucket = getApkBucket();
	if (!bucket) error(500, 'APK bucket not configured');

	const key = await resolveLatestApkKey(bucket);
	if (key === null) error(404, 'No APK available');

	redirect(302, `${APK_BINARY_BASE_URL}/${key}`);
};
