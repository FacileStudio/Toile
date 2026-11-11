/** The signed-in person, as `GET /api/auth/session` answers them. */
export type SessionUser = { email: string; name?: string; avatar_url?: string };

/** A Registre membership role. */
export type Role = 'member' | 'admin' | 'owner';

/** One of the caller's spaces. */
export type Space = { id: string; name: string; role: Role };

/** What the caller may do with a board. */
export type BoardRole = 'owner' | 'editor' | 'commenter' | 'viewer';

/** A board as `GET /api/boards` answers it. */
export type Board = {
	id: string;
	space_id: string | null;
	name: string;
	created_at: string;
	updated_at: string;
	role: BoardRole;
	thumbnail_url: string | null;
	deleted_at?: string;
};

/** `GET /api/me/entitlements`: the plan values of the personal scope or of one space. */
export type Entitlements = {
	scope: 'personal' | 'space';
	entitlements: { 'ai.access': boolean; 'toile.upload_mb': number };
	upgrade_url: string;
};
