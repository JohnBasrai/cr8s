CREATE TABLE crates (
  id SERIAL PRIMARY KEY,
  rustacean_id integer NOT NULL REFERENCES rustaceans(id),
  code varchar(64) NOT NULL,
  name varchar(128) NOT NULL,
  version varchar(64) NOT NULL,
  description text,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
)
