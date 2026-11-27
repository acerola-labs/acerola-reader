import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { useTheme } from '#lib/hooks/theme/use-theme.svelte.ts';
import { readCookie, THEME_COOKIE } from '#lib/utils/preference.util.ts';
import AcerolaThemePicker from './acerola-theme-picker.svelte';

// useTheme() has module-level singleton $state, so it can be called directly like a plain
// function; state leaks across tests in this file unless it's reset here.
const themeCtx = useTheme();

beforeEach(() => {
	themeCtx.setMode('dark');
});

describe('useTheme', () => {
	it('defaults to dark mode', () => {
		expect(themeCtx.mode).toBe('dark');
		expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
		expect(document.documentElement.classList.contains('dark')).toBe(true);
	});

	it('setMode applies the theme and shares it with every Acerola site', () => {
		themeCtx.setMode('light');

		expect(themeCtx.mode).toBe('light');
		expect(readCookie(THEME_COOKIE)).toBe('light');
		expect(document.documentElement.getAttribute('data-theme')).toBe('light');
		expect(document.documentElement.classList.contains('dark')).toBe(false);
	});
});

describe('AcerolaThemePicker', () => {
	it('switches dark -> light -> dark on repeated clicks', async () => {
		render(AcerolaThemePicker);
		const user = userEvent.setup();
		const button = screen.getByRole('button', { name: 'Mudar tema' });

		await user.click(button);
		expect(themeCtx.mode).toBe('light');

		await user.click(button);
		expect(themeCtx.mode).toBe('dark');
	});
});
