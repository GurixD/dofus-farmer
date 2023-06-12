-- Your SQL goes here
CREATE TABLE drops (
  monster_id INTEGER REFERENCES monsters(id),
  item_id INTEGER REFERENCES items(id),
  PRIMARY KEY(monster_id, item_id)
);

CREATE INDEX item_index ON drops (item_id);