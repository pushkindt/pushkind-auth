-- Your SQL goes here
CREATE VIRTUAL TABLE user_fts USING fts5(
    name,
    email,
    content='users',
    content_rowid='id',
    tokenize = 'unicode61'
);

INSERT INTO user_fts(user_fts) VALUES('rebuild');

CREATE TRIGGER users_ai AFTER INSERT ON users BEGIN
  INSERT INTO user_fts(rowid, name, email) VALUES (new.id, new.name, new.email);
END;
CREATE TRIGGER users_ad AFTER DELETE ON users BEGIN
  INSERT INTO user_fts(user_fts, rowid, name, email) VALUES('delete', old.id, old.name, old.email);
END;
CREATE TRIGGER users_au AFTER UPDATE ON users BEGIN
  INSERT INTO user_fts(user_fts, rowid, name, email) VALUES('delete', old.id, old.name, old.email);
  INSERT INTO user_fts(rowid, name, email) VALUES (new.id, new.name, new.email);
END;
