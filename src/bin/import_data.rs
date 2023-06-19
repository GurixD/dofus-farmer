use std::{collections::HashMap, fs};

use database::{connection::establish_pooled_connection, models::area::Area};
use diesel::{insert_into, PgConnection, QueryResult, RunQueryDsl};
use serde_json::{from_str, Value};

use crate::database::models::{
    drop::Drop, item::Item, map::Map, monster::Monster, monster_sub_area::MonsterSubArea,
    recipe::Recipe, sub_area::SubArea,
};

mod database;

fn main() {
    let pool = establish_pooled_connection();
    let mut connection = pool.get().unwrap();
    // let mut connection = establish_connection();

    // Recipes.json and Items.json need to change their Nan values to null
    // MapPositions.json need to change his id to integer by removing .0
    // Simple search and replace does the job
    connection
        .build_transaction()
        .run(|connection| {
            let name_map = create_name_map();
            fill_areas(connection, &name_map);
            fill_sub_areas(connection, &name_map);
            fill_maps(connection, &name_map);
            fill_items(connection, &name_map);
            fill_monsters(connection, &name_map);
            fill_drops(connection);
            fill_recipes(connection);
            fill_monsters_sub_areas(connection);

            QueryResult::Ok(())
        })
        .unwrap();
}

fn create_name_map() -> HashMap<u32, String> {
    let i18n: Value =
        from_str(include_str!("../resources/json/i18n_fr.json")).expect("Can't parse i18n");

    let names = i18n["texts"].as_object().unwrap();

    let mut name_map = HashMap::new();

    for (id, name) in names {
        let name = name.as_str().unwrap();
        if !name.is_empty() && name != "-" {
            name_map.insert(id.parse::<u32>().unwrap(), name.to_owned());
        }
    }

    name_map
}

fn fill_areas(connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::areas;

    println!("Starting fill_areas");

    fill_table("Areas.json", connection, |v, connection| {
        for area in v {
            let area = area.as_object().unwrap();
            let area = Area::new(
                area["id"].as_i64().unwrap() as i32,
                name_map[&(area["nameId"].as_u64().unwrap() as u32)].to_owned(),
            );
            insert_into(areas::table)
                .values(&area)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_areas");
}

fn fill_sub_areas(connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::sub_areas;

    println!("Starting fill_sub_areas");

    fill_table("SubAreas.json", connection, |v, connection| {
        for sub_area in v {
            let sub_area = sub_area.as_object().unwrap();
            let sub_area = SubArea::new(
                sub_area["id"].as_i64().unwrap() as i32,
                name_map[&(sub_area["nameId"].as_u64().unwrap() as u32)].to_owned(),
                sub_area["areaId"].as_i64().unwrap() as i32,
            );

            insert_into(sub_areas::table)
                .values(sub_area)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_sub_areas");
}

fn fill_maps(connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    println!("Starting fill_maps");

    use database::schema::maps;

    fill_table("MapPositions.json", connection, |v, connection| {
        for map in v {
            let map_object = map.as_object().unwrap();
            let world_map = map_object["worldMap"].as_i64().unwrap();
            if world_map == 1 {
                let map = Map::new(
                    map_object["id"].as_i64().unwrap() as i32,
                    name_map
                        .get(&(map_object["nameId"].as_u64().unwrap() as u32))
                        .cloned(),
                    map_object["posX"].as_i64().unwrap() as i16,
                    map_object["posY"].as_i64().unwrap() as i16,
                    map_object["subAreaId"].as_i64().unwrap() as i32,
                );

                let sql = insert_into(maps::table).values(&map);

                if map_object["hasPriorityOnWorldmap"].as_bool().unwrap() {
                    sql.on_conflict((maps::x, maps::y))
                        .do_update()
                        .set(&map)
                        .execute(connection)
                        .unwrap();
                } else {
                    sql.on_conflict_do_nothing().execute(connection).unwrap();
                }
            }
        }
    });

    println!("End fill_maps");
}

fn fill_items(connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::items;

    println!("Starting fill_items");

    let mut item_types_categories = HashMap::new();
    {
        let types: Vec<Value> = from_str(include_str!("../resources/json/ItemTypes.json"))
            .expect("Can't parse ItemTypes");

        for item_type in types {
            item_types_categories.insert(
                item_type["id"].as_u64().unwrap(),
                item_type["categoryId"].as_i64().unwrap() as i16,
            );
        }
    }

    fill_table("Items.json", connection, |v, connection| {
        for item in v {
            let item = item.as_object().unwrap();
            let category = item_types_categories[&item["typeId"].as_u64().unwrap()];
            // if [0, 1, 2].contains(&(category as _))
            //
            {
                let item = Item::new(
                    item["id"].as_i64().unwrap() as _,
                    name_map[&(item["nameId"].as_u64().unwrap() as _)].to_owned(),
                    category,
                    item["iconId"].as_i64().unwrap() as _,
                );

                insert_into(items::table)
                    .values(item)
                    .execute(connection)
                    .unwrap();
            }
        }
    });

    println!("End fill_items");
}

fn fill_monsters(connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::monsters;

    println!("Starting fill_monsters");

    fill_table("Monsters.json", connection, |v, connection| {
        for monster in v {
            let monster = monster.as_object().unwrap();
            let monster = Monster::new(
                monster["id"].as_i64().unwrap() as i32,
                name_map[&(monster["nameId"].as_u64().unwrap() as u32)].to_owned(),
            );

            insert_into(monsters::table)
                .values(monster)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_monsters");
}

fn fill_drops(connection: &mut PgConnection) {
    use crate::database::schema::drops;

    println!("Starting fill_drops");

    fill_table("Items.json", connection, |v, connection| {
        for item in v {
            let item = item.as_object().unwrap();
            let id = item["id"].as_i64().unwrap() as i32;

            let mut monster_drop_ids: Vec<_> = item["dropMonsterIds"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|v| v.as_i64().unwrap() as i32)
                .collect();

            // Need dedup because there might be duplicate, although I don't know why
            monster_drop_ids.sort();
            monster_drop_ids.dedup();

            let drops: Vec<_> = monster_drop_ids
                .into_iter()
                .map(|monster_id| Drop::new(monster_id, id))
                // .inspect(|drop| println!("{}, {}", drop.item_id, drop.monster_id))
                .collect();

            insert_into(drops::table)
                .values(&drops)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_drops");
}

fn fill_recipes(connection: &mut PgConnection) {
    use crate::database::schema::recipes;

    println!("Starting fill_recipes");

    fill_table("Recipes.json", connection, |v, connection| {
        for recipe in v {
            let recipe = recipe.as_object().unwrap();

            let result_id = recipe["resultId"].as_i64().unwrap() as i32;
            let ingredient_ids = recipe["ingredientIds"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i32);
            let quantities = recipe["quantities"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap() as i16);

            let results_ingredients_quantities: Vec<_> = ingredient_ids
                .zip(quantities)
                .map(|(ingredient, quantity)| Recipe::new(result_id, ingredient, quantity))
                .collect();

            insert_into(recipes::table)
                .values(&results_ingredients_quantities)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_recipes");
}

fn fill_monsters_sub_areas(connection: &mut PgConnection) {
    use crate::database::schema::monsters_sub_areas;

    println!("Starting fill_monsters_sub_areas");

    fill_table("SubAreas.json", connection, |v, connection| {
        for sub_area in v {
            let sub_area = sub_area.as_object().unwrap();
            let id = sub_area["id"].as_i64().unwrap() as i32;

            let monster_ids: Vec<_> = sub_area["monsters"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|v| v.as_i64().unwrap() as i32)
                .collect();

            // Need dedup because there might be duplicate, although I don't know why
            // monster_ids.sort();
            // monster_ids.dedup();

            let sub_area_monsters: Vec<_> = monster_ids
                .into_iter()
                .map(|monster_id| MonsterSubArea::new(monster_id, id))
                // .inspect(|drop| println!("{}, {}", drop.item_id, drop.monster_id))
                .collect();

            insert_into(monsters_sub_areas::table)
                .values(&sub_area_monsters)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_monsters_sub_areas");
}

fn fill_table<F: Fn(&Vec<Value>, &mut PgConnection)>(
    json_file: &str,
    connection: &mut PgConnection,
    function: F,
) {
    let json: Value = from_str(&read_file(&("src/resources/json/".to_owned() + json_file)))
        .unwrap_or_else(|_| panic!("Can't parse {json_file}"));
    let json = json.as_array().unwrap();

    function(json, connection);
    // for value in json {
    //     function(value, &mut connection);
    // }
}

fn read_file(file: &str) -> String {
    fs::read_to_string(file).unwrap()
}
