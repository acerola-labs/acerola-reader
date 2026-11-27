import type { Handle } from '@sveltejs/kit';
import { defineCustomServerStrategy, getTextDirection } from '#lib/paraglide/runtime.js';
import { paraglideMiddleware } from '#lib/paraglide/server.js';
import { LOCALE_COOKIE, readCookie } from '#lib/utils/preference.util.ts';

// A request with no locale prefix that reaches the server (a page not prerendered) is sent to
// the locale in the shared cookie, like app.html does for the prerendered ones.
defineCustomServerStrategy('custom-acerola', {
	getLocale: (request) => readCookie(LOCALE_COOKIE, request?.headers.get('cookie') ?? '')
});

const handleParaglide: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;

		return resolve(event, {
			transformPageChunk: ({ html }) =>
				html
					.replace('%paraglide.lang%', locale)
					.replace('%paraglide.dir%', getTextDirection(locale))
		});
	});

export const handle: Handle = handleParaglide;
