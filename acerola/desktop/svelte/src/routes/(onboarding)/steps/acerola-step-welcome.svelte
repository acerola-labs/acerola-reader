<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { Button } from '#lib/components/ui/button/index.ts';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ZapIcon from '@lucide/svelte/icons/zap';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';

	let { onNext } = $props<{ onNext: () => void }>();
</script>

<div class="flex h-full w-full items-center justify-center p-6 sm:p-8">
	<div class="flex max-w-lg flex-col items-center space-y-8 text-center">
		<!-- Brand moment: the mascot welcomes (local asset, the app does not depend on the CDN). -->
		<img src="/brand/acerola-mascot.svg" alt="" class="size-36" />


		<!-- Title & Subtitle -->
		<div class="space-y-3">
			<h1 class="text-4xl font-black tracking-tight text-foreground sm:text-5xl">
				Acerola
			</h1>
			<p class="mx-auto max-w-md text-base leading-relaxed text-muted-foreground sm:text-lg">
				{m['onboarding.welcome']()}
			</p>
		</div>

		<!-- Feature Highlight Chips -->
		<div class="grid w-full grid-cols-1 gap-2.5 sm:grid-cols-3">
			<div
				class="flex items-center gap-2.5 rounded-inner border border-border/50 bg-card/60 p-3 text-left backdrop-blur-sm transition-all"
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-inner bg-primary/10 text-primary">
					<ZapIcon size={16} />
				</div>
				<div class="min-w-0">
					<p class="text-xs font-semibold text-foreground">{m['onboarding.welcomeBadgeFastTitle']()}</p>
					<p class="truncate text-[10px] text-muted-foreground">{m['onboarding.welcomeBadgeFastDesc']()}</p>
				</div>
			</div>

			<div
				class="flex items-center gap-2.5 rounded-inner border border-border/50 bg-card/60 p-3 text-left backdrop-blur-sm transition-all"
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-inner bg-chart-3/10 text-chart-3">
					<Share2Icon size={16} />
				</div>
				<div class="min-w-0">
					<p class="text-xs font-semibold text-foreground">{m['onboarding.welcomeBadgeP2pTitle']()}</p>
					<p class="truncate text-[10px] text-muted-foreground">{m['onboarding.welcomeBadgeP2pDesc']()}</p>
				</div>
			</div>

			<div
				class="flex items-center gap-2.5 rounded-inner border border-border/50 bg-card/60 p-3 text-left backdrop-blur-sm transition-all"
			>
				<div class="flex size-8 shrink-0 items-center justify-center rounded-inner bg-chart-2/10 text-chart-2">
					<ShieldCheckIcon size={16} />
				</div>
				<div class="min-w-0">
					<p class="text-xs font-semibold text-foreground">{m['onboarding.welcomeBadgeLocalTitle']()}</p>
					<p class="truncate text-[10px] text-muted-foreground">{m['onboarding.welcomeBadgeLocalDesc']()}</p>
				</div>
			</div>
		</div>

		<!-- Action Button -->
		<Button
			onclick={onNext}
			size="lg"
			class="group relative inline-flex items-center gap-2 rounded-inner bg-primary px-9 py-6 text-base font-semibold text-primary-foreground shadow-lift transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
		>
			<span>{m['onboarding.start']()}</span>
			<ArrowRightIcon size={18} class="transition-transform duration-200 group-hover:translate-x-1" />
		</Button>
	</div>
</div>
