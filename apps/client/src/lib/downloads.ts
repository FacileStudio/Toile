import { icons } from '@facile/muse';

/** Where the native builds live: the assets of the newest GitHub release. */
export const RELEASES = 'https://github.com/FacileStudio/Toile/releases/latest/download';

/** One installer, by the fixed asset name the release workflow uploads. */
export type Download = { asset: string; label: string; detail: string; icon: string };

/** Every installer offered on the landing. */
export const DOWNLOADS: readonly Download[] = [
	{ asset: 'Toile-macos-arm64.dmg', label: 'macOS', detail: 'Apple Silicon', icon: icons.apple },
	{ asset: 'Toile-macos-x64.dmg', label: 'macOS', detail: 'Intel', icon: icons.apple },
	{ asset: 'Toile-windows-x64.msi', label: 'Windows', detail: 'x64 (.msi)', icon: icons.windows },
	{ asset: 'Toile-linux-x64.deb', label: 'Linux', detail: 'Debian, Ubuntu (.deb)', icon: icons.linux },
	{ asset: 'Toile-linux-x64.AppImage', label: 'Linux', detail: 'AppImage', icon: icons.linux },
	{ asset: 'Toile-android.apk', label: 'Android', detail: 'APK', icon: icons.android }
];

/** The download URL of one asset of the latest release. */
export const downloadUrl = (asset: string): string => `${RELEASES}/${encodeURIComponent(asset)}`;
