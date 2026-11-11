import { USER_COLORS } from '@facile/muse';
import type { Awareness } from 'y-protocols/awareness';
import type { Box, Point } from '$lib/canvas/camera';

/** Who a peer is, as the awareness `user` field carries it. */
export type PresenceUser = { name: string; color: string; picture?: string; guest: boolean };

/** One peer's awareness state (`docs/board-schema.md`, presence). */
export type Peer = {
	id: number;
	user: PresenceUser;
	cursor: Point | null;
	selection: string[];
	viewport: Box | null;
};

/** A stable identity colour for a name, from the suite palette. */
export function colorFor(seed: string): string {
	let hash = 0;
	for (const char of seed) hash = (hash * 31 + char.charCodeAt(0)) | 0;
	return USER_COLORS[Math.abs(hash) % USER_COLORS.length];
}

function asPeer(id: number, state: Record<string, unknown>): Peer | null {
	const user = state.user as PresenceUser | undefined;
	if (!user || typeof user.name !== 'string') return null;
	return {
		id,
		user,
		cursor: (state.cursor as Point | null) ?? null,
		selection: Array.isArray(state.selection) ? (state.selection as string[]) : [],
		viewport: (state.viewport as Box | null) ?? null
	};
}

/** The local presence of one board: writes this client's fields, reads every other client's. */
export class Presence {
	private pending: Record<string, unknown> = {};
	private frame = 0;

	constructor(readonly awareness: Awareness) {}

	/** Publishes who this client is. */
	setUser(user: PresenceUser): void {
		this.awareness.setLocalStateField('user', user);
	}

	/** Queues a presence field; queued fields go out together once per animation frame. */
	set(field: 'cursor' | 'selection' | 'viewport', value: unknown): void {
		this.pending[field] = value;
		if (this.frame) return;
		this.frame = requestAnimationFrame(() => {
			this.frame = 0;
			const current = this.awareness.getLocalState() ?? {};
			this.awareness.setLocalState({ ...current, ...this.pending });
			this.pending = {};
		});
	}

	/** Every other client with a user, in client id order. */
	peers(): Peer[] {
		const out: Peer[] = [];
		this.awareness.getStates().forEach((state, id) => {
			if (id === this.awareness.clientID) return;
			const peer = asPeer(id, state);
			if (peer) out.push(peer);
		});
		return out.sort((a, b) => a.id - b.id);
	}

	/** Calls `fn` whenever another client's presence changes (never for this client's own). Returns the unsubscribe. */
	watch(fn: () => void): () => void {
		const handler = (change: { added: number[]; updated: number[]; removed: number[] }) => {
			const ids = [...change.added, ...change.updated, ...change.removed];
			if (ids.some((id) => id !== this.awareness.clientID)) fn();
		};
		this.awareness.on('change', handler);
		return () => this.awareness.off('change', handler);
	}

	/** Stops the pending frame. */
	destroy(): void {
		cancelAnimationFrame(this.frame);
	}
}
