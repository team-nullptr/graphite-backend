CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    name text NOT NULL,
    source text NOT NULL
);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    session_id TEXT NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    name TEXT NOT NULL,
    avatar_url TEXT NOT NULL
);
