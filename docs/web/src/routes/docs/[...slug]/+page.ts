import { error } from '@sveltejs/kit';
import { getDoc, getPrevNext } from '#lib/content/docs.ts';
import { getLocale } from '#lib/paraglide/runtime.js';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
	const locale = getLocale();
	const doc = getDoc(locale, params.slug);

	// The error screen never shows this message: it is for logs only.
	if (!doc) throw error(404, 'Doc page not found');

	return {
		Doc: doc.component,
		frontmatter: doc.frontmatter,
		raw: doc.raw,
		...getPrevNext(locale, doc.slug)
	};
};
