-- Your SQL goes here
CREATE TABLE items (
  id INTEGER PRIMARY KEY,
  name VARCHAR NOT NULL,
  category SMALLINT NOT NULL,
  image_id INTEGER NOT NULL
);

-- Done in database as superuser
-- https://stackoverflow.com/a/11007216 

-- CREATE EXTENSION unaccent;

-- CREATE OR REPLACE FUNCTION public.immutable_unaccent(regdictionary, text)
--   RETURNS text
--   LANGUAGE c IMMUTABLE PARALLEL SAFE STRICT AS
-- '$libdir/unaccent', 'unaccent_dict';

-- CREATE OR REPLACE FUNCTION public.f_unaccent(text)
--   RETURNS text
--   LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
--   BEGIN ATOMIC
-- SELECT public.immutable_unaccent(regdictionary 'public.unaccent', $1);
-- END;

CREATE INDEX items_unaccent_name_index ON items(public.f_unaccent(name));