import { describe, expect, it } from 'vitest';
import { stopMessage } from './provider';

describe('stopMessage', () => {
	it('names the reason the server closed the room with', () => {
		expect(stopMessage('refused', 'access revoked')).toBe('Votre accès à ce tableau a été retiré.');
		expect(stopMessage('refused', 'board deleted')).toBe('Ce tableau a été supprimé.');
		expect(stopMessage('refused', 'shape a: has an unknown key image')).toBe(
			'Le serveur a refusé une modification. Rechargez la page. Détail : shape a: has an unknown key image'
		);
		expect(stopMessage('refused', '')).toBe('Le serveur a refusé une modification.');
		expect(stopMessage('too-large', 'update too large')).toBe('La dernière modification est trop volumineuse.');
	});
});
