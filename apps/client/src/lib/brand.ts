import { registerIcons } from '@facile/muse';

/** The brand mark of Toile: a sticky note peeling over another, the one bold-duotone glyph in the rail. */
export const logoIcon = 'toile:logo';

/** A board, in lists and empty states. */
export const boardIcon = 'toile:board';

const back =
	'M2 6.5A4.5 4.5 0 0 1 6.5 2h7A4.5 4.5 0 0 1 18 6.5V10h-5.5A2.5 2.5 0 0 0 10 12.5V18H6.5A4.5 4.5 0 0 1 2 13.5Z';
const front = 'M12 14a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v2.2h-2.8a2.6 2.6 0 0 0-2.6 2.6V22H14a2 2 0 0 1-2-2Z';
const fold = 'M22 17.4 17.4 22v-2.9a1.7 1.7 0 0 1 1.7-1.7Z';
const lines = 'M6 6.5h6M6 10h2.5';
const ink = 'M6 17.5c2-2.5 3.5-1 5-.5s2.5-1 3.5-3 2.5-2.5 4.5-1';
const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"';

registerIcons({
	[logoIcon]: {
		body: [
			`<path fill="currentColor" opacity=".5" d="${back}"/>`,
			`<path fill="currentColor" d="${front}"/>`,
			`<path fill="currentColor" opacity=".5" d="${fold}"/>`,
			`<path ${stroke} stroke-width="1.6" d="${lines}"/>`
		].join(''),
		width: 24,
		height: 24
	},
	[boardIcon]: {
		body: `<g ${stroke} stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="4"/><path d="M7 8h4v4H7Z"/><path d="${ink}"/></g>`,
		width: 24,
		height: 24
	}
});
