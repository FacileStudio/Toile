import { Browser } from '@capacitor/browser';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import { apiOrigin, isDesktop, setToken } from '$lib/platform';

/** What the desktop shell's loopback listener heard from porte's redirect. */
export type LoopbackAnswer = { code: string; state: string; error: string };

const LOGIN_EVENT = 'toile://login';
const FAILED = 'La connexion a échoué. Réessayez.';

/** A login state in the characters porte keeps on its CLI flow (`[A-Za-z0-9_-]`, at most 128). */
export function randomState(): string {
	const bytes = crypto.getRandomValues(new Uint8Array(24));
	return btoa(String.fromCharCode(...bytes))
		.replace(/\+/g, '-')
		.replace(/\//g, '_')
		.replace(/=+$/, '');
}

/** porte's CLI sign-in URL: with `port`, Registre returns to the desktop's loopback listener; without, it shows a code to paste. */
export function loginUrl(state: string, port?: number): string {
	const query = new URLSearchParams({ flow: 'cli', cli_state: state });
	if (port) query.set('port', String(port));
	return `${apiOrigin()}/api/auth/oidc?${query}`;
}

/** The one-time code a person pasted, trimmed of spaces and line breaks, or '' when it cannot be one. */
export function cleanCode(raw: string): string {
	const code = raw.replace(/\s+/g, '');
	return /^[A-Za-z0-9_-]{16,128}$/.test(code) ? code : '';
}

/** Trades porte's one-time login code for a session token and keeps it in the secure store. */
export async function redeem(code: string): Promise<void> {
	const response = await fetch(`${apiOrigin()}/api/auth/oidc/exchange`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		credentials: 'omit',
		body: JSON.stringify({ code })
	});
	if (!response.ok) throw new Error(response.status === 401 ? 'Code invalide ou expiré.' : FAILED);
	const body = (await response.json()) as { token?: string };
	if (!body.token) throw new Error(FAILED);
	await setToken(body.token);
}

/** Checks the loopback answer against the state the app sent, then redeems its code. */
export async function finishLoopback(answer: LoopbackAnswer, state: string): Promise<void> {
	if (answer.error) throw new Error(answer.error);
	if (answer.state !== state || !answer.code) throw new Error('La connexion ne correspond pas. Réessayez.');
	await redeem(answer.code);
}

async function desktopLogin(): Promise<void> {
	const state = randomState();
	let heard: (answer: LoopbackAnswer) => void = () => undefined;
	const answer = new Promise<LoopbackAnswer>((resolve) => (heard = resolve));
	const stop = await listen<LoopbackAnswer>(LOGIN_EVENT, (event) => heard(event.payload));
	try {
		const port = await invoke<number>('login_listen');
		await openUrl(loginUrl(state, port));
		await finishLoopback(await answer, state);
	} finally {
		stop();
		await getCurrentWindow()
			.setFocus()
			.catch(() => undefined);
	}
}

/**
 * Starts sign-in in the system browser. The desktop app finishes on its own through a loopback
 * listener and resolves `true`. A phone gets a code page instead and resolves `false`: the person
 * pastes the code, which {@link redeem} trades.
 */
export async function startNativeLogin(): Promise<boolean> {
	if (isDesktop()) {
		await desktopLogin();
		return true;
	}
	await Browser.open({ url: loginUrl(randomState()) });
	return false;
}

/** Closes the in-app browser a phone sign-in opened. */
export async function closeLoginBrowser(): Promise<void> {
	if (!isDesktop()) await Browser.close().catch(() => undefined);
}
