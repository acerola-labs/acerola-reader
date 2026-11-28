<script lang="ts">
	import { scrollReveal, scrollScale } from '#lib/actions/index.ts';
	import CloudOffIcon from '@lucide/svelte/icons/cloud-off';
	import CodeIcon from '@lucide/svelte/icons/code';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';
	import { Button } from '#lib/components/ui/button/index.ts';
	import AcerolaApkDownloadButton from '#lib/components/acerola-apk-download-button/acerola-apk-download-button.svelte';
	import AcerolaCallout from '#lib/components/acerola-callout/acerola-callout.svelte';
	import AcerolaHero from '#lib/components/acerola-hero/acerola-hero.svelte';
	import AcerolaMicrosoftStoreButton from '#lib/components/acerola-microsoft-store-button/acerola-microsoft-store-button.svelte';
	import AcerolaSection from '#lib/components/acerola-section/acerola-section.svelte';
	import CardGrid from '#lib/mdsvex/card-grid.svelte';
	import PlatformCard from '#lib/mdsvex/platform-card.svelte';
	import Steps from '#lib/mdsvex/steps.svelte';
	import AcerolaMarketingFeatureCode, {
		type FeatureItem,
		type CodeTab
	} from '#lib/components/acerola-marketing-feature-code/acerola-marketing-feature-code.svelte';
	import GithubIcon from '#lib/assets/icons/github.svg?component';
	import { BRAND_MASCOT_URL, GITHUB_URL } from '#lib/constants/site.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { localizeHref } from '#lib/paraglide/runtime.js';

	// Hero tapes: the site's promises running across the stage, as on the brand book.
	const heroTape = $derived([
		[
			m['landing.facts.noCloud'](),
			m['landing.facts.noAccount'](),
			m['landing.facts.openSource'](),
			m['landing.heroKicker']()
		],
		[
			m['landing.features.noCloud.title'](),
			m['landing.features.p2pSync.title'](),
			m['landing.features.openSource.title']()
		]
	]);

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

<!-- Brand moment (as on the brand book): Titan One lettering, stamp, mascot, tapes and stickers.
     The mascot comes from the brand CDN. -->
<AcerolaHero
	data={{
		mascot: BRAND_MASCOT_URL,
		word: 'Acerola',
		tape: heroTape,
		next: 'features',
		scroll: m['landing.scrollHint']()
	}}
	ui={{
		stickers: [
			{ text: 'P2P', tone: 'info' },
			{ text: m['landing.facts.noCloud'](), tone: 'success' },
			{ text: 'MPL-2.0', tone: 'warning' }
		]
	}}
>
	<!-- One column: title, lead, actions, and the downloads at the bottom. -->
	<div class="grid max-w-xl gap-5">
		<div>
			<p class="m-0 text-2xl font-black tracking-tight sm:text-3xl">
				{m['landing.heroTitle']()}
			</p>
			<p class="mt-2 mb-0 text-pretty text-muted-foreground">{m['landing.heroSubtitle']()}</p>
		</div>
		<div class="flex flex-wrap items-center gap-3">
			<Button href={localizeHref('/docs/getting-started')} size="lg">
				{m['landing.ctaGetStarted']()}
			</Button>
			<Button
				href={GITHUB_URL}
				target="_blank"
				rel="noreferrer"
				variant="secondary"
				size="lg"
			>
				<GithubIcon class="size-4" aria-hidden="true" />
				{m['landing.ctaGithub']()}
			</Button>
		</div>
		<div class="grid w-full max-w-md grid-cols-2 gap-3">
			<AcerolaApkDownloadButton />
			<AcerolaMicrosoftStoreButton />
		</div>
	</div>
</AcerolaHero>

<div use:scrollReveal>
	<AcerolaSection
		data={{
			id: 'features',
			eyebrow: m['landing.features.eyebrow'](),
			title: m['landing.features.title']()
		}}
	>
		<div
			use:scrollScale={{
				mode: 'stagger-grid',
				childrenSelector: '[data-slot="card"]',
				stagger: 0.08,
				startScale: 0.94
			}}
		>
			<CardGrid columns={3}>
				{#each features as feature (feature.key)}
					<PlatformCard
						title={FEATURE_LABELS[feature.key].title()}
						description={FEATURE_LABELS[feature.key].desc()}
						icon={feature.icon}
					/>
				{/each}
			</CardGrid>
		</div>
	</AcerolaSection>
</div>

<div use:scrollReveal>
	<AcerolaSection
		data={{
			id: 'how-it-works',
			eyebrow: m['landing.howItWorks.eyebrow'](),
			title: m['landing.howItWorks.title']()
		}}
	>
		<div class="grid max-w-3xl gap-6">
			<Steps>
				<ol>
					<li>{m['landing.howItWorks.install']()}</li>
					<li>{m['landing.howItWorks.pair']()}</li>
					<li>{m['landing.howItWorks.confirm']()}</li>
				</ol>
			</Steps>

			<AcerolaCallout type="tip" title={m['landing.howItWorks.noteTitle']()}>
				<p>{m['landing.howItWorks.noteDesc']()}</p>
			</AcerolaCallout>
		</div>
	</AcerolaSection>
</div>

<div use:scrollReveal>
	<AcerolaSection
		data={{
			id: 'architecture',
			eyebrow: m['landing.architecture.eyebrow'](),
			title: m['landing.architecture.sectionTitle'](),
			lead: m['landing.architecture.desc']()
		}}
	>
		<AcerolaMarketingFeatureCode features={architectureFeatures} tabs={architectureTabs} />
	</AcerolaSection>
</div>

<div use:scrollReveal>
	<AcerolaSection
		data={{
			id: 'platforms',
			eyebrow: m['landing.platformsEyebrow'](),
			title: m['landing.platformsTitle']()
		}}
	>
		<div
			use:scrollScale={{
				mode: 'stagger-grid',
				childrenSelector: '[data-slot="card"]',
				stagger: 0.08,
				startScale: 0.94
			}}
		>
			<CardGrid columns={3}>
				{#each platforms as platform (platform.key)}
					<PlatformCard
						title={PLATFORM_LABELS[platform.key].title()}
						description={PLATFORM_LABELS[platform.key].desc()}
						icon={platform.icon}
					/>
				{/each}
			</CardGrid>
		</div>
	</AcerolaSection>
</div>
