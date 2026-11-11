import { ApiError } from '@facile/client';
import { api } from '$lib/backend';

/** A stored board image, as `POST /api/boards/{id}/assets` answers it. */
export type UploadedImage = { asset: string; url: string; w: number; h: number };

/**
 * Uploads one image for board `boardId` into the uploader's Toile app folder in Nuage. Resolves the asset id the
 * image shape stores, its URL and its natural size. Throws the API's `ApiError` (402 past the plan's size, 400 for
 * anything but PNG, JPEG, GIF or WebP).
 */
export async function uploadImage(boardId: string, file: File): Promise<UploadedImage> {
	const body = new FormData();
	body.append('file', file, file.name || 'image');
	return api<UploadedImage>(`/boards/${encodeURIComponent(boardId)}/assets`, { method: 'POST', body });
}

/** Where an image shape's `asset` is read: the board route for members, the share route for a link holder. */
export function assetUrl(boardId: string, asset: string, token?: string): string {
	const id = encodeURIComponent(asset);
	return token
		? `/api/share/${encodeURIComponent(token)}/assets/${id}`
		: `/api/boards/${encodeURIComponent(boardId)}/assets/${id}`;
}

/** The toast for a failed upload: the plan's size for a 402, else the API's own message. */
export function uploadError(error: unknown): string {
	if (error instanceof ApiError && error.status === 402) {
		const limit = error.details.limit;
		return typeof limit === 'number' ? `Votre forfait limite les images à ${limit} Mo.` : error.message;
	}
	return error instanceof ApiError ? error.message : "L'image n'a pas pu être envoyée.";
}
