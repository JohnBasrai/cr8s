-- cr8s DB schema bootstrap file
-- updated: 2025-05-25 (also see version # below)

CREATE TABLE schema_version (
  version TEXT NOT NULL,
  applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO schema_version (version) VALUES ('1.0.0'); -- Version here.

-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- CREATE TABLE app_user (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('app_user');
--

CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;



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
