import type { Shape } from '$lib/board/schema';

/** The default font size of each text-bearing type, in world units. */
export function fontSize(shape: Shape): number {
	return shape.style.size ?? (shape.type === 'text' ? 20 : shape.type === 'sticky' ? 18 : 16);
}

/** The inner padding between a shape's box and its text, in world units. */
export function textPadding(shape: Shape): number {
	return shape.type === 'text' ? 2 : shape.type === 'sticky' ? 14 : 10;
}

/** The CSS font of a shape at its world size. */
export function fontOf(shape: Shape, family: string): string {
	const weight = shape.style.font === 'bold' ? 600 : 500;
	return `${weight} ${fontSize(shape)}px ${family}`;
}

function breakWord(ctx: CanvasRenderingContext2D, word: string, width: number): string[] {
	const out: string[] = [];
	let line = '';
	for (const char of word) {
		if (line && ctx.measureText(line + char).width > width) {
			out.push(line);
			line = '';
		}
		line += char;
	}
	return line ? [...out, line] : out;
}

/** Wraps `text` into lines no wider than `width` with the context's current font; long words break by character. */
export function wrap(ctx: CanvasRenderingContext2D, text: string, width: number): string[] {
	const lines: string[] = [];
	for (const paragraph of text.split('\n')) {
		let line = '';
		for (const word of paragraph.split(/(\s+)/)) {
			const next = line + word;
			if (!line || ctx.measureText(next).width <= width) {
				line = next;
				continue;
			}
			lines.push(line.trimEnd());
			line = word.trimStart();
			if (ctx.measureText(line).width > width) {
				const pieces = breakWord(ctx, line, width);
				line = pieces.pop() ?? '';
				lines.push(...pieces);
			}
		}
		lines.push(line);
	}
	return lines;
}

const cache = new WeakMap<Shape, string[]>();

/** Draws a shape's text in its box: wrapped, aligned, vertically centred except in text shapes. */
export function drawText(ctx: CanvasRenderingContext2D, shape: Shape, color: string, family: string): void {
	if (!shape.text) return;
	const pad = textPadding(shape);
	const size = fontSize(shape);
	const width = Math.max(1, shape.w - pad * 2);
	ctx.font = fontOf(shape, family);
	let lines = cache.get(shape);
	if (!lines) cache.set(shape, (lines = wrap(ctx, shape.text, width)));
	const lineHeight = size * 1.3;
	const align = shape.style.align ?? (shape.type === 'text' ? 'left' : 'center');
	const top =
		shape.type === 'text' ? shape.y + pad : shape.y + Math.max(pad, (shape.h - lines.length * lineHeight) / 2);
	const x = align === 'left' ? shape.x + pad : align === 'right' ? shape.x + shape.w - pad : shape.x + shape.w / 2;
	ctx.fillStyle = color;
	ctx.textAlign = align;
	ctx.textBaseline = 'middle';
	const max = Math.max(1, Math.floor((shape.h - pad) / lineHeight) + (shape.type === 'text' ? 1000 : 0));
	for (let i = 0; i < Math.min(lines.length, max); i += 1) ctx.fillText(lines[i], x, top + lineHeight * (i + 0.5));
}

/** The height a text shape needs for its content at width `w`. */
export function textHeight(ctx: CanvasRenderingContext2D, shape: Shape, family: string): number {
	ctx.font = fontOf(shape, family);
	const lines = wrap(ctx, shape.text ?? '', Math.max(1, shape.w - textPadding(shape) * 2));
	return Math.ceil(lines.length * fontSize(shape) * 1.3 + textPadding(shape) * 2);
}
