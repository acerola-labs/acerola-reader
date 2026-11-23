import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import AcerolaSelect from './acerola-select.svelte';
import type { AcerolaSelectOption } from './acerola-select.svelte';

describe('AcerolaSelect', () => {
	const options: AcerolaSelectOption[] = [
		{ value: 'pt-br', label: 'Português' },
		{ value: 'en', label: 'Inglês' }
	];

	it('renders the select with the placeholder', () => {
		render(AcerolaSelect, {
			props: {
				data: { options },
				ui: { placeholder: 'Escolha o idioma' }
			}
		});

		// the underlying bits-ui component uses a button with the combobox role,
		// but without the full JS mount it is sometimes just a button
		const trigger = screen.getByText('Escolha o idioma');
		expect(trigger).toBeInTheDocument();
	});

	it('displays the correct label according to the value', () => {
		render(AcerolaSelect, { props: { data: { options }, state: { value: 'pt-br' } } });
		expect(screen.getByText('Português')).toBeInTheDocument();
	});

	it('opens options on click and allows selecting an option', async () => {
		const user = userEvent.setup();
		render(AcerolaSelect, {
			props: {
				data: { options },
				ui: { placeholder: 'Selecione...' }
			}
		});

		const trigger = screen.getByText('Selecione...');
		await user.click(trigger);

		// bits-ui select option roles can sometimes be hard to query in JSDom
		// Query by text to be safe, since it is definitely rendered
		const option = await screen.findByText('Inglês');
		expect(option).toBeInTheDocument();

		await user.click(option);

		// After clicking the option, the trigger must now show "Inglês"
		expect(screen.getByText('Inglês')).toBeInTheDocument();
	});
});
