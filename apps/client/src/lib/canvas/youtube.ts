const ID = /^[A-Za-z0-9_-]{11}$/;

function seconds(raw: string | null): number {
	if (!raw) return 0;
	if (/^\d+$/.test(raw)) return Number(raw);
	const parts = /^(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?$/.exec(raw);
	if (!parts) return 0;
	return Number(parts[1] ?? 0) * 3600 + Number(parts[2] ?? 0) * 60 + Number(parts[3] ?? 0);
}

function parse(raw: string): URL | null {
	try {
		const url = new URL(raw.trim());
		return url.protocol === 'https:' || url.protocol === 'http:' ? url : null;
	} catch {
		return null;
	}
}

/**
 * The id of the YouTube video `raw` points at, or null: `youtu.be/<id>`, `watch?v=<id>`, `/shorts/`, `/embed/`,
 * `/live/` or `/v/` on youtube.com, its subdomains and youtube-nocookie.com. Mirrors `youtube_id` in the API.
 */
export function youtubeId(raw: string): string | null {
	const url = parse(raw);
	if (!url) return null;
	const host = url.hostname.replace(/^(www|m)\./, '');
	const parts = url.pathname.split('/').filter(Boolean);
	const pick = (id: string | null | undefined) => (id && ID.test(id) ? id : null);
	if (host === 'youtu.be') return pick(parts[0]);
	if (host !== 'youtube.com' && !host.endsWith('.youtube.com') && host !== 'youtube-nocookie.com') return null;
	if (parts[0] === 'watch') return pick(url.searchParams.get('v'));
	return ['shorts', 'embed', 'live', 'v'].includes(parts[0]) ? pick(parts[1]) : null;
}

/** The thumbnail YouTube serves for video `id`. */
export function youtubeThumbnail(id: string): string {
	return `https://img.youtube.com/vi/${id}/hqdefault.jpg`;
}

/** The privacy-enhanced player URL for `raw`, autoplaying from its `t` or `start` offset; null when it is no video. */
export function youtubeEmbed(raw: string): string | null {
	const id = youtubeId(raw);
	if (!id) return null;
	const url = parse(raw) as URL;
	const start = seconds(url.searchParams.get('t') ?? url.searchParams.get('start'));
	const params = new URLSearchParams({ autoplay: '1', rel: '0' });
	if (start > 0) params.set('start', String(start));
	return `https://www.youtube-nocookie.com/embed/${id}?${params}`;
}
