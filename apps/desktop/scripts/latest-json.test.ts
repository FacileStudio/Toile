import { describe, expect, test } from 'bun:test';
import { ASSETS, buildLatest, changelogSection, versionOf } from './latest-json';

const CHANGELOG = `# Changelog

## [Unreleased]

## [1.5.0] - 2026-10-27

### Added

- Self-update.

## [1.4.0] - 2026-10-25

### Added

- Sidebar.
`;

function signatures(): Record<string, string> {
	const out: Record<string, string> = {};
	for (const asset of new Set(Object.values(ASSETS))) {
		out[asset] = `sig-of-${asset}\n`;
	}
	return out;
}

const NOW = new Date('2026-10-27T09:30:15.123Z');

describe('changelogSection', () => {
	test('returns the body of the matching version only', () => {
		expect(changelogSection(CHANGELOG, '1.5.0')).toBe('### Added\n\n- Self-update.');
		expect(changelogSection(CHANGELOG, '1.4.0')).toBe('### Added\n\n- Sidebar.');
	});

	test('returns an empty string for an unknown version', () => {
		expect(changelogSection(CHANGELOG, '9.9.9')).toBe('');
	});
});

describe('buildLatest', () => {
	const latest = buildLatest({
		tag: 'desktop-v1.5.0',
		repo: 'FacileStudio/Toile',
		notes: 'notes',
		now: NOW,
		signatures: signatures()
	});

	test('has the manifest fields', () => {
		expect(latest.version).toBe('1.5.0');
		expect(latest.notes).toBe('notes');
		expect(latest.pub_date).toBe('2026-10-27T09:30:15Z');
	});

	test('lists every platform with asset urls and signature text', () => {
		expect(Object.keys(latest.platforms).sort()).toEqual([
			'darwin-aarch64',
			'darwin-x86_64',
			'linux-x86_64',
			'linux-x86_64-deb',
			'windows-x86_64'
		]);
		const deb = latest.platforms['linux-x86_64-deb'];
		expect(deb.url).toBe(
			'https://github.com/FacileStudio/Toile/releases/download/desktop-v1.5.0/Toile-linux-x64.deb'
		);
		expect(deb.signature).toBe('sig-of-Toile-linux-x64.deb');
		expect(latest.platforms['darwin-aarch64'].url).not.toBe(latest.platforms['darwin-x86_64'].url);
	});

	test('strips the desktop tag prefix', () => {
		expect(versionOf('desktop-v0.2.0')).toBe('0.2.0');
	});

	test('refuses a missing signature or a tag that is not a version', () => {
		const missing = { ...signatures(), 'Toile-windows-x64.msi': ' ' };
		expect(() =>
			buildLatest({ tag: 'desktop-v1.5.0', repo: 'a/b', notes: '', now: NOW, signatures: missing })
		).toThrow();
		expect(() =>
			buildLatest({ tag: 'nightly', repo: 'a/b', notes: '', now: NOW, signatures: signatures() })
		).toThrow();
	});

	test('falls back to a pointer when the changelog has no section', () => {
		const bare = buildLatest({
			tag: 'desktop-v1.5.0',
			repo: 'a/b',
			notes: '',
			now: NOW,
			signatures: signatures()
		});
		expect(bare.notes).toContain('desktop-v1.5.0');
	});
});
