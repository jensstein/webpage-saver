CREATE TABLE jwt_secrets(id INTEGER PRIMARY KEY AUTOINCREMENT, secret BLOB NOT NULL, user_id INTEGER NOT NULL REFERENCES users(id));
CREATE INDEX jwt_secrets_users_id_idx ON jwt_secrets(user_id);
