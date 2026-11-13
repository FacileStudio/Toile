import * as Y from 'yjs';
import { toast } from '@facile/muse';
import { fitBox, type Point } from '$lib/canvas/camera';
import { Editor } from '$lib/canvas/editor.svelte';
import { canComment, canEdit, commentApi, type Anchor, type BoardRole, type Comment } from './api';
import { assetUrl, uploadError, uploadImage as store } from '$lib/api/assets';
import { imageLoader, setAssets, uploadImage } from './assets';
import { Presence, type Peer, type PresenceUser } from './presence';
import { openRoom, stopMessage, type Connection, type Room, type RoomTarget, type Stop } from './provider';

/**
 * One board opened in this tab: its Y.Doc and room, the editor on it, presence, follow mode and comments.
 * Pages create one per board and `destroy` it when they leave.
 */
export class BoardSession {
	connection = $state<Connection>('connecting');
	stop = $state<Stop | null>(null);
	stopText = $state('');
	synced = $state(false);
	peers = $state.raw<Peer[]>([]);
	comments = $state.raw<Comment[]>([]);
	following = $state<number | null>(null);
	draft = $state<Anchor | null>(null);
	readonly editor: Editor;
	readonly presence: Presence;
	private room: Room;
	private comment;
	private unwatch: () => void;

	constructor(
		readonly target: RoomTarget,
		readonly role: BoardRole,
		readonly me: PresenceUser,
		readonly boardId: string
	) {
		const ydoc = new Y.Doc();
		const token = 'token' in target ? target.token : undefined;
		setAssets({
			upload: (board, file) =>
				store(board, file).catch((error: unknown) => {
					toast.danger(uploadError(error));
					return null;
				}),
			url: (board, asset) => assetUrl(board, asset, token)
		});
		this.comment = commentApi('board' in target ? { board: target.board } : { token: target.token });
		this.editor = new Editor(ydoc, {
			uploadImage: (file) => (canEdit(role) && 'board' in target ? uploadImage(boardId, file) : Promise.resolve(null)),
			imageSource: imageLoader(boardId, () => this.editor.invalidate()),
			oncomment: (at, shape) => this.startComment(at, shape)
		});
		this.editor.editable = false;
		this.editor.commentable = canComment(role);
		this.room = openRoom(target, ydoc, {
			onconnection: (state) => (this.connection = state),
			onstop: (stop, reason) => this.halt(stop, reason),
			onsynced: () => this.onsynced(),
			oncomments: () => void this.loadComments()
		});
		this.presence = new Presence(this.room.provider.awareness);
		this.presence.setUser(me);
		this.editor.presence = this.presence;
		this.unwatch = this.presence.watch(() => this.onpresence());
		void this.loadComments();
	}

	/** Closes the room and the editor. */
	destroy(): void {
		this.unwatch();
		this.presence.destroy();
		this.room.destroy();
		this.editor.destroy();
		this.editor.doc.ydoc.destroy();
	}

	/** Follows peer `id`'s viewport, or stops following with null. */
	follow(id: number | null): void {
		this.following = id;
		this.editor.onusercamera = id === null ? null : () => (this.following = null);
		this.onpresence();
	}

	/** Reloads the comment threads. */
	async loadComments(): Promise<void> {
		try {
			this.comments = await this.comment.list();
		} catch {
			this.comments = [];
		}
	}

	/** Posts a comment or a reply, then reloads. Guests sign with their display name. */
	async addComment(body: string, anchor: Anchor, parent?: string): Promise<void> {
		await this.comment.add({ anchor, body, parent, name: this.me.guest ? this.me.name : undefined });
		this.draft = null;
		await this.loadComments();
	}

	/** Resolves or reopens a thread. */
	async resolveComment(id: string, resolved: boolean): Promise<void> {
		await this.comment.resolve(id, resolved);
		await this.loadComments();
	}

	/** Deletes one of this person's comments. */
	async removeComment(id: string): Promise<void> {
		await this.comment.remove(id);
		await this.loadComments();
	}

	private startComment(at: Point, shape: string | null): void {
		if (!canComment(this.role)) return;
		const base = shape ? this.editor.scene.get(shape) : null;
		this.draft = base ? { shape: base.id, x: at.x - base.x, y: at.y - base.y } : { x: at.x, y: at.y };
	}

	private onsynced(): void {
		this.synced = true;
		this.editor.sync();
		this.editor.scene.live = true;
		this.editor.editable = canEdit(this.role) && !this.stop;
		if (this.editor.editable) this.editor.doc.ensureMeta();
	}

	private halt(stop: Stop, reason: string): void {
		this.stop = stop;
		this.stopText = stopMessage(stop, reason);
		this.editor.editable = false;
		this.editor.playing = null;
		toast.danger(this.stopText);
	}

	private onpresence(): void {
		this.peers = this.presence.peers();
		this.editor.overlay.peers = this.peers.map((peer) => ({
			id: peer.id,
			name: peer.user.name,
			color: peer.user.color,
			cursor: peer.cursor,
			selection: peer.selection
		}));
		this.editor.invalidateOverlay();
		const leader = this.peers.find((peer) => peer.id === this.following);
		if (this.following !== null && !leader) this.follow(null);
		if (leader?.viewport) {
			const { width, height } = this.editor.size;
			this.editor.setCamera(fitBox(leader.viewport, width, height, 0, 8), true);
		}
	}
}
