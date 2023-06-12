use std::{collections::HashMap, fs};

use database::{connection::establish_connection, models::area::Area};
use diesel::{insert_into, PgConnection, QueryResult, RunQueryDsl};
use serde_json::{from_str, Value};

use crate::database::models::{
    drop::Drop, item::Item, map::Map, monster::Monster, monster_sub_area::MonsterSubArea,
    recipe::Recipe, sub_area::SubArea,
};

mod database;

fn main() {
    let mut connection = establish_connection();

    // Recipes.json and Items.json need to change their Nan values to null
    // MapPositions.json need to change his id to integer by removing .0
    // Simple search and replace does the job
    connection
        .build_transaction()
        .run(|mut connection| {
            let name_map = create_name_map();
            fill_areas(&mut connection, &name_map);
            fill_sub_areas(&mut connection, &name_map);
            fill_maps(&mut connection, &name_map);
            fill_items(&mut connection, &name_map);
            fill_monsters(&mut connection, &name_map);
            fill_drops(&mut connection);
            fill_recipes(&mut connection);
            fill_monsters_sub_areas(&mut connection);

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

fn fill_areas(mut connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::areas;

    println!("Starting fill_areas");

    fill_table("Areas.json", &mut connection, |v, connection| {
        let area = v.as_object().unwrap();
        let area = Area::new(
            area["id"].as_i64().unwrap() as i32,
            name_map[&(area["nameId"].as_u64().unwrap() as u32)].to_owned(),
        );
        insert_into(areas::table)
            .values(&area)
            .execute(connection)
            .unwrap();
    });

    println!("End fill_areas");
}

fn fill_sub_areas(mut connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::sub_areas;

    println!("Starting fill_sub_areas");

    fill_table("SubAreas.json", &mut connection, |v, connection| {
        let sub_area = v.as_object().unwrap();
        let sub_area = SubArea::new(
            sub_area["id"].as_i64().unwrap() as i32,
            name_map[&(sub_area["nameId"].as_u64().unwrap() as u32)].to_owned(),
            sub_area["areaId"].as_i64().unwrap() as i32,
        );

        insert_into(sub_areas::table)
            .values(sub_area)
            .execute(connection)
            .unwrap();
    });

    println!("End fill_sub_areas");
}

fn fill_maps(mut connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::maps;

    println!("Starting fill_maps");

    fill_table("MapPositions.json", &mut connection, |v, connection| {
        let map = v.as_object().unwrap();

        let world_map = map["worldMap"].as_i64().unwrap();
        if world_map == 1 {
            let map = Map::new(
                map["id"].as_i64().unwrap() as i32,
                name_map
                    .get(&(map["nameId"].as_u64().unwrap() as u32))
                    .map(|o| o.clone()),
                map["posX"].as_i64().unwrap() as i16,
                map["posY"].as_i64().unwrap() as i16,
                map["subAreaId"].as_i64().unwrap() as i32,
            );

            insert_into(maps::table)
                .values(map)
                .execute(connection)
                .unwrap();
        }
    });

    println!("End fill_maps");
}

fn fill_items(mut connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::items;

    println!("Starting fill_items");

    fill_table("Items.json", &mut connection, |v, connection| {
        let item = v.as_object().unwrap();
        let item = Item::new(
            item["id"].as_i64().unwrap() as i32,
            name_map[&(item["nameId"].as_u64().unwrap() as u32)].to_owned(),
        );

        insert_into(items::table)
            .values(item)
            .execute(connection)
            .unwrap();
    });

    println!("End fill_items");
}

fn fill_monsters(mut connection: &mut PgConnection, name_map: &HashMap<u32, String>) {
    use crate::database::schema::monsters;

    println!("Starting fill_monsters");

    fill_table("Monsters.json", &mut connection, |v, connection| {
        let monster = v.as_object().unwrap();
        let monster = Monster::new(
            monster["id"].as_i64().unwrap() as i32,
            name_map[&(monster["nameId"].as_u64().unwrap() as u32)].to_owned(),
        );

        insert_into(monsters::table)
            .values(monster)
            .execute(connection)
            .unwrap();
    });

    println!("End fill_monsters");
}

fn fill_drops(mut connection: &mut PgConnection) {
    use crate::database::schema::drops;

    println!("Starting fill_drops");

    fill_table("Items.json", &mut connection, |v, connection| {
        let item = v.as_object().unwrap();
        let id = item["id"].as_i64().unwrap() as i32;

        let mut monster_drop_ids: Vec<_> = item["dropMonsterIds"]
            .as_array()
            .unwrap_or(&Vec::new())
            .into_iter()
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
    });

    println!("End fill_drops");
}

fn fill_recipes(mut connection: &mut PgConnection) {
    use crate::database::schema::recipes;

    println!("Starting fill_recipes");

    fill_table("Recipes.json", &mut connection, |v, connection| {
        let recipe = v.as_object().unwrap();

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
    });

    println!("End fill_recipes");
}

fn fill_monsters_sub_areas(mut connection: &mut PgConnection) {
    use crate::database::schema::monsters_sub_areas;

    println!("Starting fill_monsters_sub_areas");

    fill_table("SubAreas.json", &mut connection, |v, connection| {
        let sub_area = v.as_object().unwrap();
        let id = sub_area["id"].as_i64().unwrap() as i32;

        let monster_ids: Vec<_> = sub_area["monsters"]
            .as_array()
            .unwrap_or(&Vec::new())
            .into_iter()
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
    });

    println!("End fill_monsters_sub_areas");
}

fn fill_table<F: Fn(&Value, &mut PgConnection)>(
    json_file: &str,
    mut connection: &mut PgConnection,
    function: F,
) {
    let json: Value = from_str(&read_file(&("src/resources/json/".to_owned() + json_file)))
        .expect(&format!("Can't parse {json_file}"));
    let json = json.as_array().unwrap();

    for value in json {
        function(value, &mut connection);
    }
}

fn read_file(file: &str) -> String {
    fs::read_to_string(file).unwrap()
}
