import type { Row } from '../row';

/** The signed-in shell and its pages: navigation, boards, settings, the agent confirm page and errors. */
export const app = {
	'nav.boards': ['Boards', 'Tableaux', 'Pizarras', 'Boards'],
	'nav.account': ['Account', 'Compte', 'Cuenta', 'Konto'],
	'nav.personal': ['Personal', 'Personnel', 'Personal', 'Persönlich'],
	'reconnect.title': [
		'Reconnect your Facile session',
		'Reconnectez votre session Facile',
		'Vuelve a conectar tu sesión de Facile',
		'Verbinden Sie Ihre Facile-Sitzung erneut'
	],
	'reconnect.body': [
		'Your Facile session expired. Sign in again to use the other suite apps from Toile.',
		'Votre session Facile a expiré. Reconnectez-vous pour utiliser les autres apps de la suite depuis Toile.',
		'Tu sesión de Facile caducó. Vuelve a iniciar sesión para usar las demás apps de la suite desde Toile.',
		'Ihre Facile-Sitzung ist abgelaufen. Melden Sie sich erneut an, um die anderen Apps der Suite aus Toile zu nutzen.'
	],
	'reconnect.confirm': ['Sign in again', 'Se reconnecter', 'Volver a iniciar sesión', 'Erneut anmelden'],
	'reconnect.later': ['Later', 'Plus tard', 'Más tarde', 'Später'],
	'common.cancel': ['Cancel', 'Annuler', 'Cancelar', 'Abbrechen'],
	'common.save': ['Save', 'Enregistrer', 'Guardar', 'Speichern'],
	'common.delete': ['Delete', 'Supprimer', 'Eliminar', 'Löschen'],
	'common.home': ['Back to home', 'Retour à l’accueil', 'Volver al inicio', 'Zurück zur Startseite'],
	'common.untitled': ['Untitled', 'Sans titre', 'Sin título', 'Ohne Titel'],
	'boards.title': ['Boards', 'Tableaux', 'Pizarras', 'Boards'],
	'boards.personal': ['Personal space', 'Espace personnel', 'Espacio personal', 'Persönlicher Bereich'],
	'boards.name': ['Board name', 'Nom du tableau', 'Nombre de la pizarra', 'Name des Boards'],
	'boards.namePlaceholder': ['Monday workshop', 'Atelier du lundi', 'Taller del lunes', 'Workshop am Montag'],
	'boards.new': ['New board', 'Nouveau tableau', 'Nueva pizarra', 'Neues Board'],
	'boards.create': ['Create a board', 'Créer un tableau', 'Crear una pizarra', 'Board erstellen'],
	'boards.empty': ['No boards', 'Aucun tableau', 'No hay pizarras', 'Keine Boards'],
	'boards.emptyBody': [
		'Create a first board to get started.',
		'Créez un premier tableau pour commencer.',
		'Crea una primera pizarra para empezar.',
		'Erstellen Sie ein erstes Board, um loszulegen.'
	],
	'boards.updated': ['Edited {date}', 'Modifié le {date}', 'Editada el {date}', 'Bearbeitet am {date}'],
	'boards.actions': ['Actions for {name}', 'Actions pour {name}', 'Acciones para {name}', 'Aktionen für {name}'],
	'boards.open': ['Open', 'Ouvrir', 'Abrir', 'Öffnen'],
	'boards.rename': ['Rename', 'Renommer', 'Renombrar', 'Umbenennen'],
	'boards.duplicate': ['Duplicate', 'Dupliquer', 'Duplicar', 'Duplizieren'],
	'boards.renameTitle': ['Rename board', 'Renommer le tableau', 'Renombrar la pizarra', 'Board umbenennen'],
	'boards.nameHelper': ['1 to 200 characters', '1 à 200 caractères', 'De 1 a 200 caracteres', '1 bis 200 Zeichen'],
	'boards.renamed': ['Board renamed.', 'Tableau renommé.', 'Pizarra renombrada.', 'Board umbenannt.'],
	'boards.trashed': [
		'Board moved to the trash.',
		'Tableau placé dans la corbeille.',
		'Pizarra movida a la papelera.',
		'Board in den Papierkorb verschoben.'
	],
	'boards.deleteTitle': ['Delete “{name}”?', 'Supprimer « {name} » ?', '¿Eliminar «{name}»?', '„{name}“ löschen?'],
	'boards.deleteBody': [
		'The board goes to the trash; its share links stop working.',
		'Le tableau part dans la corbeille ; ses liens de partage cessent de fonctionner.',
		'La pizarra va a la papelera y sus enlaces compartidos dejan de funcionar.',
		'Das Board kommt in den Papierkorb, seine Freigabelinks funktionieren nicht mehr.'
	],
	'boards.loadFailed': ['Could not load', 'Chargement impossible', 'No se pudo cargar', 'Laden fehlgeschlagen'],
	'boards.createFailed': ['Could not create', 'Création impossible', 'No se pudo crear', 'Erstellen fehlgeschlagen'],
	'boards.renameFailed': ['Could not rename', 'Renommage impossible', 'No se pudo renombrar', 'Umbenennen fehlgeschlagen'],
	'boards.copyFailed': ['Could not copy', 'Copie impossible', 'No se pudo copiar', 'Kopieren fehlgeschlagen'],
	'boards.missing': ['Board not found', 'Tableau introuvable', 'Pizarra no encontrada', 'Board nicht gefunden'],
	'boards.missingBody': [
		'It was deleted, or you do not have access to it.',
		'Il a été supprimé, ou vous n’y avez pas accès.',
		'Se eliminó o no tienes acceso.',
		'Es wurde gelöscht oder Sie haben keinen Zugriff.'
	],
	'boards.shared': ['Shared board', 'Tableau partagé', 'Pizarra compartida', 'Geteiltes Board'],
	'boards.invalidLink': ['Invalid link', 'Lien invalide', 'Enlace no válido', 'Ungültiger Link'],
	'boards.invalidLinkBody': [
		'This link was revoked or does not exist.',
		'Ce lien a été révoqué ou n’existe pas.',
		'Este enlace se revocó o no existe.',
		'Dieser Link wurde widerrufen oder existiert nicht.'
	],
	'error.missing': ['Page not found', 'Page introuvable', 'Página no encontrada', 'Seite nicht gefunden'],
	'error.title': ['Error', 'Erreur', 'Error', 'Fehler'],
	'error.failed': ['Something went wrong', 'Une erreur est survenue', 'Algo salió mal', 'Etwas ist schiefgelaufen'],
	'error.missingBody': [
		'This address leads to no page in Toile.',
		'Cette adresse ne mène à aucune page de Toile.',
		'Esta dirección no lleva a ninguna página de Toile.',
		'Diese Adresse führt zu keiner Seite von Toile.'
	],
	'error.failedBody': [
		'Reload the page. If the error comes back, try again later.',
		'Rechargez la page. Si l’erreur revient, réessayez plus tard.',
		'Recarga la página. Si el error vuelve, inténtalo más tarde.',
		'Laden Sie die Seite neu. Wenn der Fehler wiederkommt, versuchen Sie es später.'
	],
	'settings.title': ['Settings', 'Réglages', 'Ajustes', 'Einstellungen'],
	'settings.profile': ['Profile', 'Profil', 'Perfil', 'Profil'],
	'settings.appearance': ['Appearance', 'Apparence', 'Apariencia', 'Darstellung'],
	'settings.shortcuts': ['Shortcuts', 'Raccourcis', 'Atajos', 'Tastenkürzel'],
	'settings.apps': ['Apps', 'Applications', 'Aplicaciones', 'Apps'],
	'settings.plan': ['Plan', 'Forfait', 'Plan', 'Tarif'],
	'settings.member': ['Member', 'Membre', 'Miembro', 'Mitglied'],
	'settings.admin': ['Admin', 'Administrateur', 'Administrador', 'Administrator'],
	'settings.owner': ['Owner', 'Propriétaire', 'Propietario', 'Eigentümer'],
	'settings.appsTitle': ['Connected apps', 'Applications connectées', 'Aplicaciones conectadas', 'Verbundene Apps'],
	'settings.appsBody': [
		'Toile acts in these apps with your Facile account, only when you ask on a board.',
		'Toile agit dans ces applications avec votre compte Facile, uniquement quand vous le demandez sur un tableau.',
		'Toile actúa en estas aplicaciones con tu cuenta de Facile, solo cuando lo pides en una pizarra.',
		'Toile handelt in diesen Apps mit Ihrem Facile-Konto, nur wenn Sie es auf einem Board verlangen.'
	],
	'settings.manageAccess': ['Manage access', 'Gérer les accès', 'Gestionar accesos', 'Zugriffe verwalten'],
	'settings.openApp': ['Open {app}', 'Ouvrir {app}', 'Abrir {app}', '{app} öffnen'],
	'settings.planBody': [
		'Toile is free, with no limit on boards. A paid plan adds AI with Bulle and larger uploads.',
		'Toile est gratuit, sans limite de tableaux. Un forfait payant ajoute l’IA avec Bulle et des imports plus lourds.',
		'Toile es gratis, sin límite de pizarras. Un plan de pago añade la IA con Bulle y subidas más grandes.',
		'Toile ist kostenlos, ohne Begrenzung der Boards. Ein bezahlter Tarif fügt KI mit Bulle und größere Uploads hinzu.'
	],
	'settings.manageSubscription': ['Manage subscription', 'Gérer l’abonnement', 'Gestionar la suscripción', 'Abo verwalten'],
	'settings.plans': ['Facile plans', 'Les forfaits Facile', 'Los planes de Facile', 'Die Facile-Tarife'],
	'pending.title': ['Agent request', 'Demande d’un agent', 'Solicitud de un agente', 'Anfrage eines Agenten'],
	'pending.intro': [
		'An agent wants to make a change that needs your approval.',
		'Un agent veut faire une modification qui demande votre accord.',
		'Un agente quiere hacer un cambio que necesita tu aprobación.',
		'Ein Agent möchte eine Änderung vornehmen, die Ihre Zustimmung braucht.'
	],
	'pending.refused': [
		'To confirm, sign in again in your browser. The mobile or desktop app, the CLI and API tokens cannot confirm an agent request.',
		'Pour confirmer, reconnectez-vous dans votre navigateur. L’app mobile ou de bureau, la CLI et les jetons d’API ne peuvent pas confirmer une demande d’agent.',
		'Para confirmar, vuelve a iniciar sesión en tu navegador. La app móvil o de escritorio, la CLI y los tokens de API no pueden confirmar una solicitud de agente.',
		'Melden Sie sich zum Bestätigen erneut im Browser an. Die Mobil- oder Desktop-App, die CLI und API-Tokens können keine Agentenanfrage bestätigen.'
	],
	'pending.signIn': ['Sign in again', 'Se reconnecter', 'Volver a iniciar sesión', 'Erneut anmelden'],
	'pending.applied': ['Change applied', 'Modification appliquée', 'Cambio aplicado', 'Änderung übernommen'],
	'pending.declined': ['Request declined', 'Demande refusée', 'Solicitud rechazada', 'Anfrage abgelehnt'],
	'pending.appliedBody': [
		'The shapes were removed from the board.',
		'Les formes ont été supprimées du tableau.',
		'Las formas se eliminaron de la pizarra.',
		'Die Formen wurden vom Board entfernt.'
	],
	'pending.declinedBody': ['The board stays as it is.', 'Le tableau reste tel quel.', 'La pizarra queda igual.', 'Das Board bleibt, wie es ist.'],
	'pending.openBoard': ['Open board', 'Ouvrir le tableau', 'Abrir la pizarra', 'Board öffnen'],
	'pending.missing': ['Request not found', 'Demande introuvable', 'Solicitud no encontrada', 'Anfrage nicht gefunden'],
	'pending.missingBody': [
		'It was already confirmed, declined or expired, or it is not in your name.',
		'Elle a déjà été confirmée, refusée ou a expiré, ou elle n’est pas à votre nom.',
		'Ya se confirmó, se rechazó o caducó, o no está a tu nombre.',
		'Sie wurde bereits bestätigt, abgelehnt oder ist abgelaufen, oder sie läuft nicht auf Ihren Namen.'
	],
	'pending.client': ['Client {id}', 'Client {id}', 'Cliente {id}', 'Client {id}'],
	'pending.expires': ['Expires {date}', 'Expire le {date}', 'Caduca el {date}', 'Läuft ab am {date}'],
	'pending.viewBoard': ['View board', 'Voir le tableau', 'Ver la pizarra', 'Board ansehen'],
	'pending.unavailable': ['Requests unavailable', 'Demandes indisponibles', 'Solicitudes no disponibles', 'Anfragen nicht verfügbar'],
	'pending.answerFailed': ['Could not answer', 'Réponse impossible', 'No se pudo responder', 'Antwort fehlgeschlagen'],
	'pending.ready': ['Ready', 'Prête', 'Lista', 'Bereit'],
	'pending.changed': [
		'The board changed since the request: confirming will fail',
		'Le tableau a changé depuis la demande : la confirmation échouera',
		'La pizarra cambió desde la solicitud: la confirmación fallará',
		'Das Board hat sich seit der Anfrage geändert: Die Bestätigung schlägt fehl'
	],
	'pending.gone': [
		'The board no longer exists or you no longer have access to it',
		'Le tableau n’existe plus ou vous n’y avez plus accès',
		'La pizarra ya no existe o ya no tienes acceso',
		'Das Board existiert nicht mehr oder Sie haben keinen Zugriff mehr'
	],
} satisfies Record<string, Row>;
