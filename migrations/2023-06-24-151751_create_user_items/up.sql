-- Your SQL goes here
CREATE TABLE user_items (
    item_id INTEGER PRIMARY KEY REFERENCES items(id),
    quantity SMALLINT NOT NULL
);