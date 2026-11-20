<script lang="ts">
	import { onMount } from 'svelte';
	import { gsap } from 'gsap';
	import { ScrollTrigger } from 'gsap/ScrollTrigger';
	import { scrollReveal, scrollScale } from '#lib/actions/index.ts';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import CloudOffIcon from '@lucide/svelte/icons/cloud-off';
	import CodeIcon from '@lucide/svelte/icons/code';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';
	import { Badge } from '#lib/components/ui/badge/index.ts';
	import { Button } from '#lib/components/ui/button/index.ts';
	import AcerolaApkDownloadButton from '#lib/components/acerola-apk-download-button/acerola-apk-download-button.svelte';
	import AcerolaCallout from '#lib/components/acerola-callout/acerola-callout.svelte';
	import AcerolaMicrosoftStoreButton from '#lib/components/acerola-microsoft-store-button/acerola-microsoft-store-button.svelte';
	import AcerolaShinyText from '#lib/components/acerola-shiny-text/acerola-shiny-text.svelte';
	import CardGrid from '#lib/mdsvex/card-grid.svelte';
	import PlatformCard from '#lib/mdsvex/platform-card.svelte';
	import Steps from '#lib/mdsvex/steps.svelte';
	import AcerolaMarketingFeatureCode, {
		type FeatureItem,
		type CodeTab
	} from '#lib/components/acerola-marketing-feature-code/acerola-marketing-feature-code.svelte';
	import GithubIcon from '#lib/icons/github.svelte';
	import { BRAND_MASCOT_URL, GITHUB_URL } from '#lib/constants/site.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { localizeHref } from '#lib/paraglide/runtime.js';

	let heroEl = $state<HTMLElement | null>(null);
	let heroContentEl = $state<HTMLDivElement | null>(null);

	onMount(() => {
		if (typeof window === 'undefined') return;
		if (window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches) return;
		if (!heroEl) return;

		const logo = heroEl.querySelector('.hero-logo');
		const kicker = heroEl.querySelector('.hero-kicker');
		const title = heroEl.querySelector('.hero-title');
		const subtitle = heroEl.querySelector('.hero-subtitle');
		const downloads = heroEl.querySelector('.hero-downloads');
		const ctas = heroEl.querySelector('.hero-ctas');
		const facts = heroEl.querySelector('.hero-facts');
		const scrollIndicator = heroEl.querySelector('.scroll-indicator');

		const tl = gsap.timeline({ defaults: { ease: 'power3.out' } });

		if (logo) {
			tl.fromTo(
				logo,
				{ opacity: 0, scale: 0.7, y: -15 },
				{ opacity: 1, scale: 1, y: 0, duration: 0.6, ease: 'back.out(1.6)' }
			);
		}
		if (kicker) {
			tl.fromTo(kicker, { opacity: 0, y: 15 }, { opacity: 1, y: 0, duration: 0.5 }, '-=0.35');
		}
		if (title) {
			tl.fromTo(
				title,
				{ opacity: 0, y: 25, scale: 0.96 },
				{ opacity: 1, y: 0, scale: 1, duration: 0.65 },
				'-=0.3'
			);
		}
		if (subtitle) {
			tl.fromTo(
				subtitle,
				{ opacity: 0, y: 15 },
				{ opacity: 1, y: 0, duration: 0.5 },
				'-=0.35'
			);
		}
		if (downloads) {
			tl.fromTo(
				downloads,
				{ opacity: 0, y: 20 },
				{ opacity: 1, y: 0, duration: 0.55 },
				'-=0.3'
			);
		}
		if (ctas) {
			tl.fromTo(ctas, { opacity: 0, y: 15 }, { opacity: 1, y: 0, duration: 0.45 }, '-=0.3');
		}
		if (facts) {
			tl.fromTo(
				facts,
				{ opacity: 0, scale: 0.92 },
				{ opacity: 1, scale: 1, duration: 0.5, ease: 'back.out(1.4)' },
				'-=0.25'
			);
		}
		if (scrollIndicator) {
			tl.fromTo(
				scrollIndicator,
				{ opacity: 0, y: 15 },
				{ opacity: 1, y: 0, duration: 0.55 },
				'-=0.2'
			);
		}

		let scrollTween: gsap.core.Tween | null = null;
		if (heroContentEl) {
			gsap.registerPlugin(ScrollTrigger);
			scrollTween = gsap.to(heroContentEl, {
				scale: 0.94,
				opacity: 0.25,
				y: -35,
				ease: 'none',
				scrollTrigger: {
					trigger: heroEl,
					start: 'top top',
					end: 'bottom 20%',
					scrub: 0.5
				}
			});
		}

		return () => {
			scrollTween?.scrollTrigger?.kill();
			scrollTween?.kill();
			tl.kill();
		};
	});

	const features = [
		{ key: 'no_cloud', icon: CloudOffIcon },
		{ key: 'p2p_sync', icon: WaypointsIcon },
		{ key: 'open_source', icon: CodeIcon }
	] as const;

	const FEATURE_LABELS = {
		no_cloud: {
			title: m['landing.features.noCloud.title'],
			desc: m['landing.features.noCloud.desc']
		},
		p2p_sync: {
			title: m['landing.features.p2pSync.title'],
			desc: m['landing.features.p2pSync.desc']
		},
		open_source: {
			title: m['landing.features.openSource.title'],
			desc: m['landing.features.openSource.desc']
		}
	} as const;

	const platforms = [
		{ key: 'android', icon: SmartphoneIcon },
		{ key: 'desktop', icon: MonitorIcon },
		{ key: 'relay', icon: ServerIcon },
		{ key: 'p2p', icon: NetworkIcon },
		{ key: 'relay_lib', icon: RadioIcon }
	] as const;

	const PLATFORM_LABELS = {
		android: {
			title: m['landing.platform.android.title'],
			desc: m['landing.platform.android.desc']
		},
		desktop: {
			title: m['landing.platform.desktop.title'],
			desc: m['landing.platform.desktop.desc']
		},
		relay: { title: m['landing.platform.relay.title'], desc: m['landing.platform.relay.desc'] },
		p2p: { title: m['landing.platform.p2p.title'], desc: m['landing.platform.p2p.desc'] },
		relay_lib: {
			title: m['landing.platform.relayLib.title'],
			desc: m['landing.platform.relayLib.desc']
		}
	} as const;

	const architectureFeatures = $derived<FeatureItem[]>([
		{
			icon: RadioIcon,
			title: m['landing.architecture.p2p.title'](),
			description: m['landing.architecture.p2p.desc']()
		},
		{
			icon: ServerIcon,
			title: m['landing.architecture.relay.title'](),
			description: m['landing.architecture.relay.desc']()
		},
		{
			icon: CodeIcon,
			title: m['landing.architecture.metadata.title'](),
			description: m['landing.architecture.metadata.desc']()
		}
	]);

	const architectureTabs = $derived<CodeTab[]>([
		{
			id: 'p2p-sync',
			label: m['landing.architecture.tabs.p2pLabel'](),
			filename: 'src/p2p/sync.rs',
			code: `${m['landing.architecture.tabs.p2pCommentConnect']()}
let endpoint = Endpoint::builder()
    .discovery_n0()
    .bind()
    .await?;

let connection = endpoint.connect(peer_node_id, ALPN).await?;
let (mut send, mut recv) = connection.open_bi().await?;

${m['landing.architecture.tabs.p2pCommentTransfer']()}
send.write_all(&comic_sync_payload).await?;`
		},
		{
			id: 'caddyfile',
			label: m['landing.architecture.tabs.caddyLabel'](),
			filename: 'Caddyfile',
			code: `${m['landing.architecture.tabs.caddyComment']()}
relay.reader.acerola.app {
    reverse_proxy localhost:3340 {
        header_up Host {host}
        header_up X-Real-IP {remote_host}
    }
}`
		},
		{
			id: 'comicinfo',
			label: 'ComicInfo.xml',
			filename: 'ComicInfo.xml',
			code: `<?xml version="1.0" encoding="utf-8"?>
<ComicInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <Title>Chainsaw Man - Vol. 1</Title>
  <Series>Chainsaw Man</Series>
  <Number>1</Number>
  <Writer>Tatsuki Fujimoto</Writer>
  <Manga>YesAndRightToLeft</Manga>
</ComicInfo>`
		}
	]);
</script>

<svelte:head>
	<title>{m['site.name']()} — {m['nav.docs']()}</title>
</svelte:head>

<div class="relative overflow-hidden">
	<!-- Ambient glow behind hero -->
	<div
		class="pointer-events-none absolute inset-x-0 -top-24 -z-10 flex justify-center overflow-hidden"
		aria-hidden="true"
	>
		<div
			class="h-[460px] w-[800px] rounded-full bg-gradient-to-b from-primary/15 via-primary/5 to-transparent blur-3xl"
		></div>
	</div>

	<div class="mx-auto max-w-5xl px-4 sm:px-6 lg:px-8">
		<!-- Hero: centered with a scroll indicator -->
		<section
			bind:this={heroEl}
			class="relative flex min-h-[calc(100vh-4.5rem)] flex-col items-center justify-center py-12 text-center sm:py-16 lg:py-20"
		>
			<div
				bind:this={heroContentEl}
				class="flex max-w-3xl flex-col items-center will-change-transform"
			>
				<!-- Brand moment: the mascot, from the brand CDN. -->
				<div class="hero-logo relative mb-6">
					<img
						src={BRAND_MASCOT_URL}
						alt=""
						width="144"
						height="144"
						class="relative size-28 transition-transform duration-300 hover:scale-105 sm:size-36"
					/>
				</div>

				<div class="hero-kicker">
					<Badge variant="secondary" class="mb-4">{m['landing.heroKicker']()}</Badge>
				</div>

				<h1
					class="hero-title font-display text-3xl text-balance sm:text-4xl md:text-5xl lg:text-6xl"
				>
					<AcerolaShinyText text={m['landing.heroTitle']()} />
				</h1>

				<p
					class="hero-subtitle mt-4 max-w-2xl text-base text-muted-foreground sm:text-lg lg:text-xl"
				>
					{m['landing.heroSubtitle']()}
				</p>

				<div class="hero-downloads mt-8 grid w-full max-w-md grid-cols-2 gap-3">
					<AcerolaApkDownloadButton />
					<AcerolaMicrosoftStoreButton />
				</div>

				<div class="hero-ctas mt-4 flex flex-wrap items-center justify-center gap-3">
					<Button href={localizeHref('/docs/getting-started')} variant="link">
						{m['landing.ctaGetStarted']()}
					</Button>
					<Button href={GITHUB_URL} target="_blank" rel="noreferrer" variant="ghost">
						<GithubIcon size={16} />
						{m['landing.ctaGithub']()}
					</Button>
				</div>

				<div class="hero-facts mt-8 flex flex-wrap items-center justify-center gap-2">
					<Badge variant="outline">{m['landing.facts.noCloud']()}</Badge>
					<Badge variant="outline">{m['landing.facts.noAccount']()}</Badge>
					<Badge variant="outline">{m['landing.facts.openSource']()}</Badge>
				</div>
			</div>

			<!-- Scroll Indicator Hint -->
			<a
				href="#features"
				class="scroll-indicator group mt-12 inline-flex flex-col items-center gap-2 text-muted-foreground transition-all duration-300 hover:text-foreground"
				aria-label={m['landing.scrollHint']()}
			>
				<span
					class="font-mono text-[0.68rem] tracking-widest text-muted-foreground/80 uppercase transition-colors group-hover:text-primary"
				>
					{m['landing.scrollHint']()}
				</span>
				<div
					class="flex h-8 w-5 items-start justify-center rounded-full border border-border/80 p-1 transition-colors"
				>
					<div class="animate-scroll-bounce h-1.5 w-1 rounded-full bg-primary"></div>
				</div>
				<ChevronDownIcon
					size={14}
					class="-mt-1 text-primary/70 transition-all group-hover:translate-y-0.5 group-hover:text-primary"
				/>
			</a>
		</section>

		<!-- Single-column sections -->
		<div class="flex flex-col gap-28 pb-24 sm:pb-32">
			<section id="features" use:scrollReveal>
				<div class="mb-10 text-center">
					<span
						class="font-mono text-xs font-semibold tracking-wider text-primary uppercase"
					>
						01.
					</span>
					<h2 class="mt-1 font-heading text-2xl font-semibold sm:text-3xl">
						{m['landing.features.title']()}
					</h2>
				</div>

				<div
					use:scrollScale={{
						mode: 'stagger-grid',
						childrenSelector: '[data-slot="card"]',
						stagger: 0.08,
						startScale: 0.88
					}}
				>
					<CardGrid>
						{#each features as feature (feature.key)}
							<PlatformCard
								title={FEATURE_LABELS[feature.key].title()}
								description={FEATURE_LABELS[feature.key].desc()}
								icon={feature.icon}
							/>
						{/each}
					</CardGrid>
				</div>
			</section>

			<section id="how-it-works" use:scrollReveal class="mx-auto w-full max-w-3xl">
				<div class="mb-10 text-center">
					<span
						class="font-mono text-xs font-semibold tracking-wider text-primary uppercase"
					>
						02.
					</span>
					<h2 class="mt-1 font-heading text-2xl font-semibold sm:text-3xl">
						{m['landing.howItWorks.title']()}
					</h2>
				</div>

				<div use:scrollScale={{ mode: 'scale-up', startScale: 0.94 }}>
					<Steps>
						<ol>
							<li>{m['landing.howItWorks.install']()}</li>
							<li>{m['landing.howItWorks.pair']()}</li>
							<li>{m['landing.howItWorks.confirm']()}</li>
						</ol>
					</Steps>

					<div class="mt-8">
						<AcerolaCallout type="tip" title={m['landing.howItWorks.noteTitle']()}>
							<p>{m['landing.howItWorks.noteDesc']()}</p>
						</AcerolaCallout>
					</div>
				</div>
			</section>

			<section id="architecture" use:scrollReveal class="mx-auto w-full max-w-5xl">
				<div class="mb-10 text-center">
					<span
						class="font-mono text-xs font-semibold tracking-wider text-primary uppercase"
					>
						03.
					</span>
					<h2 class="mt-1 font-heading text-2xl font-semibold sm:text-3xl">
						{m['landing.architecture.sectionTitle']()}
					</h2>
				</div>

				<div use:scrollScale={{ mode: 'scale-up', startScale: 0.94 }}>
					<AcerolaMarketingFeatureCode
						eyebrow={m['landing.architecture.eyebrow']()}
						title={m['landing.architecture.title']()}
						description={m['landing.architecture.desc']()}
						features={architectureFeatures}
						tabs={architectureTabs}
					/>
				</div>
			</section>

			<section id="platforms" use:scrollReveal>
				<div class="mb-10 text-center">
					<span
						class="font-mono text-xs font-semibold tracking-wider text-primary uppercase"
					>
						04.
					</span>
					<h2 class="mt-1 font-heading text-2xl font-semibold sm:text-3xl">
						{m['landing.platformsTitle']()}
					</h2>
				</div>

				<div
					use:scrollScale={{
						mode: 'stagger-grid',
						childrenSelector: '[data-slot="card"]',
						stagger: 0.08,
						startScale: 0.88
					}}
				>
					<CardGrid>
						{#each platforms as platform (platform.key)}
							<PlatformCard
								title={PLATFORM_LABELS[platform.key].title()}
								description={PLATFORM_LABELS[platform.key].desc()}
								icon={platform.icon}
							/>
						{/each}
					</CardGrid>
				</div>
			</section>
		</div>
	</div>
</div>
