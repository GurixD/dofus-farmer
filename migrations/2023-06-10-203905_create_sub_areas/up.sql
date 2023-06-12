-- Your SQL goes here
CREATE TABLE sub_areas (
  id INTEGER PRIMARY KEY,
  name VARCHAR NOT NULL,
  area_id INTEGER REFERENCES areas(id) NOT NULL
);