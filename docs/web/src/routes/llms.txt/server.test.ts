import { describe, expect, it } from 'vitest';
import { SITE_URL } from '#lib/constants/site.ts';
import { getFlatOrder } from '#lib/content/docs.ts';
import { GET } from './+server';

const textAt = async (path: string) => {
	const event = { url: new URL(`${SITE_URL}${path}`) } as unknown as Parameters<typeof GET>[0];
	return (await GET(event)).text();
};

describe('GET /llms.txt', () => {
	it('is an index of the per-locale files', async () => {
		const body = await textAt('/llms.txt');

		expect(body).toContain(`(${SITE_URL}/pt-br/llms.txt)`);
		expect(body).toContain(`(${SITE_URL}/en/llms.txt)`);
	});

	it('lists every doc page of its locale, under its prefix', async () => {
		const body = await textAt('/en/llms.txt');

		for (const doc of getFlatOrder('en')) {
			expect(body).toContain(`[${doc.frontmatter.title}](${SITE_URL}/en/docs/${doc.slug})`);
		}
	});
});
