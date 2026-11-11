CREATE TABLE entitlement_state (
	id         smallint PRIMARY KEY CHECK (id = 1),
	document   text NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now()
);
