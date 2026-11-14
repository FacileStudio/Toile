import type { Row } from '../row';

/** The words inside the landing demos. */
export const demos = {
	'hero.window': ['Sprint retro', 'Rétro du sprint', 'Retro del sprint', 'Sprint-Retro'],
	'hero.share': ['Share', 'Partager', 'Compartir', 'Teilen'],
	'hero.ideas': ['Ideas', 'Idées', 'Ideas', 'Ideen'],
	'hero.todo': ['To do', 'À faire', 'Por hacer', 'Aufgaben'],
	'hero.export': ['PDF export', 'Export PDF', 'Exportar PDF', 'PDF-Export'],
	'hero.onboarding': ['Onboarding', 'Onboarding', 'Onboarding', 'Onboarding'],
	'hero.beta': ['Launch the beta', 'Lancer la bêta', 'Lanzar la beta', 'Beta starten'],
	'hero.note': ['by Friday', 'pour vendredi', 'para el viernes', 'bis Freitag'],
	'hero.line': ['+ PNG and SVG', '+ PNG et SVG', '+ PNG y SVG', '+ PNG und SVG'],
	'hero.label': [
		'Demo: three people work on the same board at once. Léa moves a sticky, Malik writes in another, you draw an arrow.',
		'Démonstration : trois personnes travaillent en même temps sur le même tableau. Léa déplace un post-it, Malik écrit dans un autre, vous tracez une flèche.',
		'Demostración: tres personas trabajan a la vez en la misma pizarra. Léa mueve una nota, Malik escribe en otra y tú trazas una flecha.',
		'Demo: Drei Personen arbeiten gleichzeitig am selben Board. Léa verschiebt eine Haftnotiz, Malik schreibt in eine andere, Sie ziehen einen Pfeil.'
	],
	'comment.title': ['Comment right on the board', 'Commentez sur place', 'Comenta en el sitio', 'Direkt am Board kommentieren'],
	'comment.body': [
		'A thread placed on the board, right where the idea is.',
		'Un fil de discussion posé sur le tableau, là où se trouve l’idée.',
		'Un hilo de conversación en la pizarra, justo donde está la idea.',
		'Ein Gesprächsfaden direkt auf dem Board, genau bei der Idee.'
	],
	'comment.panel': ['Comments', 'Commentaires', 'Comentarios', 'Kommentare'],
	'comment.sticky': ['New logo', 'Nouveau logo', 'Nuevo logo', 'Neues Logo'],
	'comment.question': ['Keep this title?', 'On garde ce titre ?', '¿Dejamos este título?', 'Titel so lassen?'],
	'comment.answer': ['Yes, approved.', 'Oui, validé.', 'Sí, aprobado.', 'Ja, passt.'],
	'cluster.title': ['Bulle sorts your ideas', 'Bulle range vos idées', 'Bulle ordena tus ideas', 'Bulle ordnet Ihre Ideen'],
	'cluster.body': [
		'Select some stickies and the AI groups them by theme in frames.',
		'Sélectionnez des post-it, l’IA les regroupe par thème dans des cadres.',
		'Selecciona notas y la IA las agrupa por tema en marcos.',
		'Wählen Sie Haftnotizen aus, die KI gruppiert sie nach Thema in Rahmen.'
	],
	'cluster.badge': ['Pro plan', 'Offre Pro', 'Plan Pro', 'Pro-Tarif'],
	'cluster.button': ['Sort 4 stickies', 'Ranger 4 post-it', 'Ordenar 4 notas', '4 Notizen ordnen'],
	'cluster.welcome': ['Onboarding', 'Accueil', 'Bienvenida', 'Einstieg'],
	'cluster.payment': ['Payment', 'Paiement', 'Pago', 'Zahlung'],
	'opus.title': ['Your Opus tasks on the board', 'Vos tâches Opus sur le tableau', 'Tus tareas de Opus en la pizarra', 'Ihre Opus-Aufgaben auf dem Board'],
	'opus.body': [
		'Paste an Opus task link: it becomes a card with its title.',
		'Collez le lien d’une tâche Opus : il devient une carte avec son titre.',
		'Pega el enlace de una tarea de Opus y se convierte en una tarjeta con su título.',
		'Fügen Sie den Link einer Opus-Aufgabe ein, er wird zur Karte mit ihrem Titel.'
	],
	'opus.sticky': ['Pricing page', 'Page tarifs', 'Página de precios', 'Preisseite'],
	'opus.task': ['Redo the pricing page', 'Refaire la page tarifs', 'Rehacer la página de precios', 'Preisseite überarbeiten']
} satisfies Record<string, Row>;
