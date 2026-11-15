import { locale } from '$lib/i18n/index.svelte';

/** A timestamp from the API as a date and time in the current locale, or the raw value when it does not parse. */
export function formatDate(value: string): string {
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return value;
	return new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(date);
}
