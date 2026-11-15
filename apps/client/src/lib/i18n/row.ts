import type { Message } from '@facile/muse';

/** One message in every locale, in the order en, fr, es, de. */
export type Row = readonly [en: Message, fr: Message, es: Message, de: Message];
