<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import InboxIcon from '@lucide/svelte/icons/inbox';
	import AcerolaEmptyState from './acerola-empty-state.svelte';

	const { Story } = defineMeta({
		title: 'Components/AcerolaEmptyState',
		component: AcerolaEmptyState,
		tags: ['autodocs']
	});
</script>

<Story name="Mascot" asChild>
	<AcerolaEmptyState data={{ title: 'Sua biblioteca está vazia', hint: 'Escolha uma pasta com seus quadrinhos.' }} />
</Story>

<Story name="Icon" asChild>
	<AcerolaEmptyState data={{ title: 'Nenhum aparelho', hint: 'Pareie um aparelho na mesma rede.', icon: InboxIcon }} />
</Story>
