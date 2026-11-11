CREATE TABLE spaces (
	id         text PRIMARY KEY,
	name       text NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now(),
	deleted_at timestamptz
);

CREATE TABLE space_members (
	space_id text NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
	subject  text NOT NULL,
	email    text NOT NULL,
	name     text NOT NULL DEFAULT '',
	picture  text NOT NULL DEFAULT '',
	role     text NOT NULL CHECK (role IN ('member', 'admin', 'owner')),
	PRIMARY KEY (space_id, subject)
);
CREATE INDEX space_members_subject_idx ON space_members (subject);

CREATE TABLE directory_state (
	id      smallint PRIMARY KEY CHECK (id = 1),
	version bigint NOT NULL
);

CREATE TABLE deletion_state (
	id         smallint PRIMARY KEY CHECK (id = 1),
	state      bytea NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE boards (
	id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	owner_user_id bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	space_id      text REFERENCES spaces(id) ON DELETE CASCADE,
	name          text NOT NULL,
	created_at    timestamptz NOT NULL DEFAULT now(),
	updated_at    timestamptz NOT NULL DEFAULT now(),
	deleted_at    timestamptz,
	CHECK (space_id IS NOT NULL OR owner_user_id IS NOT NULL)
);
CREATE INDEX boards_space_idx ON boards (space_id, updated_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX boards_owner_idx ON boards (owner_user_id, updated_at DESC) WHERE space_id IS NULL AND deleted_at IS NULL;
