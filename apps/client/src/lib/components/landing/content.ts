import { icons } from '@facile/muse';

/** One suite app line: an icon, a title and one sentence. */
export type Feature = { icon: string; title: string; body: string };

/** The suite apps Toile works with, as the settings describe them. */
export const SUITE: readonly (Feature & { href: string })[] = [
	{
		icon: icons.folder,
		title: 'Nuage',
		href: 'https://nuage.facile.studio',
		body: 'Les images que vous déposez sur un tableau sont rangées dans votre Nuage, dossier Toile.'
	},
	{
		icon: icons.text,
		title: 'Docs',
		href: 'https://docs.facile.studio',
		body: 'Collez le lien d’un document : il devient une carte vivante, avec son titre à jour.'
	},
	{
		icon: icons.check,
		title: 'Opus',
		href: 'https://opus.facile.studio',
		body: 'Transformez un post-it en tâche Opus, sans recopier.'
	},
	{
		icon: icons.ideas,
		title: 'Bulle',
		href: 'https://bulle.facile.studio',
		body: 'Demandez à l’IA de résumer ou de regrouper vos post-it. Inclus dans les forfaits payants.'
	}
];
