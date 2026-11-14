import type { Row } from '../row';

/** The suite apps Toile works with, one line each. */
export const suite = {
	'with.line': [
		'Toile hands your ideas to the other Facile apps, on the same account.',
		'Toile passe vos idées aux autres apps Facile, avec le même compte.',
		'Toile pasa tus ideas a las demás apps de Facile, con la misma cuenta.',
		'Toile gibt Ihre Ideen an die anderen Facile-Apps weiter, mit demselben Konto.'
	],
	'with.opus': [
		'Paste an Opus task: it becomes a card on the board.',
		'Collez une tâche Opus : elle devient une carte sur le tableau.',
		'Pega una tarea de Opus y se convierte en una tarjeta en la pizarra.',
		'Fügen Sie eine Opus-Aufgabe ein, sie wird zur Karte auf dem Board.'
	],
	'with.docs': [
		'Your board shows inside a Docs page.',
		'Votre tableau s’affiche dans une page Docs.',
		'Tu pizarra aparece dentro de una página de Docs.',
		'Ihr Board erscheint in einer Docs-Seite.'
	],
	'with.nuage': [
		'Board images are stored in Nuage.',
		'Les images du tableau se rangent dans Nuage.',
		'Las imágenes de la pizarra se guardan en Nuage.',
		'Bilder vom Board landen in Nuage.'
	],
	'with.bulle': [
		'Groups and sums up your stickies (Pro plan).',
		'Regroupe et résume vos post-it (offre Pro).',
		'Agrupa y resume tus notas (plan Pro).',
		'Gruppiert und fasst Ihre Haftnotizen zusammen (Pro-Tarif).'
	]
} satisfies Record<string, Row>;
