<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ZapIcon from '@lucide/svelte/icons/zap';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';

	let { onNext } = $props<{ onNext: () => void }>();

	const highlights = $derived([
		{
			icon: ZapIcon,
			tone: 'bg-primary-soft text-primary',
			title: m['onboarding.welcomeBadgeFastTitle'](),
			desc: m['onboarding.welcomeBadgeFastDesc']()
		},
		{
			icon: Share2Icon,
			tone: 'bg-info-soft text-info',
			title: m['onboarding.welcomeBadgeP2pTitle'](),
			desc: m['onboarding.welcomeBadgeP2pDesc']()
		},
		{
			icon: ShieldCheckIcon,
			tone: 'bg-success-soft text-success',
			title: m['onboarding.welcomeBadgeLocalTitle'](),
			desc: m['onboarding.welcomeBadgeLocalDesc']()
		}
	]);
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="flex max-w-3xl flex-col items-center space-y-8 text-center">
		<!-- Brand moment: the mascot welcomes (local asset, the app does not depend on the CDN). -->
		<img src="/brand/acerola-mascot.svg" alt="" class="size-36" />

		<div class="space-y-3">
			<!-- Brand lettering (Titan One is allowed on the welcome/splash). -->
			<h1 class="font-display text-5xl text-foreground sm:text-6xl">Acerola</h1>
			<p class="mx-auto max-w-md text-base leading-relaxed text-muted-foreground sm:text-lg">
				{m['onboarding.welcome']()}
			</p>
		</div>

		<div class="grid w-full grid-cols-1 gap-3 sm:grid-cols-3">
			{#each highlights as item (item.title)}
				<div class="flex items-center gap-3 rounded-tile bg-surface p-3 text-left">
					<span
						class={[
							'flex size-10 shrink-0 items-center justify-center rounded-full',
							item.tone
						]}
					>
						<item.icon size={18} />
					</span>
					<div class="min-w-0">
						<p class="text-sm font-black tracking-tight text-foreground">
							{item.title}
						</p>
						<p class="text-xs text-muted-foreground">
							{item.desc}
						</p>
					</div>
				</div>
			{/each}
		</div>

		<AcerolaButton ui={{ size: 'lg', class: 'group gap-2 px-8' }} events={{ onClick: onNext }}>
			<span>{m['onboarding.start']()}</span>
			<ArrowRightIcon
				size={18}
				class="transition-transform duration-200 ease-brand group-hover:translate-x-1"
			/>
		</AcerolaButton>
	</div>
</div>
