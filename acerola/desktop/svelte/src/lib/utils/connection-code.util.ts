/// Mirrors `acerola_p2p::api::peer::PeerAddr` — no `rename_all` on the Rust side, so the
/// fields come in snake_case straight from the backend (unlike our own payloads).
export type LocalPeerAddr = {
	id: { id: string; device_id: string | null };
	addrs: number[];
};

const CODE_PREFIX = 'acerola1:';

type ConnectionEnvelope = { i: string; d: string | null; a: string };

function bytesToBase64(bytes: number[]): string {
	let binary = '';
	for (const byte of bytes) binary += String.fromCharCode(byte);
	return btoa(binary);
}

function base64ToBytes(base64: string): number[] {
	const binary = atob(base64);
	// Stryker disable next-line ArrayDeclaration : a sizeless `new Array()` ends up byte-for-byte
	// identical after the loop below fills each index in sequence (0..length-1) without
	// skipping any — no input observable through the public API tells the two apart.
	const bytes: number[] = new Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

/**
 * Packs a `PeerAddr` into an opaque token for QR Code/copy-paste. `addrs` is a `Vec<u8>`
 * in Rust — without this, `JSON.stringify` serializes each byte as a comma-separated decimal
 * number (~4 characters per byte). Encoding the bytes in base64 first, and only then packing
 * everything into a single base64 blob, makes the code much shorter and free of "scary"
 * characters like brackets/commas — just an opaque string, the way an invite code should
 * look.
 */
export function encodeConnectionCode(addr: LocalPeerAddr): string {
	const envelope: ConnectionEnvelope = {
		i: addr.id.id,
		d: addr.id.device_id,
		a: bytesToBase64(addr.addrs)
	};

	return CODE_PREFIX + btoa(JSON.stringify(envelope));
}

/** Specific error to tell a "malformed code" apart from other failures in the UI. */
export class InvalidConnectionCodeError extends Error {
	constructor() {
		super('invalid_connection_code');
	}
}

export function decodeConnectionCode(code: string): LocalPeerAddr {
	const trimmed = code.trim();

	if (!trimmed.startsWith(CODE_PREFIX)) {
		throw new InvalidConnectionCodeError();
	}

	try {
		const envelope = JSON.parse(atob(trimmed.slice(CODE_PREFIX.length))) as ConnectionEnvelope;

		// Stryker disable next-line OptionalChaining : the catch-all right below normalizes ANY
		// error (including the TypeError from accessing `.i` on a null/undefined envelope) into
		// an identical InvalidConnectionCodeError — swapping `?.` for `.` here does not change
		// what the public API observes.
		if (!envelope?.i || typeof envelope.a !== 'string') {
			throw new InvalidConnectionCodeError();
		}

		return {
			id: { id: envelope.i, device_id: envelope.d ?? null },
			addrs: base64ToBytes(envelope.a)
		};
	} catch {
		throw new InvalidConnectionCodeError();
	}
}

/** Truncates a long id (peer id, device id) for display: `909b713a…6511a8`. */
export function shortId(id: string, keepStart = 8, keepEnd = 6): string {
	if (id.length <= keepStart + keepEnd + 1) return id;
	return `${id.slice(0, keepStart)}…${id.slice(-keepEnd)}`;
}
