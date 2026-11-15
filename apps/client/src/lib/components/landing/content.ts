import { icons } from '@facile/muse';
import type { AppKey } from '$lib/i18n/index.svelte';

/** One suite app line: an icon, a title and the catalog key of one sentence. */
export type Feature = { icon: string; title: string; body: AppKey };

/** The suite apps Toile works with, as the settings describe them. */
export const SUITE: readonly (Feature & { href: string })[] = [
	{
		icon: icons.folder,
		title: 'Nuage',
		href: 'https://nuage.facile.studio',
		body: 'with.nuage'
	},
	{
		icon: icons.text,
		title: 'Docs',
		href: 'https://docs.facile.studio',
		body: 'with.docs'
	},
	{
		icon: icons.check,
		title: 'Opus',
		href: 'https://opus.facile.studio',
		body: 'with.opus'
	},
	{
		icon: icons.ideas,
		title: 'Bulle',
		href: 'https://bulle.facile.studio',
		body: 'with.bulle'
	}
];
