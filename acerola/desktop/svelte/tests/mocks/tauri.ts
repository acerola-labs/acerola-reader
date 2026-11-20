import { vi } from 'vitest';

/**
 * Creates a LazyStore mock with custom initial values.
 * Use in tests that need to simulate a state saved in the store.
 *
 * @example
 * vi.mock("@tauri-apps/plugin-store", () => ({
 *   LazyStore: mockLazyStore({ mode: "dark" }),
 * }));
 */
export function mockLazyStore(savedValues: Record<string, unknown> = {}) {
	return vi.fn().mockImplementation(function () {
		return {
			get: vi.fn().mockImplementation((key: string) => Promise.resolve(savedValues[key] ?? null)),
			set: vi.fn().mockResolvedValue(undefined)
		};
	});
}
