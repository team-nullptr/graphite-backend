CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    name text NOT NULL
);

CREATE TABLE sessions (
    id INT PRIMARY KEY AUTO_INCREMENT NOT NULL,
    session_id TEXT NOT NULL,
    name TEXT NOT NULL,
    avatar_url TEXT NOT NULL
)