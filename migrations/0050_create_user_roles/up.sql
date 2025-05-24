CREATE TABLE user_roles (
  id SERIAL PRIMARY KEY,
  user_id integer NOT NULL REFERENCES app_user(id),
  role_id integer NOT NULL REFERENCES role(id),
  CONSTRAINT user_role_unique UNIQUE (user_id, role_id)
);
