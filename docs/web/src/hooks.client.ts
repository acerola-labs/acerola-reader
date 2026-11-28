import { defineCustomClientStrategy } from '#lib/paraglide/runtime.js';
import { LOCALE_COOKIE, readCookie, writeCookie } from '#lib/utils/preference.util.ts';

// The language follows the person across every Acerola site: setLocale() saves it in the shared
// cookie, then the url strategy navigates to the page in that language.
defineCustomClientStrategy('custom-acerola', {
	getLocale: () => readCookie(LOCALE_COOKIE),
	setLocale: (locale) => writeCookie(LOCALE_COOKIE, locale)
});
