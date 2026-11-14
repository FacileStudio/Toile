const KEY = 'toile.return';

function local(path: string): boolean {
	return path.startsWith('/') && !path.startsWith('//') && !path.startsWith('/\\');
}

/** Remembers `path`, a same-origin path, as the page to open after the next sign-in. */
export function rememberReturn(path: string): void {
	try {
		if (local(path)) sessionStorage.setItem(KEY, path);
	} catch {
		return;
	}
}

/** The path `rememberReturn` kept, once, or null. */
export function takeReturn(): string | null {
	try {
		const path = sessionStorage.getItem(KEY);
		sessionStorage.removeItem(KEY);
		return path && local(path) ? path : null;
	} catch {
		return null;
	}
}
