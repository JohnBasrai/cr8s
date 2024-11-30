CREATE TABLE roles (
  id SERIAL PRIMARY KEY,
  code varchar(64) NOT NULL UNIQUE,
  name varchar(128) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
)
