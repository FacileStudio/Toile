import { beforeEach, describe, expect, it } from 'vitest';
import { rememberReturn, takeReturn } from './returnTo';

describe('returnTo', () => {
	beforeEach(() => sessionStorage.clear());

	it('hands back a same-origin path once', () => {
		rememberReturn('/pending?action=a1');
		expect(takeReturn()).toBe('/pending?action=a1');
		expect(takeReturn()).toBeNull();
	});

	it('refuses another origin', () => {
		for (const path of ['//evil.test/x', 'https://evil.test', '/\\evil.test']) {
			rememberReturn(path);
			expect(takeReturn()).toBeNull();
		}
	});
});
