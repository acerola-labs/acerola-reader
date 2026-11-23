import { render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LandingPage from './+page.svelte';

// FaultyTerminal uses the `ogl` lib to draw a WebGL shader in the hero — jsdom has no WebGL,
// and the team has already documented (see faulty-terminal.svelte) this effect being unstable
// under a virtual/software GPU even in Playwright's headless Chromium. We mock the whole lib
// here: this test is about the landing page's composition, not the shader itself.
vi.mock('ogl', () => {
	class FakeRenderer {
		gl = {
			clearColor: () => {},
			canvas: document.createElement('canvas'),
			getExtension: () => ({ loseContext: () => {} })
		};
		setSize() {}
		render() {}
	}
	class FakeProgram {
		uniforms: Record<string, { value: unknown }>;
		constructor(_gl: unknown, options: { uniforms: Record<string, { value: unknown }> }) {
			this.uniforms = options.uniforms;
		}
	}
	class FakeMesh {}
	class FakeTriangle {}
	class FakeColor {
		constructor(...args: number[]) {
			Object.assign(this, args);
		}
	}

	return {
		Renderer: FakeRenderer,
		Program: FakeProgram,
		Mesh: FakeMesh,
		Triangle: FakeTriangle,
		Color: FakeColor
	};
});

describe('landing +page', () => {
	beforeEach(() => {
		globalThis.ResizeObserver = class {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof ResizeObserver;
	});

	it('renders the hero title and the get started / github CTAs', () => {
		render(LandingPage);

		expect(screen.getByRole('link', { name: /get started|começar/i })).toHaveAttribute(
			'href',
			'/docs/getting-started'
		);
		expect(screen.getByRole('link', { name: /github/i })).toHaveAttribute(
			'href',
			expect.stringContaining('github.com')
		);
	});

	it('renders a platform card for each supported platform', () => {
		render(LandingPage);

		expect(screen.getAllByText(/android|desktop|relay|p2p/i).length).toBeGreaterThan(0);
	});
});
