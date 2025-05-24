-- author/up.sql
CREATE TABLE author (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  email VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
);

-- Add author_id to app_user after author table exists
ALTER TABLE app_user
ADD COLUMN author_id INTEGER UNIQUE REFERENCES author(id);
