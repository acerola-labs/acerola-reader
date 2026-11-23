<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import { locales } from '#lib/paraglide/runtime.js';
	import { setLocale, getLocale } from '#lib/paraglide/runtime.js';
	import type { Locale } from '#lib/paraglide/runtime.js';

	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';

	let selectedLocale = $state<Locale>(getLocale());

	const localeOptions = $derived(
		locales.map((locale) => ({
			value: locale,
			label:
				locale === 'pt-br'
					? 'Português (BR)'
					: locale === 'en'
						? 'English'
						: (locale as string).toUpperCase(),
			flag: locale === 'pt-br' ? '🇧🇷' : '🇺🇸',
			nativeName: locale === 'pt-br' ? 'Português' : 'English',
			sub:
				locale === 'pt-br'
					? m['onboarding.language.countryBr']()
					: m['onboarding.language.countryUs']()
		}))
	);

	function handleLocaleChange(locale: string) {
		selectedLocale = locale as Locale;
		setLocale(selectedLocale);
	}

	let { onNext, onPrev } = $props<{ onNext: () => void; onPrev: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="w-full max-w-xl space-y-8 text-center sm:text-left">
		<!-- Header -->
		<div class="flex flex-col items-center gap-3 sm:flex-row sm:items-center">
			<span
				class="flex size-14 shrink-0 items-center justify-center rounded-full bg-info-soft text-info"
			>
				<LanguagesIcon size={24} />
			</span>
			<div>
				<h1 class="text-3xl font-black tracking-tight text-foreground sm:text-4xl">
					{m['onboarding.language.title']()}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground">{m['onboarding.language.desc']()}</p>
			</div>
		</div>

		<!-- Language cards: the selected one goes to the primary tone (no colored border). -->
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
			{#each localeOptions as opt (opt.value)}
				{@const isSelected = selectedLocale === opt.value}
				<button
					type="button"
					aria-pressed={isSelected}
					onclick={() => handleLocaleChange(opt.value)}
					class={[
						'group relative flex cursor-pointer items-start justify-between rounded-card p-5 text-left shadow-lift transition-colors duration-200 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
						isSelected ? 'bg-primary-soft text-primary' : 'bg-card hover:bg-surface'
					]}
				>
					<div class="space-y-2">
						<span class="text-3xl">{opt.flag}</span>
						<div>
							<h3
								class={[
									'text-lg font-black tracking-tight',
									isSelected ? 'text-primary' : 'text-foreground'
								]}
							>
								{opt.nativeName}
							</h3>
							<p class="text-xs text-muted-foreground">{opt.sub}</p>
						</div>
					</div>

					<span
						class={[
							'flex size-7 items-center justify-center rounded-full transition-colors',
							isSelected
								? 'bg-primary text-primary-foreground'
								: 'bg-surface text-transparent'
						]}
					>
						<CheckCircle2Icon size={16} strokeWidth={2.5} />
					</span>
				</button>
			{/each}
		</div>

		<!-- Action Controls -->
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
