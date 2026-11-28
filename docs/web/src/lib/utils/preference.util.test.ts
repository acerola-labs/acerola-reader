import { afterEach, describe, expect, it, vi } from 'vitest';
import {
	cookieDomain,
	LOCALE_COOKIE,
	readCookie,
	THEME_COOKIE,
	watchCookie,
	writeCookie
} from './preference.util';

describe('watchCookie', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		writeCookie(THEME_COOKIE, 'dark');
	});

	it('reports the new value when another site changes the cookie', () => {
		const cookieStore = new EventTarget();
		vi.stubGlobal('cookieStore', cookieStore);
		writeCookie(THEME_COOKIE, 'dark');
		const onChange = vi.fn();
		watchCookie(THEME_COOKIE, onChange);

		writeCookie(THEME_COOKIE, 'light');
		cookieStore.dispatchEvent(new Event('change'));

		expect(onChange).toHaveBeenCalledExactlyOnceWith('light');
	});

	it('reads again when the tab comes back, and stays quiet when nothing changed', () => {
		writeCookie(THEME_COOKIE, 'dark');
		const onChange = vi.fn();
		const stop = watchCookie(THEME_COOKIE, onChange);

		document.dispatchEvent(new Event('visibilitychange'));
		expect(onChange).not.toHaveBeenCalled();

		writeCookie(THEME_COOKIE, 'light');
		window.dispatchEvent(new Event('focus'));
		expect(onChange).toHaveBeenCalledExactlyOnceWith('light');
		stop();
	});

	it('stops watching', () => {
		writeCookie(THEME_COOKIE, 'dark');
		const onChange = vi.fn();
		watchCookie(THEME_COOKIE, onChange)();

		writeCookie(THEME_COOKIE, 'light');
		document.dispatchEvent(new Event('visibilitychange'));

		expect(onChange).not.toHaveBeenCalled();
	});
});

describe('readCookie', () => {
	it('finds a cookie among others', () => {
		expect(readCookie('acerola-theme', 'a=1; acerola-theme=light; b=2')).toBe('light');
	});

	it('does not match a cookie whose name only ends the same', () => {
		expect(readCookie('acerola-theme', 'x-acerola-theme=light')).toBeUndefined();
	});
});

describe('cookieDomain', () => {
	it('shares the cookie across acerola.app and its subdomains', () => {
		expect(cookieDomain('acerola.app')).toBe('acerola.app');
		expect(cookieDomain('docs.reader.acerola.app')).toBe('acerola.app');
	});

	it('keeps the cookie on the exact host elsewhere', () => {
		expect(cookieDomain('localhost')).toBeUndefined();
		expect(cookieDomain('notacerola.app')).toBeUndefined();
	});
});

describe('writeCookie', () => {
	it('saves a value readable by every page', () => {
		writeCookie(LOCALE_COOKIE, 'en');
		expect(readCookie(LOCALE_COOKIE)).toBe('en');
	});
});
