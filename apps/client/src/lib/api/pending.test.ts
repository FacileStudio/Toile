import { ApiError } from '@facile/client';
import { describe, expect, it } from 'vitest';
import { OWN_SESSION_REFUSAL, notOwnSession, targetBoard } from './pending';

describe('targetBoard', () => {
	it('reads the board of a delete and refuses anything else', () => {
		expect(targetBoard({ target: '{"board_id":"b1","ids":["s"]}' })).toBe('b1');
		expect(targetBoard({ target: 'not json' })).toBeNull();
		expect(targetBoard({ target: '{"board_id":3}' })).toBeNull();
	});
});

describe('notOwnSession', () => {
	it('spots only the own-session refusal', () => {
		expect(notOwnSession(new ApiError(403, 'permission_denied', OWN_SESSION_REFUSAL))).toBe(true);
		expect(notOwnSession(new ApiError(403, 'permission_denied', 'forbidden'))).toBe(false);
		expect(notOwnSession(new Error(OWN_SESSION_REFUSAL))).toBe(false);
	});
});
