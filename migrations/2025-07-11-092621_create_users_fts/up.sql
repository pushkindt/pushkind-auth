-- Your SQL goes here
CREATE VIRTUAL TABLE user_fts USING fts5(
    name,
    email,
    content='users',
    content_rowid='id',
    tokenize = 'unicode61 remove_diacritics 2'
);
INSERT INTO user_fts(rowid, name, email)
SELECT id, name, email FROM users;
CREATE TRIGGER users_ai AFTER INSERT ON users BEGIN
    INSERT INTO user_fts(rowid, name, email) VALUES (new.id, new.name, new.email);
END;

CREATE TRIGGER users_ad AFTER DELETE ON users BEGIN
    DELETE FROM user_fts WHERE rowid = old.id;
END;

CREATE TRIGGER users_au AFTER UPDATE ON users BEGIN
    UPDATE user_fts SET name = new.name, email = new.email WHERE rowid = new.id;
END;
