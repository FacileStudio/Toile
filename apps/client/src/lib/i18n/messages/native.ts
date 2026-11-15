import type { Row } from '../row';

/** The desktop and mobile app: the intro slides and the sign-in screen. */
export const native = {
	'native.skip': ['Skip', 'Passer', 'Omitir', 'Überspringen'],
	'native.slide1Title': ['A board without edges', 'Un tableau sans bords', 'Una pizarra sin bordes', 'Ein Board ohne Ränder'],
	'native.slide1Body': [
		'Stickies, shapes, arrows and freehand drawing on an infinite canvas.',
		'Post-it, formes, flèches et dessin à main levée sur une toile infinie.',
		'Notas, formas, flechas y dibujo a mano alzada en un lienzo infinito.',
		'Haftnotizen, Formen, Pfeile und Freihandzeichnen auf einer unendlichen Fläche.'
	],
	'native.slide2Title': ['Together, live', 'À plusieurs, en direct', 'En equipo, en directo', 'Gemeinsam, live'],
	'native.slide2Body': [
		'See the others’ cursors and draw together, at the office or on the go.',
		'Voyez les curseurs des autres et dessinez ensemble, au bureau comme en déplacement.',
		'Ve los cursores de los demás y dibujad juntos, en la oficina o de viaje.',
		'Sehen Sie die Cursor der anderen und zeichnen Sie gemeinsam, im Büro oder unterwegs.'
	],
	'native.slide3Title': ['With a finger or a stylus', 'Au doigt ou au stylet', 'Con el dedo o el lápiz', 'Mit Finger oder Stift'],
	'native.slide3Body': [
		'Pinch to zoom, swipe with two fingers, write with a pressure-sensitive stylus.',
		'Pincez pour zoomer, glissez à deux doigts, écrivez au stylet avec la pression.',
		'Pellizca para hacer zoom, desliza con dos dedos, escribe con un lápiz sensible a la presión.',
		'Zum Zoomen zusammenziehen, mit zwei Fingern wischen, mit drucksensitivem Stift schreiben.'
	],
	'native.carousel': ['carousel', 'carrousel', 'carrusel', 'Karussell'],
	'native.slide': ['slide', 'diapositive', 'diapositiva', 'Folie'],
	'native.slides': ['Slides', 'Diapositives', 'Diapositivas', 'Folien'],
	'native.slideN': ['Slide {n}', 'Diapositive {n}', 'Diapositiva {n}', 'Folie {n}'],
	'native.goTo': ['Go to slide {n}', 'Aller à la diapositive {n}', 'Ir a la diapositiva {n}', 'Zu Folie {n}'],
	'native.signIn': ['Sign in', 'Connectez-vous', 'Inicia sesión', 'Anmelden'],
	'native.pasteCode': [
		'Once signed in in the browser, copy the code shown and paste it here.',
		'Une fois connecté dans le navigateur, copiez le code affiché et collez-le ici.',
		'Cuando inicies sesión en el navegador, copia el código que aparece y pégalo aquí.',
		'Kopieren Sie nach der Anmeldung im Browser den angezeigten Code und fügen Sie ihn hier ein.'
	],
	'native.account': [
		'Your Facile account opens your boards and your spaces’ boards.',
		'Votre compte Facile ouvre vos tableaux et ceux de vos espaces.',
		'Tu cuenta de Facile abre tus pizarras y las de tus espacios.',
		'Ihr Facile-Konto öffnet Ihre Boards und die Ihrer Bereiche.'
	],
	'native.code': ['Sign-in code', 'Code de connexion', 'Código de acceso', 'Anmeldecode'],
	'native.submit': ['Confirm', 'Valider', 'Confirmar', 'Bestätigen'],
	'native.reopen': ['Reopen sign-in', 'Rouvrir la connexion', 'Volver a abrir el acceso', 'Anmeldung erneut öffnen'],
	'native.codeMissing': [
		'Paste the code shown after signing in.',
		'Collez le code affiché après la connexion.',
		'Pega el código que aparece al iniciar sesión.',
		'Fügen Sie den Code ein, der nach der Anmeldung angezeigt wird.'
	],
	'native.failed': ['Sign-in failed. Try again.', 'La connexion a échoué. Réessayez.', 'El acceso falló. Inténtalo de nuevo.', 'Anmeldung fehlgeschlagen. Versuchen Sie es erneut.'],
	'native.invalidCode': ['Invalid or expired code.', 'Code invalide ou expiré.', 'Código no válido o caducado.', 'Ungültiger oder abgelaufener Code.'],
	'native.mismatch': [
		'The sign-in does not match. Try again.',
		'La connexion ne correspond pas. Réessayez.',
		'El acceso no coincide. Inténtalo de nuevo.',
		'Die Anmeldung passt nicht. Versuchen Sie es erneut.'
	]
} satisfies Record<string, Row>;
