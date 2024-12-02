# Extract data
## Json
Use PyDofus: [PyDofus](https://github.com/balciseri/PyDofus) or this [fork](https://github.com/GurixD/PyDofus)

Get this list of files in the input directory:
- Areas.d2o
- Dungeons.d2o
- Items.d2o
- ItemSets.d2o
- ItemTypes.d2o
- MapCoordinates.d2o
- MapReferences.d2o
- MonsterMiniBoss.d2o
- Monsters.d2o
- MonsterSuperRaces.d2o
- RandomDropGroups.d2o
- Recipes.d2o
- SubAreas.d2o
- WorldMaps.d2o

They're located in ```$DOFUS_PATH/data/common```.  
  
Run this command to unpack them to json:
```bash
python d2o_unpack.py
```
Now the jsons are located in output. Put them in ```src/resources/json```.

Unpack the i18n_fr.d2i with the command ```python d2i_unpack.py $FILE```, the json output will be in the same directory of the input. The file is located in ```$DOFUS_PATH/data/i18n```. Put the json in the same json directory as before.

## Images
Find the items images file in ```$DOFUS_PATH/content/gfx/items``` and unpack all ```bitmap*.d2p``` with ```python d2p_unpack.py``` and put all the images in ```src/resources/images/items```. There should be a bit more than ~10k images.  
  
Do the same with monsters and worldmap: from ```$DOFUS_PATH/content/gfx/monsters``` into ```src/resources/images/monsters``` and ```$DOFUS_PATH/content/gfx/maps``` into ```src/resources/images/worldmap``` (for this, keep only the folder "1" which means the main world map, and under it all the sizes: "0.2", "0.4", "0.6", "0.8", "1").

Add an environment variable DOFUS_RESOURCES pointing to the "resources" directory.

# Database
setup postgresql container:
```docker-compose
services:
  postgres:
    image: postgres:latest
    ports:
      - 49237:5432
    volumes:
      - ~/apps/postgres:/var/lib/postgresql/data
    environment:
      - POSTGRES_PASSWORD=dofus_database_secret
      - POSTGRES_USER=dofus_user
    restart: always
```
```docker compose up -d```  
  
If you need to connect to the database:  
```psql -p 49237 -U dofus_user``` 

Add this to .env for diesel_cli:
```DATABASE_URL=postgres://dofus_user:dofus_database_secret@localhost:49237/dofus_farmer```

# Disesel
## Installation
Install libpq [Postgresql (libpq)](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads), choose command line tools  
Set the environment variable PQ_LIB_DIR (ex: D:\Program Files\PostgreSQL\17\lib)
Install diesel_cli :  
```cargo install diesel_cli --no-default-features --features postgres```

## Migrations
Create the function:  
```
-- Done in database as superuser
-- https://stackoverflow.com/a/11007216 

CREATE DATABASE dofus_farmer;

\c dofus_farmer;

CREATE EXTENSION unaccent;

CREATE OR REPLACE FUNCTION public.immutable_unaccent(regdictionary, text)
  RETURNS text
  LANGUAGE c IMMUTABLE PARALLEL SAFE STRICT AS '$libdir/unaccent', 'unaccent_dict';

CREATE OR REPLACE FUNCTION public.f_unaccent(text)
  RETURNS text
  LANGUAGE sql IMMUTABLE PARALLEL SAFE STRICT
  BEGIN ATOMIC
SELECT public.immutable_unaccent(regdictionary 'public.unaccent', $1);
END;
```
  
Generate tables:  
```
diesel setup
diesel migration run
```  
Populate database:
```
cargo run --bin import_data
```