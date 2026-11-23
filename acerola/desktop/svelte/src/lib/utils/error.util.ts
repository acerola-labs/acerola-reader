import type { ErrorPayload } from '#lib/contracts/shared/shared.payloads.ts';
import { resolveErrorMessage } from '#lib/contracts/errors/errors.i18n.ts';
import { m } from '#lib/paraglide/messages.js';

/**
 * Turns any error status into one of the error screen variants (acerola-brand reference). The
 * screen never shows the raw error message: only the status code (as decoration) and a text that
 * says what to do.
 */
export type ErrorKind =
	| 'badRequest'
	| 'unauthorized'
	| 'forbidden'
	| 'notFound'
	| 'timeout'
	| 'tooMany'
	| 'server'
	| 'unavailable'
	| 'offline'
	| 'unknown';

/** What the primary action does: go home (the problem is the address) or try again. */
export type ErrorAction = 'home' | 'retry';

const BY_STATUS: Record<number, ErrorKind> = {
	400: 'badRequest',
	401: 'unauthorized',
	403: 'forbidden',
	404: 'notFound',
	408: 'timeout',
	429: 'tooMany',
	500: 'server',
	502: 'unavailable',
	503: 'unavailable',
	504: 'unavailable'
};

const RETRY: ErrorKind[] = ['timeout', 'tooMany', 'server', 'unavailable', 'offline', 'unknown'];

export function errorKind(status: number, online = true): ErrorKind {
	if (!online) return 'offline';
	if (BY_STATUS[status]) return BY_STATUS[status];
	if (status >= 500) return 'server';
	if (status >= 400) return 'badRequest';
	return 'unknown';
}

export function errorAction(kind: ErrorKind): ErrorAction {
	return RETRY.includes(kind) ? 'retry' : 'home';
}

/**
 * Extracts a user-friendly error message from an unknown thrown error.
 * Safely parses string errors, Tauri ErrorPayload objects, or Error instances without using `any`.
 */
export function extractErrorMessage(error: unknown): string {
	if (error === null || error === undefined) {
		return m['error.unknown.title']();
	}
	// Stryker disable next-line ConditionalExpression,BlockStatement,StringLiteral: disabling/short-circuiting
	// this guard makes a string input fall into the `return String(error)` below. `String(x) === x`
	// for any primitive string by spec, so the observable output never changes — a mutant that
	// removes or drops this check is truly equivalent for any string input.
	if (typeof error === 'string') {
		return error;
	}
	if (typeof error === 'object') {
		const payload = error as Partial<ErrorPayload>;
		// Payload coming from a Tauri command with a typed `errorType` (ComicError) — translated
		// via Paraglide instead of leaking Rust's raw English `message` to the UI. A type not
		// mapped in COMIC_ERROR_MESSAGES falls back to the raw `message` (same fallback as ever).
		if (typeof payload.errorType === 'string' && typeof payload.message === 'string') {
			return resolveErrorMessage(payload as ErrorPayload);
		}
		// Stryker disable next-line ConditionalExpression,LogicalOperator,StringLiteral,EqualityOperator,BlockStatement,MethodExpression:
		// this branch and the fallback right below read the same `message` property of the same
		// object (`payload` is just `error` with a cast, not a copy), without trim. Whenever
		// `typeof payload.message === 'string'` is true, `'message' in error` is necessarily
		// true too, so the fallback fires with the identical value. That makes the exact shape
		// of this condition unobservable: disabling it, forcing any operand, flipping the
		// `.trim().length > 0` boundary either way, dropping `.trim()` or emptying the if-block
		// — all still return the same string, the flow just reroutes through the fallback
		// branch (or vice versa), but the return value is always identical.
		if (typeof payload.message === 'string' && payload.message.trim().length > 0) {
			return payload.message;
		}
		if ('message' in error && typeof (error as { message: unknown }).message === 'string') {
			return (error as { message: string }).message;
		}
	}
	return String(error);
}
