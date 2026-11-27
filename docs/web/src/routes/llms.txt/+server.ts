import { GITHUB_URL, SITE_URL } from '#lib/constants/site.ts';
import { getFlatOrder } from '#lib/content/docs.ts';
import { m } from '#lib/paraglide/messages.js';
import {
	baseLocale,
	extractLocaleFromUrl,
	locales,
	localizeHref,
	type Locale
} from '#lib/paraglide/runtime.js';
import type { RequestHandler } from './$types';

export const prerender = true;

const text = (body: string) =>
	new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });

const LOCALE_NAMES: Record<Locale, string> = { 'pt-br': 'Português', en: 'English' };

/**
 * /llms.txt is the index of the per-locale files (/pt-br/llms.txt, /en/llms.txt); each one
 * describes the project and lists every doc page in its locale (llmstxt.org). The text comes
 * from the messages and the docs' frontmatter.
 */
export const GET: RequestHandler = ({ url }) => {
	const locale = extractLocaleFromUrl(url);
	const shown = locale ?? baseLocale;
	const head = [
		`# ${m['site.og.title']({}, { locale: shown })}`,
		'',
		`> ${m['site.description']({}, { locale: shown })}`,
		'',
		`- GitHub: ${GITHUB_URL}`,
		'- MPL-2.0',
		''
	].join('\n');

	if (!locale) {
		const files = locales
			.map(
				(l) =>
					`- [${LOCALE_NAMES[l]}](${SITE_URL}${localizeHref('/llms.txt', { locale: l })})`
			)
			.join('\n');
		return text(`${head}\n## ${m['llms.locales']({}, { locale: shown })}\n\n${files}\n`);
	}

	const docs = getFlatOrder(locale)
		.map((doc) => {
			const link = `[${doc.frontmatter.title}](${SITE_URL}${localizeHref(`/docs/${doc.slug}`, { locale })})`;
			return doc.frontmatter.description
				? `- ${link}: ${doc.frontmatter.description}`
				: `- ${link}`;
		})
		.join('\n');
	return text(`${head}\n## ${m['llms.docs']({}, { locale })}\n\n${docs}\n`);
};
