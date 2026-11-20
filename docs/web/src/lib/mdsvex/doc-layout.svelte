<script lang="ts">
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ChevronUpIcon from '@lucide/svelte/icons/chevron-up';
	import MinusIcon from '@lucide/svelte/icons/minus';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';
	import CheckIcon from '@lucide/svelte/icons/check';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import { page } from '$app/state';
	import AcerolaCopyMarkdownButton from '#lib/components/acerola-copy-markdown-button/acerola-copy-markdown-button.svelte';
	import { DOC_RAW_CONTEXT_KEY, type DocRawContext } from '#lib/content/doc-raw-context.ts';
	import { useTheme } from '#lib/hooks/theme/use-theme.svelte.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getContext, mount, type Snippet } from 'svelte';

	let {
		title,
		description,
		children
	}: { title?: string; description?: string; children: Snippet } = $props();

	const themeCtx = useTheme();
	const docRaw = getContext<DocRawContext | undefined>(DOC_RAW_CONTEXT_KEY);
	let container = $state<HTMLElement>();

	const MIN_SCALE = 0.5;
	const MAX_SCALE = 3;
	const PAN_STEP = 60;
	type Transform = { scale: number; x: number; y: number };
	const transforms = new WeakMap<HTMLElement, Transform>();

	function iconButton(labelText: string, icon: typeof MinusIcon, onClick: () => void) {
		const button = document.createElement('button');
		button.type = 'button';
		button.setAttribute('aria-label', labelText);
		button.className =
			'flex items-center justify-center rounded-inner p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground';
		mount(icon, { target: button, props: { size: 14 } });
		button.addEventListener('click', onClick);
		return button;
	}

	// The diagrams are plain HTML coming from the markdown pipeline (see rehypeMermaid in
	// mdsvex.config.js), not Svelte templates — the controls have to be grafted onto each
	// `.mermaid` node imperatively instead of declared in the template.
	// Pans/scales the diagram in place (like GitHub's mermaid viewer, which this mirrors)
	// instead of opening a dialog — a cloned SVG fighting a dialog's own layout for space
	// proved far more fragile than transforming the original SVG.
	function addZoomControls(node: HTMLElement) {
		if (node.querySelector('[data-mermaid-controls]')) return;

		const svg = node.querySelector<SVGElement>('svg');
		if (!svg) return;

		transforms.set(node, { scale: 1, x: 0, y: 0 });

		const zoomLabel = document.createElement('span');
		zoomLabel.className = 'text-center text-xs tabular-nums text-muted-foreground';
		zoomLabel.textContent = '100%';

		function apply() {
			const t = transforms.get(node)!;
			svg!.style.transform = `translate(${t.x}px, ${t.y}px) scale(${t.scale})`;
			svg!.style.transformOrigin = 'center';
			zoomLabel.textContent = Math.round(t.scale * 100) + '%';
		}

		function pan(dx: number, dy: number) {
			const t = transforms.get(node)!;
			t.x += dx;
			t.y += dy;
			apply();
		}

		function zoomBy(delta: number) {
			const t = transforms.get(node)!;
			t.scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, t.scale + delta));
			apply();
		}

		function reset() {
			transforms.set(node, { scale: 1, x: 0, y: 0 });
			apply();
		}

		const pad = document.createElement('div');
		pad.className = 'grid grid-cols-3 grid-rows-3 place-items-center';
		const cell = (col: number, row: number, el: HTMLElement) => {
			el.style.gridColumn = String(col);
			el.style.gridRow = String(row);
			return el;
		};
		pad.append(
			cell(
				2,
				1,
				iconButton(m['mermaid.panUp'](), ChevronUpIcon, () => pan(0, PAN_STEP))
			),
			cell(
				1,
				2,
				iconButton(m['mermaid.panLeft'](), ChevronLeftIcon, () => pan(PAN_STEP, 0))
			),
			cell(2, 2, iconButton(m['mermaid.reset'](), RotateCcwIcon, reset)),
			cell(
				3,
				2,
				iconButton(m['mermaid.panRight'](), ChevronRightIcon, () => pan(-PAN_STEP, 0))
			),
			cell(
				2,
				3,
				iconButton(m['mermaid.panDown'](), ChevronDownIcon, () => pan(0, -PAN_STEP))
			)
		);

		const zoomStack = document.createElement('div');
		zoomStack.className = 'flex flex-col items-center gap-0.5';
		zoomStack.append(
			iconButton(m['mermaid.zoomIn'](), PlusIcon, () => zoomBy(0.25)),
			zoomLabel,
			iconButton(m['mermaid.zoomOut'](), MinusIcon, () => zoomBy(-0.25))
		);

		const toolbar = document.createElement('div');
		toolbar.dataset.mermaidControls = '';
		toolbar.className =
			'absolute bottom-2 right-2 z-10 flex items-center gap-1.5 rounded-inner border border-border bg-background/90 p-1.5 opacity-0 backdrop-blur transition-opacity';
		toolbar.append(pad, zoomStack);
		node.appendChild(toolbar);
	}

	// mermaid.run() replaces each node's content with a rendered <svg>, so the
	// diagram's original source is captured here on the first pass and reused to
	// reset the node before each new render (palette/mode change).
	const sources = new Map<HTMLElement, string>();

	async function renderMermaid() {
		const nodes = container?.querySelectorAll<HTMLElement>('.mermaid');
		if (!nodes || nodes.length === 0) return;

		const { default: mermaid } = await import('mermaid');
		const styles = getComputedStyle(document.documentElement);
		const primaryColor = styles.getPropertyValue('--primary').trim();
		const lineColor = styles.getPropertyValue('--border').trim();
		const textColor = styles.getPropertyValue('--foreground').trim();
		const background = styles.getPropertyValue('--card').trim();

		mermaid.initialize({
			startOnLoad: false,
			theme: themeCtx.resolved === 'dark' ? 'dark' : 'default',
			themeVariables: {
				primaryColor: background,
				primaryTextColor: textColor,
				primaryBorderColor: primaryColor,
				lineColor,
				textColor,
				background
			}
		});

		for (const node of nodes) {
			if (!sources.has(node)) sources.set(node, node.textContent ?? '');
			node.textContent = sources.get(node) ?? '';
			node.removeAttribute('data-processed');
		}

		await mermaid.run({ nodes: [...nodes] });

		for (const node of nodes) addZoomControls(node);
	}

	// The brand code panel (AcerolaCode) for the markdown code blocks: rehype already rendered the
	// <pre>, so the window bar (dots + language + copy pill) is built here, with the same classes.
	const COPY_IDLE = 'bg-surface text-muted-foreground hover:text-foreground';
	const COPY_DONE = 'bg-success-soft text-success';

	function addCodeCopyButtons() {
		if (!container) return;
		const pres = container.querySelectorAll<HTMLPreElement>('pre');
		for (const pre of pres) {
			if (pre.closest('.mermaid')) continue;
			const parent = pre.parentElement;
			const wrapped =
				parent?.tagName.toLowerCase() === 'figure' || parent?.dataset.codeBox !== undefined;
			if (wrapped && parent?.querySelector('[data-code-copy-button]')) continue;

			// rehype-pretty-code wraps every block in a <figure>; a bare <pre> gets its own box.
			let box = wrapped ? parent : null;
			if (!box) {
				box = document.createElement('div');
				box.dataset.codeBox = '';
				pre.before(box);
				box.append(pre);
			}

			const bar = document.createElement('div');
			bar.dataset.codeBar = '';
			bar.className = 'flex items-center justify-between gap-3 bg-mantle px-3 py-2';

			const left = document.createElement('div');
			left.className = 'flex min-w-0 items-center gap-3';
			const dots = document.createElement('span');
			dots.className = 'flex shrink-0 items-center gap-1.5 pl-1';
			dots.setAttribute('aria-hidden', 'true');
			for (const tone of ['bg-destructive', 'bg-warning', 'bg-success']) {
				const dot = document.createElement('span');
				dot.className = `size-2.5 rounded-full ${tone}`;
				dots.append(dot);
			}
			const language = document.createElement('span');
			language.className = 'truncate font-mono text-xs font-bold text-muted-foreground';
			language.textContent =
				pre.dataset.language ?? pre.querySelector('code')?.dataset.language ?? 'code';
			left.append(dots, language);

			const btn = document.createElement('button');
			btn.type = 'button';
			btn.dataset.codeCopyButton = '';
			btn.setAttribute('aria-label', m['code.copy']());
			btn.className = `flex shrink-0 items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-black transition-all duration-200 ease-brand active:scale-95 ${COPY_IDLE}`;

			const iconSpan = document.createElement('span');
			iconSpan.className = 'flex items-center justify-center';
			mount(CopyIcon, { target: iconSpan, props: { size: 13 } });
			const labelSpan = document.createElement('span');
			labelSpan.textContent = m['common.copy']();
			btn.append(iconSpan, labelSpan);

			let resetTimeout: ReturnType<typeof setTimeout> | undefined;
			btn.addEventListener('click', async (e) => {
				e.stopPropagation();
				const codeEl = pre.querySelector('code') ?? pre;
				try {
					await navigator.clipboard.writeText(codeEl.innerText.trim());
				} catch {
					return; // no clipboard permission: the code stays visible to copy by hand
				}

				iconSpan.innerHTML = '';
				mount(CheckIcon, { target: iconSpan, props: { size: 13, strokeWidth: 2.75 } });
				labelSpan.textContent = m['common.copied']();
				btn.setAttribute('aria-label', m['common.copied']());
				btn.classList.remove(...COPY_IDLE.split(' '));
				btn.classList.add(...COPY_DONE.split(' '));

				clearTimeout(resetTimeout);
				resetTimeout = setTimeout(() => {
					iconSpan.innerHTML = '';
					mount(CopyIcon, { target: iconSpan, props: { size: 13 } });
					labelSpan.textContent = m['common.copy']();
					btn.setAttribute('aria-label', m['code.copy']());
					btn.classList.remove(...COPY_DONE.split(' '));
					btn.classList.add(...COPY_IDLE.split(' '));
				}, 2000);
			});

			bar.append(left, btn);
			box.prepend(bar);
		}
	}

	$effect(() => {
		// Re-render whenever the light/dark mode changes.
		themeCtx.resolved;
		renderMermaid();
		addCodeCopyButtons();
	});
</script>

<article bind:this={container} class="doc-content max-w-none">
	{#if title}
		<div class="mb-3 flex items-start justify-between gap-4">
			<h1 class="m-0 text-4xl leading-none font-black tracking-tighter sm:text-[44px]">
				{title}
			</h1>
			{#if docRaw}
				<AcerolaCopyMarkdownButton
					raw={docRaw.value}
					url={page.url.href}
					label={m['doc.copy.markdown']()}
					copiedLabel={m['doc.copy.copied']()}
					failedLabel={m['doc.copy.failed']()}
				/>
			{/if}
		</div>
	{/if}
	{#if description}
		<p class="mb-10 max-w-2xl text-lg text-pretty text-muted-foreground">{description}</p>
	{/if}
	{@render children()}
</article>
