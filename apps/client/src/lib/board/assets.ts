import { toast } from '@facile/muse';

/** An uploaded image: its Toile asset id and natural size. */
export type Uploaded = { asset: string; w: number; h: number };

/** Uploads one image for board `board`. Resolves null when nothing was stored. */
export type Uploader = (board: string, file: File) => Promise<Uploaded | null>;

/** The image URL of an asset, or null when it cannot be shown yet. */
export type AssetUrl = (board: string, asset: string) => string | null;

let uploader: Uploader = async () => {
	toast.info("L'envoi d'images arrive bientôt.");
	return null;
};

let assetUrl: AssetUrl = () => null;

/** Wires the real upload path (the Nuage integration); until then a paste or drop only says uploads arrive soon. */
export function setAssets(next: { upload?: Uploader; url?: AssetUrl }): void {
	if (next.upload) uploader = next.upload;
	if (next.url) assetUrl = next.url;
}

/** Uploads an image file. Images never enter the document as data URLs: only an asset id is stored. */
export function uploadImage(board: string, file: File): Promise<Uploaded | null> {
	return uploader(board, file);
}

/** A loader that turns asset ids into drawable images, calling `onload` when one finishes so the canvas repaints. */
export function imageLoader(board: string, onload: () => void): (asset: string) => CanvasImageSource | null {
	const images = new Map<string, HTMLImageElement | null>();
	return (asset) => {
		if (images.has(asset)) {
			const image = images.get(asset);
			return image && image.complete && image.naturalWidth ? image : null;
		}
		const url =
			asset.startsWith('http://') || asset.startsWith('https://') || asset.startsWith('data:')
				? asset
				: assetUrl(board, asset);
		if (!url) {
			images.set(asset, null);
			return null;
		}
		const image = new Image();
		image.onload = onload;
		image.src = url;
		if (image.complete) onload();
		images.set(asset, image);
		return image.complete && image.naturalWidth ? image : null;
	};
}
