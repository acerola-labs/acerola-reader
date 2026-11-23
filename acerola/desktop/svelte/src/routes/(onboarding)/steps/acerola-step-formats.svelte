<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import FileArchiveIcon from '@lucide/svelte/icons/file-archive';
	import LayersIcon from '@lucide/svelte/icons/layers';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();

	// Each format keeps one tone: soft background + text of the tone (brand book state rule).
	const formats = $derived([
		{
			extension: 'CBZ',
			name: m['onboarding.formats.cbz.title'](),
			badge: m['onboarding.formats.cbz.badge'](),
			tone: 'bg-primary-soft text-primary',
			icon: FileArchiveIcon,
			description: m['onboarding.formats.cbz.desc'](),
			features: [
				m['onboarding.formats.cbz.feature1'](),
				m['onboarding.formats.cbz.feature2']()
			]
		},
		{
			extension: 'CBR',
			name: m['onboarding.formats.cbr.title'](),
			badge: m['onboarding.formats.cbr.badge'](),
			tone: 'bg-info-soft text-info',
			icon: LayersIcon,
			description: m['onboarding.formats.cbr.desc'](),
			features: [
				m['onboarding.formats.cbr.feature1'](),
				m['onboarding.formats.cbr.feature2']()
			]
		},
		{
			extension: 'PDF',
			name: m['onboarding.formats.pdf.title'](),
			badge: m['onboarding.formats.pdf.badge'](),
			tone: 'bg-warning-soft text-warning',
			icon: FileTextIcon,
			description: m['onboarding.formats.pdf.desc'](),
			features: [
				m['onboarding.formats.pdf.feature1'](),
				m['onboarding.formats.pdf.feature2']()
			]
		}
	]);
</script>

<div class="flex h-full w-full items-center justify-center overflow-y-auto p-6 sm:p-8">
	<div class="w-full max-w-3xl space-y-8">
		<div class="space-y-2 text-center">
			<span
				class="mb-2 inline-flex size-14 items-center justify-center rounded-full bg-primary-soft text-primary"
			>
				<FileArchiveIcon size={24} />
			</span>
			<h1 class="text-3xl font-black tracking-tight text-foreground">
				{m['onboarding.formats.title']()}
			</h1>
			<p class="mx-auto max-w-md text-sm text-muted-foreground">
				{m['onboarding.formats.desc']()}
			</p>
		</div>

		<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
			{#each formats as item (item.extension)}
				<div class="flex flex-col justify-between rounded-card bg-card p-5 shadow-lift">
					<div>
						<div class="mb-5 flex items-center justify-between">
							<span
								class={[
									'flex size-10 items-center justify-center rounded-full',
									item.tone
								]}
							>
								<item.icon size={18} />
							</span>
							<span
								class={['rounded-full px-3 py-1 text-[11px] font-black', item.tone]}
							>
								{item.badge}
							</span>
						</div>

						<h3 class="mb-1 text-3xl font-black tracking-tight text-foreground">
							{item.extension}
						</h3>
						<p class="mb-3 text-xs font-semibold text-muted-foreground">
							{item.name}
						</p>
						<p class="text-sm leading-relaxed text-muted-foreground">
							{item.description}
						</p>
					</div>

					<div class="mt-5 space-y-2 rounded-tile bg-surface p-3">
						{#each item.features as feat (feat)}
							<div
								class="flex items-center gap-2 text-xs font-semibold text-foreground"
							>
								<CheckCircle2Icon size={16} class="shrink-0 text-primary" />
								<span>{feat}</span>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>

		<div class="flex items-center gap-3 rounded-card bg-info-soft p-5 text-sm text-info">
			<SparklesIcon size={20} class="shrink-0" />
			<p class="leading-normal">
				<strong class="font-black">{m['onboarding.formats.syncNote.title']()}</strong>
				{m['onboarding.formats.syncNote.content']()}
			</p>
		</div>

		<div class="flex items-center justify-between pt-2">
			<AcerolaButton
				ui={{ variant: 'secondary', class: 'px-6' }}
				events={{ onClick: onPrev }}
			>
				{m['onboarding.back']()}
			</AcerolaButton>
			<AcerolaButton ui={{ class: 'px-8' }} events={{ onClick: onNext }}>
				{m['onboarding.next']()}
			</AcerolaButton>
		</div>
	</div>
</div>
