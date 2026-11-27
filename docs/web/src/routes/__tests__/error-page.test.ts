import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import { m } from '#lib/paraglide/messages.js';
import ErrorPage from '../+error.svelte';

vi.mock('$app/state', () => ({ page: { status: 404 } }));

describe('+error page', () => {
	it('renders the not found screen with a link back home', () => {
		render(ErrorPage);

		expect(
			screen.getByRole('heading', { level: 1, name: m['error.notFound.title']() })
		).toBeInTheDocument();
		expect(screen.getByText(m['error.eyebrow']({ status: 404 }))).toBeInTheDocument();
		const homeLink = screen.getByRole('link', { name: m['error.actions.home']() });
		expect(homeLink).toHaveAttribute('href', '/pt-br/');
	});
});
