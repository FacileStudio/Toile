import { layout } from '@facile/blob';

/** The `#rrggbb` body color of the blob `<Blob {name}>` draws, the same for a name every time. */
export function blobColor(name: string): string {
	return layout(name).palette.head!;
}
