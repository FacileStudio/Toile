const DIGITS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
const BASE = DIGITS.length;
const MID = DIGITS[BASE >> 1];

const digit = (char: string | undefined): number => (char === undefined ? 0 : DIGITS.indexOf(char));

function after(a: string): string {
	if (a === '') return MID;
	const d = digit(a[0]);
	return d < BASE - 1 ? DIGITS[d + 1] : DIGITS[d] + after(a.slice(1));
}

function before(b: string): string {
	if (b === '') return MID;
	const d = digit(b[0]);
	return d > 1 ? DIGITS[d - 1] : DIGITS[0] + before(b.slice(1));
}

function midpoint(a: string, b: string): string {
	let n = 0;
	while ((a[n] ?? DIGITS[0]) === b[n]) n += 1;
	if (n > 0) return b.slice(0, n) + midpoint(a.slice(n), b.slice(n));
	const da = digit(a[0]);
	const db = digit(b[0]);
	if (db - da > 1) return DIGITS[Math.round((da + db) / 2)];
	if (b.length > 1) return b[0];
	return DIGITS[da] + after(a.slice(1));
}

/**
 * A fractional index strictly between `a` and `b` (null is an open end). Keys sort as plain strings and never end
 * in `0`, so a key always fits between any two. A misordered pair (concurrent writers) yields a key after `a`.
 */
export function keyBetween(a: string | null, b: string | null): string {
	if (a === null && b === null) return MID;
	if (b === null) return after(a as string);
	if (a === null) return before(b);
	return a < b ? midpoint(a, b) : after(a);
}

/** `n` ascending keys strictly between `a` and `b`, split in halves so their length grows with log(n). */
export function keysBetween(a: string | null, b: string | null, n: number): string[] {
	if (n <= 0) return [];
	const mid = keyBetween(a, b);
	const left = (n - 1) >> 1;
	return [...keysBetween(a, mid, left), mid, ...keysBetween(mid, b, n - 1 - left)];
}

/** Orders shapes by fractional index, with the id breaking ties two writers may create. */
export function compareZ(a: { z: string; id: string }, b: { z: string; id: string }): number {
	if (a.z !== b.z) return a.z < b.z ? -1 : 1;
	return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
}
