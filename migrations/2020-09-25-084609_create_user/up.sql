CREATE TABLE users (
  id uuid PRIMARY KEY,
  tel VARCHAR NOT NULL,
  nom VARCHAR NOT NULL,
  email VARCHAR NOT NULL,
  created_at timestamp NOT NULL
);