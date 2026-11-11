const KEY = 'toile.guest';

/** The display name this browser used as a guest, or empty. */
export function guestName(): string {
	try {
		return localStorage.getItem(KEY) ?? '';
	} catch {
		return '';
	}
}

/** Remembers the guest display name, trimmed to the server's 1-40 characters. Returns what was kept. */
export function rememberGuest(name: string): string {
	const kept = name.trim().slice(0, 40);
	try {
		localStorage.setItem(KEY, kept);
	} catch {
		return kept;
	}
	return kept;
}
