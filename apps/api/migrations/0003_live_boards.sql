ALTER TABLE boards
	ADD COLUMN thumbnail    bytea,
	ADD COLUMN thumbnail_at timestamptz;
CREATE INDEX boards_trash_idx ON boards (deleted_at) WHERE deleted_at IS NOT NULL;

CREATE TABLE board_state (
	board_id   uuid PRIMARY KEY REFERENCES boards(id) ON DELETE CASCADE,
	snapshot   bytea NOT NULL,
	version    bigint NOT NULL DEFAULT 1,
	updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE board_updates (
	id         bigserial PRIMARY KEY,
	board_id   uuid NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
	body       bytea NOT NULL,
	created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX board_updates_board_idx ON board_updates (board_id, id);

CREATE TABLE share_links (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	board_id   uuid NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
	token_hash text NOT NULL UNIQUE,
	role       text NOT NULL CHECK (role IN ('view', 'comment', 'edit')),
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	revoked_at timestamptz
);
CREATE INDEX share_links_board_idx ON share_links (board_id) WHERE revoked_at IS NULL;

CREATE TABLE comments (
	id             uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	board_id       uuid NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
	parent_id      uuid REFERENCES comments(id) ON DELETE CASCADE,
	anchor         jsonb NOT NULL,
	body           text NOT NULL,
	author_user_id bigint REFERENCES porte_users(id) ON DELETE CASCADE,
	guest_name     text,
	share_id       uuid REFERENCES share_links(id) ON DELETE SET NULL,
	created_at     timestamptz NOT NULL DEFAULT now(),
	updated_at     timestamptz NOT NULL DEFAULT now(),
	resolved_at    timestamptz,
	CHECK ((author_user_id IS NULL) <> (guest_name IS NULL))
);
CREATE INDEX comments_board_idx ON comments (board_id, created_at);
