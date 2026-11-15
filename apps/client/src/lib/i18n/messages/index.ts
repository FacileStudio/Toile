import type { Catalogs, Message } from '@facile/muse';
import type { Row } from '../row';
import { demos } from './demos';
import { landing } from './landing';
import { suite } from './suite';
import { app } from './app';
import { canvas } from './canvas';
import { collab } from './collab';
import { keys } from './keys';
import { native } from './native';

const rows = { ...landing, ...demos, ...suite, ...app, ...canvas, ...collab, ...keys, ...native };

export type AppKey = keyof typeof rows;
type AppCatalog = Record<AppKey, Message>;

function column(index: number): AppCatalog {
	const entries = Object.entries(rows as Record<string, Row>).map(([key, row]) => [key, row[index]]);
	return Object.fromEntries(entries) as AppCatalog;
}

/** The app catalogs: English is the typed base, and every row carries all four locales. */
export const catalogs: Catalogs<AppCatalog> = {
	en: column(0),
	fr: column(1),
	es: column(2),
	de: column(3)
};
