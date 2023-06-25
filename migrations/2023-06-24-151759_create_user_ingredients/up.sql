-- Your SQL goes here
CREATE TABLE user_ingredients (
    item_id INTEGER PRIMARY KEY REFERENCES items(id),
    quantity SMALLINT NOT NULL
);