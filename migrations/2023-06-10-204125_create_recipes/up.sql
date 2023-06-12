-- Your SQL goes here
CREATE TABLE recipes (
  result_item_id INTEGER REFERENCES items(id),
  ingredient_item_id INTEGER REFERENCES items(id),
  quantity SMALLINT NOT NULL,
  PRIMARY KEY(result_item_id, ingredient_item_id)
);

CREATE INDEX result_item_index ON recipes (result_item_id);