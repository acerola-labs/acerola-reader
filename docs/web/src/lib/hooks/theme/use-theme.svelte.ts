import { browser } from '$app/environment';
import { THEME_COOKIE, watchCookie, writeCookie } from '#lib/utils/preference.util.ts';

export type ThemeMode = 'light' | 'dark';

/** Dark is the brand default; app.html applies the shared cookie before the first paint. */
let mode = $state<ThemeMode>('dark');

function applyTheme() {
	if (!browser) return;

	const root = document.documentElement;
	root.setAttribute('data-theme', mode);
	root.classList.toggle('dark', mode === 'dark');
}

if (browser) {
	mode = document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
	// Switched on another tab or Acerola site: this page follows right away.
	watchCookie(THEME_COOKIE, (value) => {
		mode = value === 'light' ? 'light' : 'dark';
		applyTheme();
	});
}

/** Page theme, shared by every Acerola site through the acerola-theme cookie. */
export function useTheme() {
	function setMode(newMode: ThemeMode) {
		mode = newMode;
		writeCookie(THEME_COOKIE, newMode);
		applyTheme();
	}

	return {
		setMode,
		toggle: () => setMode(mode === 'dark' ? 'light' : 'dark'),
		get mode() {
			return mode;
		}
	};
}
