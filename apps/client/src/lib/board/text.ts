import type * as Y from 'yjs';
import { LIMITS } from './schema';

/** The smallest single splice that turns `before` into `after`: `[index, deleted, inserted]`. */
export function splice(before: string, after: string): [number, number, string] {
	let start = 0;
	const max = Math.min(before.length, after.length);
	while (start < max && before[start] === after[start]) start += 1;
	let end = 0;
	while (end < max - start && before[before.length - 1 - end] === after[after.length - 1 - end]) end += 1;
	return [start, before.length - start - end, after.slice(start, after.length - end)];
}

/** Brings `text` to `value` with one splice, so a peer's concurrent typing elsewhere in it survives. */
export function setText(text: Y.Text, value: string): void {
	const [index, deleted, inserted] = splice(text.toString(), value.slice(0, LIMITS.textChars));
	if (deleted === 0 && inserted === '') return;
	if (deleted > 0) text.delete(index, deleted);
	if (inserted) text.insert(index, inserted);
}
