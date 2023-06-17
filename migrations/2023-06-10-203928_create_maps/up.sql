-- Your SQL goes here
CREATE TABLE maps (
  id INTEGER PRIMARY KEY,
  name VARCHAR,
  x SMALLINT NOT NULL,
  y SMALLINT NOT NULL,
  sub_area_id INTEGER REFERENCES sub_areas(id) NOT NULL,
  UNIQUE(x, y)
);