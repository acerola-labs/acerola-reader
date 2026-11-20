import InboxIcon from '@lucide/svelte/icons/inbox';
import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AcerolaEmptyState from './acerola-empty-state.svelte';

describe('AcerolaEmptyState', () => {
	it('renders the title and the hint', () => {
		render(AcerolaEmptyState, {
			props: { data: { title: 'Nothing here', hint: 'Pick a folder', icon: InboxIcon } }
		});
		expect(screen.getByText('Nothing here')).toBeInTheDocument();
		expect(screen.getByText('Pick a folder')).toBeInTheDocument();
	});

	it('shows the mascot when there is no icon', () => {
		const { container } = render(AcerolaEmptyState, { props: { data: { title: 'Empty' } } });
		expect(screen.getByText('Empty')).toBeInTheDocument();
		expect(container.querySelector('img')?.getAttribute('src')).toBe('/brand/acerola-mascot.svg');
	});
});
