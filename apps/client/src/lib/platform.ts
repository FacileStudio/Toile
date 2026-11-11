import { Capacitor } from '@capacitor/core';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { SecureStoragePlugin } from 'capacitor-secure-storage-plugin';

/** The API every installed app talks to. */
export const APP_API_ORIGIN = 'https://toile.facile.studio';

const TOKEN_KEY = 'toile.token';

let cachedToken: string | null | undefined;

/** True inside the Capacitor app (Android, iOS). */
export function isNative(): boolean {
	return Capacitor.isNativePlatform();
}

/** True inside the Tauri desktop app. */
export function isDesktop(): boolean {
	return isTauri();
}

/** True in either installed app, which signs in with a bearer token instead of a cookie. */
export function isApp(): boolean {
	return isNative() || isDesktop();
}

/** The origin API paths are appended to: empty on the web (same origin), production in the apps. */
export function apiOrigin(): string {
	const override = (import.meta.env.VITE_API_ORIGIN as string | undefined)?.replace(/\/$/, '');
	if (override) return override;
	return isApp() ? APP_API_ORIGIN : '';
}

/** The bearer token from the platform's secure store (Keystore, Keychain, Secret Service), or null. */
export async function getToken(): Promise<string | null> {
	if (!isApp()) return null;
	if (cachedToken !== undefined) return cachedToken;
	try {
		cachedToken = isDesktop()
			? ((await invoke<string | null>('token_get')) ?? null)
			: (await SecureStoragePlugin.get({ key: TOKEN_KEY })).value || null;
	} catch {
		cachedToken = null;
	}
	return cachedToken;
}

/** Keeps `token` in the secure store. It stays in memory for this run even if the store refuses it. */
export async function setToken(token: string): Promise<void> {
	if (!isApp()) return;
	cachedToken = token;
	try {
		if (isDesktop()) await invoke('token_set', { token });
		else await SecureStoragePlugin.set({ key: TOKEN_KEY, value: token });
	} catch (error) {
		console.warn('toile: the secure store refused the token', error);
	}
}

/** Forgets the bearer token, signing the installed app out. */
export async function clearToken(): Promise<void> {
	if (!isApp()) return;
	cachedToken = null;
	try {
		if (isDesktop()) await invoke('token_clear');
		else await SecureStoragePlugin.remove({ key: TOKEN_KEY });
	} catch {
		return;
	}
}

/**
 * The board socket's subprotocols in the installed apps: a WebSocket cannot set headers, so the
 * bearer rides in `bearer.<token>`. Empty on the web and before the first API call read the token.
 */
export function socketProtocols(): string[] {
	return isApp() && cachedToken ? ['toile', `bearer.${cachedToken}`] : [];
}
