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
	// Stryker disable next-line ConditionalExpression,BlockStatement,StringLiteral: desativar/curto-circuitar
	// esse guard faz uma entrada string cair no `return String(error)` lá embaixo. `String(x) === x`
	// pra qualquer string primitiva por spec, então a saída observável nunca muda — um mutante que
	// remove ou derruba esse check é equivalente de verdade pra qualquer entrada string.
	if (typeof error === 'string') {
		return error;
	}
	if (typeof error === 'object') {
		const payload = error as Partial<ErrorPayload>;
		// Payload vindo de um comando Tauri com `errorType` tipado (ComicError) — traduz via
		// Paraglide em vez de vazar o `message` cru em inglês do Rust pra UI. Tipo não mapeado
		// em COMIC_ERROR_MESSAGES cai no `message` cru mesmo (mesmo fallback de sempre).
		if (typeof payload.errorType === 'string' && typeof payload.message === 'string') {
			return resolveErrorMessage(payload as ErrorPayload);
		}
		// Stryker disable next-line ConditionalExpression,LogicalOperator,StringLiteral,EqualityOperator,BlockStatement,MethodExpression:
		// esse branch e o fallback logo abaixo leem a mesma propriedade `message` do mesmo objeto
		// (`payload` é só `error` com cast, não uma cópia), sem trim. Sempre que
		// `typeof payload.message === 'string'` é verdadeiro, `'message' in error` também é
		// necessariamente verdadeiro, então o fallback dispara com o valor idêntico. Isso torna
		// o formato exato dessa condição inobservável: desativar, forçar qualquer operando,
		// inverter a fronteira de `.trim().length > 0` em qualquer direção, tirar o `.trim()`
		// ou esvaziar o if-block — tudo continua retornando a mesma string, o fluxo só reroteia
		// pelo branch de fallback (ou vice-versa), mas o valor de retorno é sempre idêntico.
		if (typeof payload.message === 'string' && payload.message.trim().length > 0) {
			return payload.message;
		}
		if ('message' in error && typeof (error as { message: unknown }).message === 'string') {
			return (error as { message: string }).message;
		}
	}
	return String(error);
}
