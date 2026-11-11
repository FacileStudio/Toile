CREATE TABLE assets (
	id          uuid PRIMARY KEY,
	board_id    uuid NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
	uploader_id bigint NOT NULL REFERENCES porte_users(id) ON DELETE CASCADE,
	nuage_key   text NOT NULL,
	mime        text NOT NULL,
	size        bigint NOT NULL,
	width       integer NOT NULL,
	height      integer NOT NULL,
	sha256      text NOT NULL,
	created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX assets_board_idx ON assets (board_id);
