import { describe, expect, it } from 'vitest';
import { stopMessage } from './provider';

describe('stopMessage', () => {
	it('names the reason the server closed the room with', () => {
		expect(stopMessage('refused', 'access revoked')).toBe('Your access to this board was removed.');
		expect(stopMessage('refused', 'board deleted')).toBe('This board was deleted.');
		expect(stopMessage('refused', 'shape a: has an unknown key image')).toBe(
			'The server refused a change. Reload the page. Detail: shape a: has an unknown key image'
		);
		expect(stopMessage('refused', '')).toBe('The server refused a change.');
		expect(stopMessage('too-large', 'update too large')).toBe('The last change is too large.');
	});
});
