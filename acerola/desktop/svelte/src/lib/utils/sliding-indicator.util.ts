import gsap from 'gsap';
import type { Action } from 'svelte/action';
import { cn } from '#lib/utils/cn.util.ts';

type TrackedProp = 'x' | 'y' | 'width' | 'height';

export type SlidingIndicatorOptions = {
	/// Selector (relative to the container) of the currently active item — read via attribute,
	/// not a reactive value (`aria-current="page"` in the sidebar, `data-state="on"` in the
	/// toggle group), so the action stays pure DOM and needs no `update()` tied to a rune.
	selector: string;
	/// Classes of the injected indicator element — controls its look (pill behind an icon,
	/// thin bar under a tab). Position/size properties the class already fixes via CSS
	/// (e.g. `bottom-0 h-1` on a bar) must be left out of `track` below.
	indicatorClass: string;
	duration?: number;
	/// Which properties of the active item's box to mirror on the indicator — the rest stays
	/// fixed by the CSS classes above. Default: all (the "pill" case covering the whole box).
	track?: TrackedProp[];
};

const DEFAULT_TRACK: TrackedProp[] = ['x', 'y', 'width', 'height'];
const DEFAULT_DURATION = 0.22;

/// Replaces an instant class swap (e.g. `bg-primary` jumping from item to item) with an
/// indicator that slides to the active item's position/size — reapplied whenever the attribute
/// marking "active" changes in the container (via `MutationObserver`, not a reactive prop).
export const slidingIndicator: Action<HTMLElement, SlidingIndicatorOptions> = (node, opts) => {
	// `isolate` keeps the `-z-10` indicator above the container's own (solid) background.
	node.classList.add('relative', 'isolate');

	const indicator = document.createElement('div');
	indicator.className = cn(
		'pointer-events-none absolute top-0 left-0 -z-10 opacity-0',
		opts.indicatorClass
	);
	node.prepend(indicator);

	const track = opts.track ?? DEFAULT_TRACK;
	let tween: gsap.core.Tween | undefined;

	function snap(animate: boolean) {
		const active = node.querySelector<HTMLElement>(opts.selector);
		tween?.kill();

		if (!active) {
			gsap.set(indicator, { opacity: 0 });
			return;
		}

		const containerRect = node.getBoundingClientRect();
		const rect = active.getBoundingClientRect();
		const full: Record<TrackedProp, number> = {
			x: rect.left - containerRect.left,
			y: rect.top - containerRect.top,
			width: rect.width,
			height: rect.height
		};

		const target: Record<string, number> = { opacity: 1 };
		for (const key of track) target[key] = full[key];

		if (animate) {
			tween = gsap.to(indicator, {
				...target,
				duration: opts.duration ?? DEFAULT_DURATION,
				ease: 'power2.out'
			});
		} else {
			gsap.set(indicator, target);
		}
	}

	snap(false);
	// Tab/toggle libraries (bits-ui) resolve which item starts "active" through their own
	// internal reactivity, not a direct prop — there is no guarantee the attribute is already
	// in the DOM at the synchronous moment of this `use:`. A re-snap on the next frame covers
	// that window without waiting for a mutation that may never happen (item born active).
	requestAnimationFrame(() => snap(false));
	// Font metrics only settle after the first paint — without this, an item with text may
	// measure the wrong width until the layout fully stabilizes.
	document.fonts?.ready?.then(() => snap(false));

	const mutationObserver = new MutationObserver(() => snap(true));
	mutationObserver.observe(node, {
		attributes: true,
		subtree: true,
		attributeFilter: ['aria-current', 'data-state']
	});

	const resizeObserver = new ResizeObserver(() => snap(false));
	resizeObserver.observe(node);

	return {
		destroy() {
			mutationObserver.disconnect();
			resizeObserver.disconnect();
			tween?.kill();
			indicator.remove();
		}
	};
};
