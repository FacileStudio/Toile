import { describe, expect, it } from 'vitest';
import { targetBoard } from './pending';

describe('targetBoard', () => {
	it('reads the board of a delete and refuses anything else', () => {
		expect(targetBoard({ target: '{"board_id":"b1","ids":["s"]}' })).toBe('b1');
		expect(targetBoard({ target: 'not json' })).toBeNull();
		expect(targetBoard({ target: '{"board_id":3}' })).toBeNull();
	});
});
