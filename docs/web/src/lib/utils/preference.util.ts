/**
 * Theme and language shared by every Acerola site (acerola-design skill, section 11.1): the
 * cookies live on the whole acerola.app domain, so switching to light on brand.acerola.app also
 * turns docs.reader.acerola.app light. app.html reads the same cookies inline before the first
 * paint; keep the names in sync there.
 */
export const THEME_COOKIE = 'acerola-theme';
export const LOCALE_COOKIE = 'acerola-locale';

const SHARED_DOMAIN = 'acerola.app';

/** 400 days: the longest max-age browsers keep. */
const MAX_AGE = 34560000;

export function readCookie(name: string, cookies = document.cookie): string | undefined {
	const match = cookies.match(new RegExp(`(?:^|;\\s*)${name}=([^;]*)`));
	return match ? decodeURIComponent(match[1]) : undefined;
}

/** The whole acerola.app on its own sites; the exact host anywhere else (localhost, previews). */
export function cookieDomain(hostname = location.hostname): string | undefined {
	return hostname === SHARED_DOMAIN || hostname.endsWith(`.${SHARED_DOMAIN}`)
		? SHARED_DOMAIN
		: undefined;
}

/**
 * Calls onChange as soon as the cookie changes, even when another tab or another Acerola site
 * changed it: the cookieStore change event where the browser has it, and a new read whenever the
 * tab comes back to the front. Returns the function that stops watching.
 */
export function watchCookie(name: string, onChange: (value: string | undefined) => void) {
	let last = readCookie(name);
	const check = () => {
		const value = readCookie(name);
		if (value === last) return;
		last = value;
		onChange(value);
	};

	const store = (globalThis as unknown as { cookieStore?: EventTarget }).cookieStore;
	store?.addEventListener('change', check);
	document.addEventListener('visibilitychange', check);
	window.addEventListener('focus', check);

	return () => {
		store?.removeEventListener('change', check);
		document.removeEventListener('visibilitychange', check);
		window.removeEventListener('focus', check);
	};
}

/** Saves the value for every Acerola site. */
export function writeCookie(name: string, value: string, hostname = location.hostname) {
	const domain = cookieDomain(hostname);
	document.cookie = [
		`${name}=${encodeURIComponent(value)}`,
		'path=/',
		`max-age=${MAX_AGE}`,
		'samesite=lax',
		...(domain ? [`domain=${domain}`, 'secure'] : [])
	].join('; ');
}
