import type { DownloadEntry } from '@facile/muse';

/** Where the native builds live: the assets of the newest GitHub release. */
export const RELEASES = 'https://github.com/FacileStudio/Toile/releases';

const asset = (name: string): string => `${RELEASES}/latest/download/${encodeURIComponent(name)}`;

/** Every installer offered on the landing, by the fixed asset name the release workflow uploads. */
export const DOWNLOADS: DownloadEntry[] = [
	{ platform: 'macos', href: asset('Toile-macos-arm64.dmg'), label: 'Apple Silicon', arch: 'arm64' },
	{ platform: 'macos', href: asset('Toile-macos-x64.dmg'), label: 'Intel', arch: 'x64' },
	{ platform: 'windows', href: asset('Toile-windows-x64.msi'), label: '.msi', arch: 'x64' },
	{ platform: 'linux', href: asset('Toile-linux-x64.deb'), label: '.deb', arch: 'x64' },
	{ platform: 'linux', href: asset('Toile-linux-x64.AppImage'), label: 'AppImage', arch: 'x64' },
	{ platform: 'android', href: asset('Toile-android.apk'), label: 'APK' }
];
