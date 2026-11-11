import { ApiError } from '@facile/client';

/** Set when a call needed the person's Facile session in another app and Registre refused it. */
export const reconnect = $state({ needed: false });

/** Whether `error` is the API's `412 failed_precondition`: the Facile session behind a token exchange expired. */
export function isReconnect(error: unknown): boolean {
	return error instanceof ApiError && error.status === 412 && error.code === 'failed_precondition';
}
