<script module lang="ts">
	import type { Component } from 'svelte';

	export type FeatureItem = {
		icon?: Component;
		title: string;
		description: string;
	};

	export type CodeTab = {
		id: string;
		label: string;
		filename?: string;
		code: string;
	};

	export type MarketingFeatureCodeProps = {
		eyebrow?: string;
		title?: string;
		description?: string;
		features: FeatureItem[];
		tabs: CodeTab[];
		class?: string;
	};
</script>

<script lang="ts">
	import TerminalIcon from '@lucide/svelte/icons/terminal';
	import AcerolaCode from '#lib/components/acerola-code/acerola-code.svelte';
	import { cn } from '#lib/utils/cn.util.ts';

	let {
		eyebrow,
		title,
		description,
		features = [],
		tabs = [],
		class: className
	}: MarketingFeatureCodeProps = $props();
</script>

<!-- Acerola standard: features as plain rows (round icon in primary-soft + font-black title) next
     to the brand code panel (AcerolaCode). The header is optional: inside a page section, the
     section already carries the eyebrow and the title. -->
<div class={cn('grid gap-6', className)}>
	{#if title}
		<header>
			{#if eyebrow}
				<p class="text-xs font-semibold text-muted-foreground">{eyebrow}</p>
			{/if}
			<h2 class="mt-1 text-3xl leading-none font-black tracking-tighter">{title}</h2>
			{#if description}
				<p class="mt-3 max-w-2xl text-pretty text-muted-foreground">{description}</p>
			{/if}
		</header>
	{/if}

	<div class="grid grid-cols-1 items-start gap-6 lg:grid-cols-12">
		<ul class="m-0 grid list-none gap-3 p-0 lg:col-span-5">
			{#each features as feat (feat.title)}
				<li class="flex items-start gap-4 rounded-tile bg-card p-5 shadow-lift">
					<span
						class="flex size-11 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary"
					>
						{#if feat.icon}
							<feat.icon size={20} />
						{:else}
							<TerminalIcon size={20} />
						{/if}
					</span>
					<div class="grid min-w-0 gap-1">
						<h3 class="m-0 font-black tracking-tight text-base">{feat.title}</h3>
						<p class="m-0 text-sm text-pretty text-muted-foreground">
							{feat.description}
						</p>
					</div>
				</li>
			{/each}
		</ul>

		<div class="min-w-0 lg:col-span-7">
			<AcerolaCode
				data={{
					tabs: tabs.map((tab) => ({ id: tab.id, label: tab.label, code: tab.code }))
				}}
			/>
		</div>
	</div>
</div>
