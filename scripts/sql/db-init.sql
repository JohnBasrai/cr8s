-- cr8s DB schema bootstrap file
-- updated: 2025-05-25 (also see version # below)

-- ========================================
-- TEARDOWN: Clean slate approach  
-- ========================================

-- Drop tables in reverse dependency order (children first, parents last)
DROP TABLE IF EXISTS user_roles CASCADE;
DROP TABLE IF EXISTS crate CASCADE;
DROP TABLE IF EXISTS app_user CASCADE;
DROP TABLE IF EXISTS author CASCADE;
DROP TABLE IF EXISTS role CASCADE;
DROP TABLE IF EXISTS schema_version CASCADE;

-- Drop types
DROP TYPE IF EXISTS "RoleCodeMapping" CASCADE;

-- ========================================
-- SETUP: Build everything fresh
-- ========================================

CREATE TABLE schema_version (
  version TEXT NOT NULL,
  applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO schema_version (version) VALUES ('1.0.0'); -- Version here.

-- Create tables without foreign keys first
CREATE TABLE app_user (
  id SERIAL PRIMARY KEY,
  username varchar(64) NOT NULL UNIQUE,
  password varchar(128) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
);

CREATE TABLE author (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  email VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
);

-- Add foreign key after both tables exist
ALTER TABLE app_user
ADD COLUMN author_id INTEGER UNIQUE REFERENCES author(id);

CREATE TABLE crate (
  id SERIAL PRIMARY KEY,
  author_id integer NOT NULL REFERENCES author(id),
  code varchar(64) NOT NULL,
  name varchar(128) NOT NULL,
  version varchar(64) NOT NULL,
  description text,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
);



CREATE TYPE "RoleCodeMapping" AS ENUM ('Admin', 'Editor', 'Viewer');

CREATE TABLE role (
  id SERIAL PRIMARY KEY,
  code "RoleCodeMapping" NOT NULL UNIQUE,
  name varchar(128) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
);



CREATE TABLE user_roles (
  user_id INTEGER NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
  role_id INTEGER NOT NULL REFERENCES role(id) ON DELETE CASCADE,
  CONSTRAINT user_role_unique UNIQUE (user_id, role_id)
);

INSERT INTO role (code, name) VALUES
  ('Admin', 'Administrator'),
  ('Editor', 'Editor'),
  ('Viewer', 'Viewer');
