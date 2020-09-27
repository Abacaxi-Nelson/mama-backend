CREATE TABLE users_families (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL REFERENCES users(id),
  family_id uuid NOT NULL REFERENCES families(id),
  role VARCHAR NOT NULL,
  created_at timestamp NOT NULL
);