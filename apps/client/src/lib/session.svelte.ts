import { createSession } from '@facile/client/session';
import { forgetAssistants } from './api/ai';
import { api, backend } from './backend';
import { clearToken } from './platform';
import type { SessionUser, Space } from './types';

const SPACE_KEY = 'toile.space';

function remembered(): string | null {
	try {
		return localStorage.getItem(SPACE_KEY);
	} catch {
		return null;
	}
}

function remember(id: string | null): void {
	try {
		if (id) localStorage.setItem(SPACE_KEY, id);
		else localStorage.removeItem(SPACE_KEY);
	} catch {
		return;
	}
}

/** The signed-in person, their spaces, and the active scope (`null` is personal). */
class Session {
	account = createSession<SessionUser>(api, { mePath: '/auth/session' });
	spaces = $state<Space[]>([]);
	spaceId = $state<string | null>(null);
	ready = $state(false);

	user = $derived(this.account.user);
	space = $derived(this.spaces.find((space) => space.id === this.spaceId) ?? null);

	async load(): Promise<void> {
		try {
			if (await this.account.refresh()) this.spaces = await backend.spaces();
		} catch {
			this.spaces = [];
		}
		const saved = remembered();
		this.spaceId = this.spaces.some((space) => space.id === saved) ? saved : null;
		this.ready = true;
	}

	select(id: string | null): void {
		this.spaceId = id;
		remember(id);
	}

	async logout(): Promise<void> {
		try {
			await this.account.logout();
		} finally {
			await clearToken();
			forgetAssistants();
			this.spaces = [];
		}
	}
}

/** The one session of the app. */
export const session = new Session();
