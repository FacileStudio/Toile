import { icons } from '@facile/muse';

/** One card of the landing: an icon, a title and one sentence. */
export type Feature = { icon: string; title: string; body: string };

/** What Toile does, in the order the landing tells it. */
export const FEATURES: readonly Feature[] = [
	{
		icon: icons.compass,
		title: 'Une toile sans bord',
		body: 'Post-it, formes, flèches, dessin, images et cadres, sans limite de place.'
	},
	{
		icon: icons.usersGroup,
		title: 'À plusieurs, en direct',
		body: 'Vous voyez les curseurs et les ajouts des autres à la seconde.'
	},
	{
		icon: icons.globe,
		title: 'Des liens pour vos invités',
		body: "Lecture, commentaire ou édition. Vos invités entrent sans créer de compte."
	},
	{
		icon: icons.download,
		title: 'Exportez, importez',
		body: 'PNG, SVG, PDF ou JSON Canvas en sortie, JSON Canvas et Excalidraw en entrée.'
	}
];

/** The suite apps Toile works with, as the landing and the settings describe them. */
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
