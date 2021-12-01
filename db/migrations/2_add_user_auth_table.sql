CREATE TABLE users(id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL);
CREATE INDEX users_username_idx ON users(username);

ALTER TABLE webpages ADD COLUMN user_id INTEGER NOT NULL REFERENCES users(id);
