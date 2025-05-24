-- First define the custom PostgreSQL enum type
CREATE TYPE "RoleCodeMapping" AS ENUM ('Admin', 'Editor', 'Viewer');

-- Now create the `role` table using that enum
CREATE TABLE role (
  id SERIAL PRIMARY KEY,
  code "RoleCodeMapping" NOT NULL UNIQUE,
  name varchar(128) NOT NULL,
  created_at TIMESTAMP DEFAULT NOW() NOT NULL
);
