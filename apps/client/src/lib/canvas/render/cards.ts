import type { Shape } from '$lib/board/schema';
import type { DrawEnv } from './shapes';

function truncate(ctx: CanvasRenderingContext2D, text: string, maxW: number): string {
	if (maxW <= 0) return '';
	if (ctx.measureText(text).width <= maxW) return text;
	let low = 0;
	let high = text.length;
	while (low < high) {
		const mid = (low + high + 1) >> 1;
		if (ctx.measureText(text.slice(0, mid) + '…').width <= maxW) low = mid;
		else high = mid - 1;
	}
	return text.slice(0, low) + '…';
}

function drawPlayButton(ctx: CanvasRenderingContext2D, cx: number, cy: number, topH: number): void {
	const r = Math.min(22, topH * 0.28);
	ctx.beginPath();
	ctx.arc(cx, cy, r, 0, Math.PI * 2);
	ctx.fillStyle = 'rgba(0, 0, 0, 0.65)';
	ctx.fill();
	const s = r * 0.45;
	ctx.beginPath();
	ctx.moveTo(cx - s * 0.6, cy - s);
	ctx.lineTo(cx + s * 1.1, cy);
	ctx.lineTo(cx - s * 0.6, cy + s);
	ctx.closePath();
	ctx.fillStyle = '#ffffff';
	ctx.fill();
}

function drawThumbnail(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv, topH: number): void {
	ctx.save();
	ctx.beginPath();
	ctx.roundRect(shape.x, shape.y, shape.w, shape.h, 10);
	ctx.clip();
	ctx.fillStyle = env.palette.component;
	ctx.fillRect(shape.x, shape.y, shape.w, topH);
	if (shape.image) {
		const img = env.image(shape.image);
		if (img) {
			const [iw, ih] =
				img instanceof HTMLImageElement
					? [img.naturalWidth || shape.w, img.naturalHeight || topH]
					: [shape.w, topH];
			const scale = Math.max(shape.w / iw, topH / ih);
			const dw = iw * scale;
			const dh = ih * scale;
			ctx.imageSmoothingEnabled = true;
			ctx.drawImage(img, shape.x + (shape.w - dw) / 2, shape.y + (topH - dh) / 2, dw, dh);
		}
	}
	if (shape.kind === 'youtube') drawPlayButton(ctx, shape.x + shape.w / 2, shape.y + topH / 2, topH);
	ctx.restore();
}

function drawExternalIcon(ctx: CanvasRenderingContext2D, x: number, y: number, color: string): void {
	ctx.save();
	ctx.strokeStyle = color;
	ctx.lineWidth = 1.3;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	ctx.beginPath();
	ctx.moveTo(x + 2, y + 10);
	ctx.lineTo(x + 10, y + 2);
	ctx.moveTo(x + 5, y + 2);
	ctx.lineTo(x + 10, y + 2);
	ctx.lineTo(x + 10, y + 7);
	ctx.stroke();
	ctx.restore();
}

function drawFavicon(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv, x: number, y: number): number {
	if (shape.favicon) {
		const fav = env.image(shape.favicon);
		if (fav) {
			ctx.drawImage(fav, x, y, 16, 16);
			return x + 22;
		}
	}
	ctx.save();
	ctx.strokeStyle = env.palette.muted;
	ctx.lineWidth = 1.2;
	ctx.beginPath();
	ctx.arc(x + 8, y + 8, 6.5, 0, Math.PI * 2);
	ctx.moveTo(x + 1.5, y + 8);
	ctx.lineTo(x + 14.5, y + 8);
	ctx.stroke();
	ctx.restore();
	return x + 20;
}

function hostOf(shape: Shape): string {
	if (shape.site) return shape.site;
	if (!shape.url) return '';
	try {
		return new URL(shape.url).hostname.replace(/^www\./, '');
	} catch {
		return shape.url;
	}
}

function drawFooter(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv, pad: number): void {
	const footerY = shape.y + shape.h - pad - 16;
	const favX = shape.x + pad;
	const nameX = drawFavicon(ctx, shape, env, favX, footerY);
	const arrowX = shape.x + shape.w - pad - 12;
	drawExternalIcon(ctx, arrowX, footerY + 2, env.palette.muted);
	const host = hostOf(shape);
	if (host) {
		const maxW = Math.max(0, arrowX - 8 - nameX);
		ctx.font = `500 11px ${env.palette.font}`;
		ctx.fillStyle = env.palette.muted;
		ctx.textAlign = 'left';
		ctx.textBaseline = 'middle';
		ctx.fillText(truncate(ctx, host, maxW), nameX, footerY + 8);
	}
}

/** Draws a live card plate with thumbnail banner, metadata, and link icon. */
export function drawCard(ctx: CanvasRenderingContext2D, shape: Shape, env: DrawEnv): void {
	ctx.save();
	ctx.beginPath();
	ctx.roundRect(shape.x, shape.y, shape.w, shape.h, 10);
	ctx.fillStyle = env.palette.surface;
	ctx.fill();
	const topH = shape.image ? Math.min(shape.h * 0.6, 120) : 0;
	if (topH > 0) drawThumbnail(ctx, shape, env, topH);
	if (12 * env.zoom >= 3) {
		const pad = 12;
		const maxW = shape.w - pad * 2;
		const textY = shape.y + topH + 10;
		const footerY = shape.y + shape.h - pad - 16;
		const title = shape.title || shape.url || '';
		ctx.font = `600 13px ${env.palette.font}`;
		ctx.fillStyle = env.palette.fg;
		ctx.textAlign = 'left';
		ctx.textBaseline = 'top';
		ctx.fillText(truncate(ctx, title, maxW), shape.x + pad, textY);
		const sub = shape.description || (!shape.image ? shape.url : '');
		if (sub && textY + 18 + 12 <= footerY) {
			ctx.font = `400 11px ${env.palette.font}`;
			ctx.fillStyle = env.palette.muted;
			ctx.fillText(truncate(ctx, sub, maxW), shape.x + pad, textY + 18);
		}
		drawFooter(ctx, shape, env, pad);
	}
	ctx.restore();
}
