import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import DocsIndexPage from './+page.svelte';

describe('docs/+page', () => {
	it('renders the sidebar sections with links to each doc', () => {
		render(DocsIndexPage);

		// getSidebar('pt-br') runs against the repo's real markdown content — we only ensure
		// that at least one section with at least one doc link shows up, without coupling the
		// test to the exact set of articles (which changes as the documentation evolves).
		const links = screen.getAllByRole('link');
		expect(links.length).toBeGreaterThan(0);
		expect(links[0]).toHaveAttribute('href', expect.stringContaining('/docs/'));
	});
});
