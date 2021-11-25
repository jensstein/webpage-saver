CREATE TABLE webpages(url TEXT NOT NULL, text TEXT NOT NULL, html TEXT NOT NULL);
CREATE INDEX webpage_url_idx ON webpages(url);
