CREATE TABLE projects (
    id SERIAL,
    name text NOT NULL
);

CREATE TABLE users (
    -- This is github user's id
    id integer PRIMARY KEY NOT NULL UNIQUE,
    name text NOT NULL,
    avatar_url text NOT NULL
);
