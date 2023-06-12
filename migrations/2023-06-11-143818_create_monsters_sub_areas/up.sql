-- Your SQL goes here
CREATE TABLE monsters_sub_areas (
  monster_id INTEGER REFERENCES monsters(id),
  sub_area_id INTEGER REFERENCES sub_areas(id),
  PRIMARY KEY(monster_id, sub_area_id)
);

CREATE INDEX monster_index ON monsters_sub_areas (monster_id);