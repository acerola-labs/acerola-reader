import { AwsClient } from 'aws4fetch';
import { env } from '$env/dynamic/private';
import type { ApkBucket } from '#lib/server/apk-bucket.ts';

// The R2 bucket through its S3-compatible API: the site runs as a Node server (hetzner), so the
// Worker binding is gone. The credentials come from an R2 API token scoped to this bucket, read
// at runtime — without them the APK routes behave as if no bucket were configured.
export function getApkBucket(): ApkBucket | undefined {
	const { R2_ENDPOINT, R2_BUCKET, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY } = env;
	if (!R2_ENDPOINT || !R2_BUCKET || !R2_ACCESS_KEY_ID || !R2_SECRET_ACCESS_KEY) return undefined;

	const client = new AwsClient({
		accessKeyId: R2_ACCESS_KEY_ID,
		secretAccessKey: R2_SECRET_ACCESS_KEY,
		service: 's3',
		region: 'auto'
	});

	return {
		async list({ prefix }) {
			const url = new URL(`${R2_ENDPOINT.replace(/\/$/, '')}/${R2_BUCKET}`);
			url.searchParams.set('list-type', '2');
			url.searchParams.set('prefix', prefix);

			const response = await client.fetch(url);
			if (!response.ok) throw new Error(`R2 list failed: HTTP ${response.status}`);

			return { objects: parseListKeys(await response.text()) };
		}
	};
}

// ListObjectsV2 answers in XML; only the object keys matter here. Keys are XML-escaped.
export function parseListKeys(xml: string): { key: string }[] {
	return [...xml.matchAll(/<Key>([^<]*)<\/Key>/g)].map((match) => ({
		key: match[1]
			.replaceAll('&lt;', '<')
			.replaceAll('&gt;', '>')
			.replaceAll('&quot;', '"')
			.replaceAll('&apos;', "'")
			.replaceAll('&amp;', '&')
	}));
}
