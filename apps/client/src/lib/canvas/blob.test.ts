import { describe, expect, it } from 'vitest';
import { blobColor } from './blob';

describe('blobColor', () => {
	it('answers one stable #rrggbb per name', () => {
		expect(blobColor('Pixel')).toMatch(/^#[0-9a-f]{6}$/);
		expect(blobColor('Pixel')).toBe(blobColor('Pixel'));
		expect(blobColor('Bulle')).not.toBe(blobColor('Pixel'));
	});
});
