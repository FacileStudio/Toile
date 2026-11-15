import type { Row } from '../row';

/** Working together on a board: comments, search, sharing, guests, the assistant, pasting and uploads. */
export const collab = {
	'search.shape': ['Shape', 'Forme', 'Forma', 'Form'],
	'comments.title': ['Comments', 'Commentaires', 'Comentarios', 'Kommentare'],
	'comments.close': ['Close comments', 'Fermer les commentaires', 'Cerrar los comentarios', 'Kommentare schließen'],
	'comments.guest': [' (guest)', ' (invité)', ' (invitado)', ' (Gast)'],
	'comments.write': ['Write a comment…', 'Écrire un commentaire…', 'Escribe un comentario…', 'Kommentar schreiben…'],
	'comments.new': ['New comment', 'Nouveau commentaire', 'Nuevo comentario', 'Neuer Kommentar'],
	'comments.cancel': ['Cancel', 'Annuler', 'Cancelar', 'Abbrechen'],
	'comments.post': ['Post', 'Publier', 'Publicar', 'Veröffentlichen'],
	'comments.empty': [
		'Pick the Comment tool (C), then tap the board.',
		'Choisissez l’outil Commentaire (C), puis touchez le tableau.',
		'Elige la herramienta Comentario (C) y toca la pizarra.',
		'Wählen Sie das Werkzeug Kommentar (C) und tippen Sie auf das Board.'
	],
	'comments.replyPlaceholder': ['Reply…', 'Répondre…', 'Responder…', 'Antworten…'],
	'comments.replyLabel': ['Reply', 'Réponse', 'Respuesta', 'Antwort'],
	'comments.reply': ['Reply', 'Répondre', 'Responder', 'Antworten'],
	'comments.reopen': ['Reopen', 'Rouvrir', 'Reabrir', 'Wieder öffnen'],
	'comments.resolve': ['Resolve', 'Résoudre', 'Resolver', 'Erledigen'],
	'comments.hideResolved': [
		'Hide resolved threads',
		'Masquer les fils résolus',
		'Ocultar los hilos resueltos',
		'Erledigte Threads ausblenden'
	],
	'comments.showResolved': [
		'Show resolved threads',
		'Afficher les fils résolus',
		'Mostrar los hilos resueltos',
		'Erledigte Threads anzeigen'
	],
	'comments.by': ['Comment by {author}', 'Commentaire de {author}', 'Comentario de {author}', 'Kommentar von {author}'],
	'comments.failed': ['Action failed', 'Action impossible', 'No se pudo hacer', 'Aktion fehlgeschlagen'],
	'search.title': ['Search the board', 'Rechercher dans le tableau', 'Buscar en la pizarra', 'Im Board suchen'],
	'search.placeholder': ['Text, frame, link…', 'Texte, cadre, lien…', 'Texto, marco, enlace…', 'Text, Rahmen, Link…'],
	'search.label': ['Search', 'Rechercher', 'Buscar', 'Suchen'],
	'search.results': ['Results', 'Résultats', 'Resultados', 'Ergebnisse'],
	'search.none': ['No results.', 'Aucun résultat.', 'Sin resultados.', 'Keine Ergebnisse.'],
	'search.sticky': ['Note', 'Note', 'Nota', 'Notiz'],
	'search.text': ['Text', 'Texte', 'Texto', 'Text'],
	'search.frame': ['Frame', 'Cadre', 'Marco', 'Rahmen'],
	'search.card': ['Link', 'Lien', 'Enlace', 'Link'],
	'cheatsheet.title': ['Keyboard shortcuts', 'Raccourcis clavier', 'Atajos de teclado', 'Tastenkürzel'],
	'cheatsheet.move': ['Move the selection', 'Déplacer la sélection', 'Mover la selección', 'Auswahl verschieben'],
	'cheatsheet.noSnap': ['Move without snapping', 'Déplacer sans aimantation', 'Mover sin ajuste', 'Ohne Einrasten verschieben'],
	'cheatsheet.zoom': ['Zoom', 'Zoomer', 'Zoom', 'Zoomen'],
	'cheatsheet.wheel': ['wheel', 'molette', 'rueda', 'Mausrad'],
	'cheatsheet.or': ['or', 'ou', 'o', 'oder'],
	'share.title': ['Share board', 'Partager le tableau', 'Compartir la pizarra', 'Board teilen'],
	'share.intro': [
		'Space members already have access. A link opens the board to anyone who gets it, with no account, until you revoke it. It shows only once: copy it now.',
		'Les membres de l’espace y ont déjà accès. Un lien ouvre le tableau à toute personne qui le reçoit, sans compte, jusqu’à ce que vous le révoquiez. Il n’est affiché qu’une fois : copiez-le maintenant.',
		'Los miembros del espacio ya tienen acceso. Un enlace abre la pizarra a cualquiera que lo reciba, sin cuenta, hasta que lo revoques. Solo se muestra una vez: cópialo ahora.',
		'Mitglieder des Bereichs haben bereits Zugriff. Ein Link öffnet das Board für alle, die ihn erhalten, ohne Konto, bis Sie ihn widerrufen. Er wird nur einmal angezeigt: Kopieren Sie ihn jetzt.'
	],
	'share.view': ['Read only', 'Lecture seule', 'Solo lectura', 'Nur lesen'],
	'share.comment': ['Comment', 'Commentaire', 'Comentar', 'Kommentieren'],
	'share.edit': ['Edit', 'Modification', 'Edición', 'Bearbeiten'],
	'share.role': ['Link access', 'Droit du lien', 'Permiso del enlace', 'Recht des Links'],
	'share.new': ['New link', 'Nouveau lien', 'Nuevo enlace', 'Neuer Link'],
	'share.link': ['{role} link', 'Lien {role}', 'Enlace {role}', 'Link {role}'],
	'share.copy': ['Copy link', 'Copier le lien', 'Copiar el enlace', 'Link kopieren'],
	'share.copied': ['Link copied.', 'Lien copié.', 'Enlace copiado.', 'Link kopiert.'],
	'share.copyShown': ['Copy the link shown.', 'Copiez le lien affiché.', 'Copia el enlace que se muestra.', 'Kopieren Sie den angezeigten Link.'],
	'share.links': ['Share links', 'Liens de partage', 'Enlaces compartidos', 'Freigabelinks'],
	'share.revoke': ['Revoke {role} link', 'Révoquer le lien {role}', 'Revocar el enlace {role}', 'Link {role} widerrufen'],
	'share.failed': ['Action failed', 'Action impossible', 'No se pudo hacer', 'Aktion fehlgeschlagen'],
	'guest.intro': [
		'Choose the name others will see next to your cursor.',
		'Choisissez le nom que les autres verront à côté de votre curseur.',
		'Elige el nombre que verán los demás junto a tu cursor.',
		'Wählen Sie den Namen, den andere neben Ihrem Cursor sehen.'
	],
	'guest.name': ['Your name', 'Votre nom', 'Tu nombre', 'Ihr Name'],
	'guest.open': ['Open board', 'Ouvrir le tableau', 'Abrir la pizarra', 'Board öffnen'],
	'assistant.stickies': ['Sticky', 'Post-it', 'Notas', 'Haftnotizen'],
	'assistant.stickiesHint': ['Ideas on stickies', 'Des idées en post-it', 'Ideas en notas', 'Ideen auf Haftnotizen'],
	'assistant.mindmap': ['Mind map', 'Carte mentale', 'Mapa mental', 'Mindmap'],
	'assistant.mindmapHint': ['A topic and its branches', 'Un sujet et ses branches', 'Un tema y sus ramas', 'Ein Thema und seine Zweige'],
	'assistant.flowchart': ['Flowchart', 'Logigramme', 'Diagrama de flujo', 'Flussdiagramm'],
	'assistant.flowchartHint': ['Steps and decisions', 'Des étapes et des décisions', 'Pasos y decisiones', 'Schritte und Entscheidungen'],
	'assistant.added': [
		{ one: '{name} added {count} shape to the board.', other: '{name} added {count} shapes to the board.' },
		{ one: '{name} a ajouté {count} forme au tableau.', other: '{name} a ajouté {count} formes au tableau.' },
		{ one: '{name} añadió {count} forma a la pizarra.', other: '{name} añadió {count} formas a la pizarra.' },
		{ one: '{name} hat {count} Form zum Board hinzugefügt.', other: '{name} hat {count} Formen zum Board hinzugefügt.' }
	],
	'assistant.clustered': [
		'{count} stickies sorted into {themes} themes.',
		'{count} post-it rangés en {themes} thèmes.',
		'{count} notas ordenadas en {themes} temas.',
		'{count} Haftnotizen in {themes} Themen sortiert.'
	],
	'assistant.close': ['Close', 'Fermer', 'Cerrar', 'Schließen'],
	'assistant.thinking': ['{name} is thinking…', '{name} réfléchit…', '{name} está pensando…', '{name} denkt nach…'],
	'assistant.prompt': ['What do you want to draw?', 'Que voulez-vous dessiner ?', '¿Qué quieres dibujar?', 'Was möchten Sie zeichnen?'],
	'assistant.placeholder': [
		'Ask {name} to draw, for example the back-to-school launch',
		'Demandez à {name} de dessiner, par exemple le lancement de la rentrée',
		'Pide a {name} que dibuje, por ejemplo el lanzamiento de la vuelta al cole',
		'Bitten Sie {name} zu zeichnen, zum Beispiel den Start nach den Ferien'
	],
	'assistant.kind': ['Shape', 'Forme', 'Forma', 'Form'],
	'assistant.summarize': ['Summarize the board', 'Résumer le tableau', 'Resumir la pizarra', 'Board zusammenfassen'],
	'assistant.cluster': [
		{ one: 'Sort {count} sticky', other: 'Sort {count} stickies' },
		{ one: 'Ranger {count} post-it', other: 'Ranger {count} post-it' },
		{ one: 'Ordenar {count} nota', other: 'Ordenar {count} notas' },
		{ one: '{count} Haftnotiz sortieren', other: '{count} Haftnotizen sortieren' }
	],
	'assistant.draw': ['Draw', 'Dessiner', 'Dibujar', 'Zeichnen'],
	'assistant.busy': [
		'Too many AI requests right now. Try again later.',
		'Trop de demandes à l’IA pour le moment. Réessayez plus tard.',
		'Demasiadas solicitudes a la IA por ahora. Inténtalo más tarde.',
		'Gerade zu viele KI-Anfragen. Versuchen Sie es später erneut.'
	],
	'assistant.failed': [
		'The AI could not answer.',
		'L’IA n’a pas pu répondre.',
		'La IA no pudo responder.',
		'Die KI konnte nicht antworten.'
	],
	'paste.unreadable': [
		'These copied items cannot be read: copy them again.',
		'Ces éléments copiés sont illisibles : copiez-les à nouveau.',
		'No se pueden leer estos elementos copiados: vuelve a copiarlos.',
		'Diese kopierten Elemente sind unlesbar: Kopieren Sie sie erneut.'
	],
	'image.soon': [
		'Image uploads are coming soon.',
		'L’envoi d’images arrive bientôt.',
		'La subida de imágenes llegará pronto.',
		'Das Hochladen von Bildern kommt bald.'
	],
	'image.limit': [
		'Your plan limits images to {limit} MB.',
		'Votre forfait limite les images à {limit} Mo.',
		'Tu plan limita las imágenes a {limit} MB.',
		'Ihr Tarif begrenzt Bilder auf {limit} MB.'
	],
	'image.failed': [
		'The image could not be uploaded.',
		'L’image n’a pas pu être envoyée.',
		'No se pudo subir la imagen.',
		'Das Bild konnte nicht hochgeladen werden.'
	]
} satisfies Record<string, Row>;
