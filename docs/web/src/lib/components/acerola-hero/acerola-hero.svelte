<script lang="ts">
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import { onMount, type Snippet } from 'svelte';
	import gsap from 'gsap';

	type Props = {
		/**
		 * tape: one list of phrases per tape. next: id of the section the scroll cue leads to.
		 * scroll: the scroll cue label (already translated by the page).
		 */
		data: { mascot: string; word: string; tape: string[][]; next?: string; scroll?: string };
		ui?: { stickers?: { text: string; tone: 'warning' | 'info' | 'success' }[] };
		/** Copy and actions below the stage. */
		children?: Snippet;
	};

	let { data, ui = {}, children }: Props = $props();

	let root: HTMLElement;

	/** 24-point sticker star as a clip-path: the "stamp" behind the mascot. */
	const burst = Array.from({ length: 48 }, (_, i) => {
		const angle = (i / 48) * Math.PI * 2;
		const r = i % 2 ? 41 : 50;
		return `${(50 + r * Math.cos(angle)).toFixed(2)}% ${(50 + r * Math.sin(angle)).toFixed(2)}%`;
	}).join(', ');

	/** Each letter with its own tilt: the crooked lettering of a skate deck. */
	const letters = $derived(
		[...data.word].map((char, i) => ({ char, tilt: [-4, 3, -2, 5, -3, 2, -5][i % 7] }))
	);

	onMount(() => {
		const mm = gsap.matchMedia();

		mm.add('(prefers-reduced-motion: no-preference)', () => {
			const q = gsap.utils.selector(root);

			gsap.timeline({ defaults: { ease: 'back.out(1.8)' } })
				.from(q('.hero__letter'), {
					yPercent: 120,
					rotate: () => gsap.utils.random(-25, 25),
					opacity: 0,
					duration: 0.9,
					stagger: 0.07
				})
				.from(
					q('.hero__burst'),
					{ scale: 0, rotate: -90, duration: 1, ease: 'expo.out' },
					0.2
				)
				.from(
					q('.hero__mascot'),
					{ yPercent: 40, scale: 0.85, opacity: 0, duration: 1 },
					0.45
				)
				.from(
					q('.hero__tape'),
					{ scaleX: 0, duration: 0.8, ease: 'expo.out', stagger: 0.12 },
					0.6
				)
				.from(
					q('.hero__sticker'),
					{ scale: 0, rotate: -45, duration: 0.6, stagger: 0.1 },
					0.9
				)
				.from(
					q('.hero__copy > *'),
					{ y: 24, opacity: 0, duration: 0.6, stagger: 0.08, ease: 'power3.out' },
					1
				);

			// Life after the intro: the mascot breathes, the stamp spins, the tapes run.
			gsap.to(q('.hero__mascot'), {
				y: -14,
				duration: 2.6,
				ease: 'sine.inOut',
				yoyo: true,
				repeat: -1
			});
			gsap.to(q('.hero__burst'), { rotate: 360, duration: 60, ease: 'none', repeat: -1 });
			q('.hero__track').forEach((track, i) => {
				gsap.fromTo(
					track,
					{ xPercent: i % 2 ? -50 : 0 },
					// ~4s per phrase: a tape keeps the same pace whether it has 4 or 20 phrases.
					{
						xPercent: i % 2 ? 0 : -50,
						duration: (data.tape[i]?.length ?? 6) * 4,
						ease: 'none',
						repeat: -1
					}
				);
			});

			// Scroll cue: the wheel dot slides down and fades; the chevron bounces.
			gsap.fromTo(
				q('.hero__wheel'),
				{ y: 0, opacity: 1 },
				{
					y: 12,
					opacity: 0,
					duration: 1.4,
					ease: 'power2.in',
					repeat: -1,
					repeatDelay: 0.2
				}
			);
			gsap.to(q('.hero__chevron'), {
				y: 5,
				duration: 0.7,
				ease: 'sine.inOut',
				yoyo: true,
				repeat: -1
			});

			// Light mouse parallax: lettering and mascot move in opposite directions.
			const wordX = gsap.quickTo(q('.hero__word'), 'x', { duration: 0.8, ease: 'power3' });
			const mascotX = gsap.quickTo(q('.hero__mascot-wrap'), 'x', {
				duration: 0.8,
				ease: 'power3'
			});
			const onMove = (event: PointerEvent) => {
				const offset = event.clientX / window.innerWidth - 0.5;
				wordX(offset * -30);
				mascotX(offset * 40);
			};
			window.addEventListener('pointermove', onMove);

			// A letter jumps when the pointer passes over it.
			const bounce = (event: Event) =>
				gsap.fromTo(
					event.currentTarget,
					{ y: 0 },
					{ y: -24, duration: 0.25, ease: 'power2.out', yoyo: true, repeat: 1 }
				);
			const letterEls = q('.hero__letter');
			letterEls.forEach((el) => el.addEventListener('pointerenter', bounce));

			return () => {
				window.removeEventListener('pointermove', onMove);
				letterEls.forEach((el) => el.removeEventListener('pointerenter', bounce));
			};
		});

		return () => mm.revert();
	});
</script>

<section bind:this={root} class="hero" aria-label={data.word}>
	<h1 class="hero__word" aria-label={data.word}>
		{#each letters as letter, i (i)}
			<span class="hero__letter" style:--tilt="{letter.tilt}deg" aria-hidden="true"
				>{letter.char}</span
			>
		{/each}
	</h1>

	<div class="hero__stage" aria-hidden="true">
		<div class="hero__burst" style:clip-path="polygon({burst})"></div>

		{#each data.tape as phrases, i (i)}
			<div class="hero__tape hero__tape--{i % 2 ? 'b' : 'a'}">
				<div class="hero__track">
					{#each [0, 1] as copy (copy)}
						<span class="hero__tape-text">
							{#each phrases as phrase (phrase)}{phrase}<b>✦</b>{/each}
						</span>
					{/each}
				</div>
			</div>
		{/each}

		<div class="hero__mascot-wrap">
			<img class="hero__mascot" src={data.mascot} alt="" />
		</div>

		{#each ui.stickers ?? [] as sticker, i (sticker.text)}
			<span class="hero__sticker hero__sticker--{i} hero__sticker--{sticker.tone}"
				>{sticker.text}</span
			>
		{/each}
	</div>

	{#if children}
		<div class="hero__copy">{@render children()}</div>
	{/if}

	{#if data.next}
		<a class="hero__scroll" href="#{data.next}">
			{#if data.scroll}<span class="hero__scroll-label">{data.scroll}</span>{/if}
			<span class="hero__mouse" aria-hidden="true"><span class="hero__wheel"></span></span>
			<span class="hero__chevron" aria-hidden="true"><ChevronDown size={16} /></span>
		</a>
	{/if}
</section>

<style>
	.hero {
		position: relative;
		isolation: isolate;
		display: grid;
		grid-template-rows: auto 1fr auto;
		min-height: calc(100svh - 4.5rem);
		overflow: hidden;
		padding-top: 1.5rem;
	}

	/* ---------- Lettering: Titan One sticker, dark outline and a hard shadow in the brand color ---------- */

	.hero__word {
		position: relative;
		z-index: 2;
		display: flex;
		justify-content: center;
		margin: 0;
		font-family: var(--font-display);
		font-size: clamp(4.5rem, 18.5vw, 20rem);
		font-weight: 400;
		line-height: 0.82;
		letter-spacing: -0.01em;
		text-transform: uppercase;
		user-select: none;
	}

	.hero__letter {
		display: inline-block;
		color: var(--accent-hero);
		-webkit-text-stroke: 0.05em var(--crust);
		paint-order: stroke fill;
		text-shadow:
			0.035em 0.035em 0 var(--primary),
			0.07em 0.07em 0 var(--crust);
		transform: rotate(var(--tilt));
		will-change: transform;
	}
	.hero__letter:nth-child(even) {
		color: var(--foreground);
	}

	/* ---------- Stage: stamp, tapes, mascot and stickers ---------- */

	.hero__stage {
		position: relative;
		margin-top: -9vw;
		min-height: 26rem;
	}

	.hero__burst {
		position: absolute;
		z-index: -1;
		left: 50%;
		top: 8%;
		width: min(88vw, 44rem);
		aspect-ratio: 1;
		translate: -50% 0;
		background: var(--primary);
	}

	.hero__tape {
		position: absolute;
		left: -10%;
		width: 120%;
		overflow: hidden;
		padding: 0.6rem 0;
		font-family: var(--font-display);
		font-size: clamp(1.25rem, 2.6vw, 2.25rem);
		text-transform: uppercase;
		white-space: nowrap;
		box-shadow: var(--shadow-float);
	}
	.hero__tape--a {
		z-index: 0;
		top: 52%;
		rotate: -6deg;
		background: var(--accent-hero);
		color: var(--accent-hero-foreground);
	}
	.hero__tape--b {
		z-index: 3;
		top: 74%;
		rotate: 4deg;
		background: var(--foreground);
		color: var(--background);
	}
	.hero__track {
		display: flex;
		width: max-content;
	}
	.hero__tape-text b {
		margin: 0 0.6em;
		color: var(--primary);
		font-weight: 400;
	}

	.hero__mascot-wrap {
		position: relative;
		z-index: 2;
		display: flex;
		justify-content: center;
	}
	.hero__mascot {
		width: min(78vw, 34rem);
		will-change: transform;
	}

	.hero__sticker {
		position: absolute;
		z-index: 4;
		padding: 0.5em 0.9em;
		border-radius: var(--radius-full);
		font-family: var(--font-display);
		font-size: clamp(0.85rem, 1.4vw, 1.25rem);
		text-transform: uppercase;
		color: var(--crust);
		box-shadow:
			0 0 0 3px var(--crust),
			4px 5px 0 3px var(--crust);
	}
	.hero__sticker--warning {
		background: var(--warning);
	}
	.hero__sticker--info {
		background: var(--info);
	}
	.hero__sticker--success {
		background: var(--success);
	}
	.hero__sticker--0 {
		left: 8%;
		top: 14%;
		rotate: -12deg;
	}
	.hero__sticker--1 {
		right: 9%;
		top: 6%;
		rotate: 9deg;
	}
	.hero__sticker--2 {
		right: 14%;
		top: 40%;
		rotate: -6deg;
	}

	.hero__copy {
		position: relative;
		z-index: 5;
		display: grid;
		gap: 1.25rem;
		align-items: end;
		padding: 1.5rem 1rem 2rem;
	}
	@media (min-width: 768px) {
		.hero__copy {
			grid-template-columns: 1fr auto;
			padding-inline: 1.5rem;
		}
	}

	/* ---------- Scroll cue ---------- */

	.hero__scroll {
		position: absolute;
		z-index: 6;
		bottom: 1.5rem;
		left: 50%;
		display: none;
		flex-direction: column;
		align-items: center;
		gap: 0.6rem;
		padding: 0.9rem 1.4rem 0.7rem;
		border-radius: var(--radius-card);
		background: var(--background);
		box-shadow: var(--shadow-float);
		translate: -50% 0;
		color: var(--muted-foreground);
		text-decoration: none;
		transition: color 300ms var(--ease-brand);
	}
	.hero__scroll:hover {
		color: var(--foreground);
	}
	.hero__scroll-label {
		font-size: 0.75rem;
		font-weight: 700;
		letter-spacing: 0.22em;
		text-transform: uppercase;
	}
	.hero__mouse {
		display: flex;
		justify-content: center;
		width: 1.5rem;
		height: 2.4rem;
		padding-top: 0.45rem;
		border-radius: var(--radius-full);
		box-shadow: inset 0 0 0 2px currentColor;
	}
	.hero__wheel {
		width: 0.3rem;
		height: 0.55rem;
		border-radius: var(--radius-full);
		background: var(--primary);
	}
	.hero__chevron {
		display: flex;
		color: var(--primary);
	}
	@media (min-width: 768px) {
		.hero__scroll {
			display: flex;
		}
	}

	@media (max-width: 640px) {
		.hero__stage {
			margin-top: -3vw;
			min-height: 17rem;
		}
		.hero__mascot {
			width: 62vw;
		}
		.hero__tape--a {
			top: 58%;
		}
		.hero__tape--b {
			top: 80%;
		}
		.hero__sticker--2 {
			display: none;
		}
	}
</style>
