import { browser } from '$app/environment';
import { createTranslator, LOCALE_COOKIE, resolveLocale, type Locale } from '@facile/muse';
import { catalogs } from './messages';

let active: () => Locale = () => 'en';

function cookieLocale(): string | null {
	if (!browser) return null;
	const prefix = `${LOCALE_COOKIE}=`;
	const found = document.cookie.split('; ').find((part) => part.startsWith(prefix));
	return found ? decodeURIComponent(found.slice(prefix.length)) : null;
}

/** The locale source for `provideLocale`: the picked cookie, then the browser. */
export function localeSource(): string | null {
	return cookieLocale() ?? (browser ? navigator.language : null);
}

/** Persists the picked locale for the next visit. */
export function rememberLocale(locale: Locale) {
	document.cookie = `${LOCALE_COOKIE}=${locale}; path=/; max-age=31536000; samesite=lax`;
}

/** Points the translator at the locale `provideLocale` resolved. */
export function bindLocale(source: () => Locale) {
	active = source;
}

/** The locale currently in force. */
export function locale(): Locale {
	return resolveLocale(active());
}

/** Translates an app message in the current locale. */
export const t = createTranslator(catalogs, () => active());

export type { AppKey } from './messages';
