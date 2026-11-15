import { describe, expect, it } from 'vitest';
import { catalogs } from './messages';

const locales = ['fr', 'es', 'de'] as const;

function placeholders(message: unknown): string {
	const texts = typeof message === 'string' ? [message] : Object.values(message as Record<string, string>);
	const names = texts.flatMap((text) => [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]));
	return [...new Set(names)].sort().join(',');
}

describe('message catalogs', () => {
	for (const locale of locales) {
		it(`${locale} has every English key with the same placeholders`, () => {
			const problems: string[] = [];
			for (const [key, english] of Object.entries(catalogs.en)) {
				const message = (catalogs[locale] as Record<string, unknown>)[key];
				if (!message) problems.push(`${key}: empty`);
				else if (placeholders(message) !== placeholders(english)) problems.push(`${key}: placeholders differ`);
			}
			expect(problems).toEqual([]);
		});
	}

	it('has no em dash in any locale', () => {
		const dashed = Object.entries(catalogs).flatMap(([locale, catalog]) =>
			Object.entries(catalog)
				.filter(([, message]) => JSON.stringify(message).includes('—'))
				.map(([key]) => `${locale}.${key}`)
		);
		expect(dashed).toEqual([]);
	});
});
