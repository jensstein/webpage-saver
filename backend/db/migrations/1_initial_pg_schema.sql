CREATE TABLE users(
    id BIGSERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);
CREATE INDEX users_username_idx ON users(username);

CREATE TABLE webpages(
    id BIGSERIAL PRIMARY KEY,
    url TEXT NOT NULL,
    text TEXT NOT NULL,
    html TEXT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    image_url TEXT,
    added TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);
CREATE INDEX webpage_url_idx ON webpages(url);

CREATE TABLE jwt_secrets(
    id BIGSERIAL PRIMARY KEY,
    secret TEXT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id)
);
CREATE INDEX jwt_secrets_users_id_idx ON jwt_secrets(user_id);

CREATE TABLE tags (
        id BIGSERIAL PRIMARY KEY,
        tag TEXT UNIQUE NOT NULL
);
CREATE INDEX tags_tag_idx ON tags (tag);
CREATE TABLE tags_to_webpages (
        id BIGSERIAL PRIMARY KEY,
        tag_id BIGINT NOT NULL REFERENCES tags(id),
        webpage_id BIGINT NOT NULL REFERENCES webpages(id)
);
CREATE INDEX tags_to_webpages_tag_idx ON tags_to_webpages (tag_id);
CREATE INDEX tags_to_webpages_webpage_idx ON tags_to_webpages (webpage_id);
