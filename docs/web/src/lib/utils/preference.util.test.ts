import { describe, expect, it } from 'vitest';
import { cookieDomain, LOCALE_COOKIE, readCookie, writeCookie } from './preference.util';

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
