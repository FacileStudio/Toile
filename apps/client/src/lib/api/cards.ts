import { api } from '$lib/backend';

/** A link resolved for the viewer: a suite item's title and status, or a page's Open Graph title. */
export type ResolvedCard = {
	kind: 'docs' | 'feuilles' | 'opus' | 'agenda' | 'nuage' | 'link';
	title: string;
	subtitle?: string;
	status?: string;
	icon: string;
	url: string;
};

/** Resolves `url` through `POST /api/cards/resolve`. */
export function resolveCard(url: string): Promise<ResolvedCard> {
	return api<ResolvedCard>('/cards/resolve', { method: 'POST', body: JSON.stringify({ url }) });
}
