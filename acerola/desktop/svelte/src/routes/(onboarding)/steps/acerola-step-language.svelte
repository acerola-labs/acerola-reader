<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { Button } from '#lib/components/ui/button/index.ts';
	import * as Select from '#lib/components/ui/select/index.ts';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import { locales } from '#lib/paraglide/runtime.js';
	import { setLocale, getLocale } from '#lib/paraglide/runtime.js';
	import type { Locale } from '#lib/paraglide/runtime.js';

	import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
	import GlobeIcon from '@lucide/svelte/icons/globe';

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
		<div class="flex flex-col items-center gap-3 sm:flex-row sm:items-start">
			<div
				class="flex size-14 items-center justify-center rounded-inner bg-chart-3/10 text-chart-3"
			>
				<LanguagesIcon size={28} />
			</div>
			<div>
				<h1 class="text-3xl font-bold tracking-tight text-foreground sm:text-4xl">
					{m['onboarding.language.title']()}
				</h1>
				<p class="mt-1 text-sm text-muted-foreground">{m['onboarding.language.desc']()}</p>
			</div>
		</div>

		<!-- Visual Language Cards (VibePrompts Role/Selection Picker inspired) -->
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
			{#each localeOptions as opt}
				{@const isSelected = selectedLocale === opt.value}
				<button
					type="button"
					onclick={() => handleLocaleChange(opt.value)}
					class="group relative flex cursor-pointer items-start justify-between rounded-inner border p-5 text-left backdrop-blur-sm transition-all duration-200 {isSelected
						? ' bg-linear-to-b from-primary/10 via-card/70 to-card/90 shadow-lift ring-2 ring-primary/40'
						: 'border-border/40 bg-card/40 hover:border-border hover:bg-card/70'}"
				>
					<div class="space-y-2">
						<span class="text-3xl">{opt.flag}</span>
						<div>
							<h3 class="text-lg font-bold text-foreground">{opt.nativeName}</h3>
							<p class="text-xs text-muted-foreground">{opt.sub}</p>
						</div>
					</div>

					<div
						class="flex size-6 items-center justify-center rounded-full transition-all {isSelected
							? 'bg-primary text-primary-foreground'
							: 'border border-border/80 bg-background/50 text-transparent'}"
					>
						<CheckCircle2Icon size={16} strokeWidth={2.5} />
					</div>
				</button>
			{/each}
		</div>

		<!-- Dropdown Alternative -->
		<div class="rounded-inner border border-border/40 bg-card/30 p-4 backdrop-blur-sm">
			<div class="mb-2 flex items-center gap-2 text-xs font-medium text-muted-foreground">
				<GlobeIcon size={14} />
				<span>{m['onboarding.language.quickSelect']()}</span>
			</div>
			<Select.Root type="single" value={selectedLocale} onValueChange={handleLocaleChange}>
				<Select.Trigger class="w-full rounded-inner">
					{localeOptions.find((l) => l.value === selectedLocale)?.label || selectedLocale}
				</Select.Trigger>
				<Select.Content>
					{#each localeOptions as option}
						<Select.Item value={option.value} label={option.label}>
							{option.label}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<!-- Action Controls -->
		<div class="flex items-center justify-between pt-2">
			<Button onclick={onPrev} variant="outline" class="rounded-inner px-6">
				{m['onboarding.back']()}
			</Button>
			<Button
				onclick={onNext}
				class="rounded-inner px-8 font-semibold shadow-lift transition-all hover:shadow-lift"
			>
				{m['onboarding.next']()}
			</Button>
		</div>
	</div>
</div>
