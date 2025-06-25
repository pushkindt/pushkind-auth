-- Your SQL goes here
CREATE TABLE menu (
    id INTEGER NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    url VARCHAR NOT NULL,
    hub_id INTEGER NOT NULL REFERENCES hubs(id)
);
