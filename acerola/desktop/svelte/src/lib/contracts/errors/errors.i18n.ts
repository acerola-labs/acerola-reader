import * as m from '#lib/paraglide/messages.js';
import type { ErrorPayload } from '#lib/contracts/shared/shared.payloads.ts';

// Contract between the Rust error types (ComicError) and the Paraglide messages.
// The key is the exact value of the `errorType` field from the Tauri event payload.
export const COMIC_ERROR_MESSAGES: Record<string, () => string> = {
	AlreadyExists: m['tauriErrors.comic.alreadyExists.label'],
	SyncInProgress: m['tauriErrors.comic.syncInProgress.label'],
	NotFound: m['tauriErrors.comic.notFound.label'],
	InvalidRequest: m['tauriErrors.comic.invalidRequest.label'],
	IntegrityViolation: m['tauriErrors.comic.integrityViolation.label'],
	SystemFailure: m['tauriErrors.comic.systemFailure.label'],
	Io: m['tauriErrors.comic.ioError.label']
};

// Resolves the translated message from the error payload.
// Uses Rust's technical `message` as a fallback if the type is not mapped.
export function resolveErrorMessage(payload: ErrorPayload): string {
	return COMIC_ERROR_MESSAGES[payload.errorType]?.() ?? payload.message;
}
