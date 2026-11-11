import { registerIcons } from '@facile/muse';

/** The brand mark of Toile, the one bold-duotone glyph in the rail. */
export const logoIcon = 'toile:logo';

/** A board, in lists and empty states. */
export const boardIcon = 'toile:board';

const board =
	'M2 7c0-2.4 0-3.5.7-4.3S4.6 2 7 2h10c2.4 0 3.5 0 4.3.7s.7 1.9.7 4.3v10c0 2.4 0 3.5-.7 4.3S19.4 22 17 22H7c-2.4 0-3.5 0-4.3-.7S2 19.4 2 17Z';
const sticky = 'M6 7.5C6 6.7 6.7 6 7.5 6h3c.8 0 1.5.7 1.5 1.5v3c0 .8-.7 1.5-1.5 1.5h-3C6.7 12 6 11.3 6 10.5Z';
const ink = 'M6 17.5c2-2.5 3.5-1 5-.5s2.5-1 3.5-3 2.5-2.5 4.5-1';
const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"';

registerIcons({
	[logoIcon]: {
		body: `<path fill="currentColor" opacity=".5" d="${board}"/><path fill="currentColor" d="${sticky}"/><path ${stroke} stroke-width="1.8" d="${ink}"/>`,
		width: 24,
		height: 24
	},
	[boardIcon]: {
		body: `<g ${stroke} stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="4"/><path d="M7 8h4v4H7Z"/><path d="${ink}"/></g>`,
		width: 24,
		height: 24
	}
});
