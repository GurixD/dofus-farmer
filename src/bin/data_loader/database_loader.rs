use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::mpsc::Sender,
};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::Context;
use lombok::AllArgsConstructor;

use crate::{
    database::models::{
        drop::Drop,
        item::{Item, ItemList},
        map::Map,
        monster::Monster,
        monster_sub_area::MonsterSubArea,
        sub_area::SubArea,
        user_ingredient::UserIngredient,
        user_item::UserItem,
    },
    gui::{image::Image, main_window::Ingredients},
};
use diesel::prelude::*;

use super::data_loader::DataLoader;

#[derive(AllArgsConstructor)]
pub struct DatabaseLoader {
    pool: Pool<ConnectionManager<PgConnection>>,
    item_tx: Sender<(Item, i16)>,
}

impl DataLoader for DatabaseLoader {
    fn new(pool: Pool<ConnectionManager<PgConnection>>, item_tx: Sender<(Item, i16)>) -> Self {
        Self { pool, item_tx }
    }

    fn load_all_sub_areas(
        &self,
    ) -> std::collections::HashMap<
        crate::database::models::sub_area::SubArea,
        Vec<crate::database::models::map::Map>,
    > {
        use crate::database::schema::sub_areas;
        use diesel::prelude::*;

        let mut connection = self.pool.get().unwrap();

        let sub_areas = sub_areas::table
            .select(SubArea::as_select())
            .load(&mut connection)
            .unwrap();

        let maps = Map::belonging_to(&sub_areas)
            .select(Map::as_select())
            .load(&mut connection)
            .unwrap();

        let mut maps_per_sub_area: HashMap<SubArea, Vec<Map>> = maps
            .grouped_by(&sub_areas)
            .into_iter()
            .zip(sub_areas)
            .map(|(maps, sub_area)| (sub_area, maps))
            .collect();

        maps_per_sub_area.retain(|_, vec| !vec.is_empty());

        maps_per_sub_area
    }

    fn load_initial_ingredients_quantity(&self) -> ItemList {
        use crate::database::schema::{items, user_ingredients, user_items};
        use diesel::prelude::*;

        let mut connection = self.pool.get().unwrap();

        let user_items: Vec<(UserItem, Item)> = user_items::table
            .inner_join(items::table)
            .load(&mut connection)
            .unwrap();

        user_items.into_iter().for_each(|(user_item, item)| {
            self.item_tx.send((item, user_item.quantity as _)).unwrap()
        });

        ItemList::with_items(
            user_ingredients::table
                .inner_join(items::table)
                .load::<(UserIngredient, Item)>(&mut connection)
                .unwrap()
                .into_iter()
                .map(|(user_ingredient, item)| (item, user_ingredient.quantity))
                .collect(),
        )
    }

    fn item_has_recipe(&self, item: &Item) -> bool {
        use crate::database::schema::recipes::dsl::*;
        use diesel::dsl::*;

        let mut connection = self.pool.get().unwrap();

        let count: i64 = recipes
            .select(count(result_item_id))
            .filter(result_item_id.eq(item.id))
            .first(&mut connection)
            .unwrap();

        count > 0
    }

    fn get_calculated_inventory(&self, inventory: &ItemList) -> ItemList {
        let mut calculated_inventory = ItemList::new();
        inventory.iter().for_each(|(ingredient, quantity)| {
            calculated_inventory.add_item(ingredient, quantity);
            if self.item_has_recipe(ingredient) {
                let (base_ingredients, steps) = Self::item_get_full_recipe(
                    self.pool.clone(),
                    ingredient.clone(),
                    quantity.clone(),
                );
                calculated_inventory.add_items(&base_ingredients);
                steps.iter().for_each(|step_list| {
                    calculated_inventory.add_items(step_list);
                });
            }
        });

        calculated_inventory
    }

    fn map_from_ui_and_index(
        &self,
        ctx: Context,
        index: u16,
        tx: Sender<(Image, u16, usize)>,
        zoom: f32,
        zoom_index: usize,
    ) {
        tokio::spawn(async move {
            // Images start at 1
            let path = format!("images/worldmap/{}/{}.jpg", zoom, index + 1);
            let path = Path::new(&path);
            let image = Image::from_path(&ctx, &path);
            tx.send((image, index, zoom_index)).unwrap();
            ctx.request_repaint();
        });
    }

    fn item_from_id(&self, tx: Sender<(Item, Image)>, ctx: Context, item: Item) {
        tokio::spawn(async move {
            let image = Image::item_from_image_id(&ctx, item.image_id);
            tx.send((item, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn load_monster_image(&self, tx: Sender<(Monster, Image)>, ctx: Context, monster: Monster) {
        tokio::spawn(async move {
            let image = Image::monster_from_id(&ctx, monster.id);
            tx.send((monster, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn load_recipe_of_item(&self, tx: Sender<Ingredients>, item: Item, quantity: i16) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            use crate::database::schema::*;
            use diesel::prelude::*;

            let mut connection = pool.get().unwrap();

            let mut result_hash_map = HashMap::new();

            let (base_ingredients, ingredients_steps) =
                Self::item_get_full_recipe(pool, item.clone(), 1);

            base_ingredients.iter().for_each(|(ingredient, quantity)| {
                let result: Vec<(SubArea, MonsterSubArea, Monster, Drop, Item)> = sub_areas::table
                    .inner_join(
                        monsters_sub_areas::table
                            .on(sub_areas::id.eq(monsters_sub_areas::sub_area_id)),
                    )
                    .inner_join(monsters::table.on(monsters_sub_areas::monster_id.eq(monsters::id)))
                    .inner_join(drops::table.on(monsters::id.eq(drops::monster_id)))
                    .inner_join(items::table.on(drops::item_id.eq(items::id)))
                    .filter(items::id.eq(ingredient.id))
                    .load(&mut connection)
                    .unwrap();

                result_hash_map.insert(ingredient.clone(), (*quantity, Default::default()));
                let mut sub_areas_for_monsters: HashMap<Monster, HashSet<SubArea>> = HashMap::new();

                result.into_iter().for_each(|(sub_area, _, monster, _, _)| {
                    sub_areas_for_monsters
                        .entry(monster)
                        .and_modify(|sub_areas| {
                            sub_areas.insert(sub_area.clone());
                        })
                        .or_insert_with(|| {
                            let mut sub_areas = HashSet::new();
                            sub_areas.insert(sub_area);
                            sub_areas
                        });
                });

                result_hash_map
                    .entry((*ingredient).clone())
                    .and_modify(|(_, monsters)| {
                        *monsters = sub_areas_for_monsters;
                    });
            });

            tx.send((item, quantity, result_hash_map, ingredients_steps))
                .unwrap();
        });
    }
}

impl DatabaseLoader {
    fn item_get_full_recipe(
        pool: Pool<ConnectionManager<PgConnection>>,
        item: Item,
        quantity: i16,
    ) -> (ItemList, Vec<ItemList>) {
        // used in loop to know which item we still need the recipe of
        let mut items_to_make = ItemList::new();
        items_to_make.add_item(&item, &quantity);

        // The last materials of the items
        let mut base_ingredients = ItemList::new();

        // All the steps to create the item
        let mut ingredients_steps: Vec<ItemList> = Vec::new();

        // Temporary to set items to make
        let mut next_items_to_make = ItemList::new();

        let mut connection = pool.get().unwrap();

        while !items_to_make.is_empty() {
            ingredients_steps.push(ItemList::new());
            items_to_make.iter().for_each(|(item, quantity)| {
                if item.has_recipe(&mut connection) {
                    ingredients_steps
                        .last_mut()
                        .unwrap()
                        .add_item(item, quantity);

                    let next = &item.get_recipe(quantity, &mut connection);
                    next_items_to_make.add_items(next);
                } else {
                    base_ingredients.add_item(item, quantity);
                }
            });

            items_to_make.clear();
            items_to_make.add_items(&next_items_to_make);
            next_items_to_make.clear();
        }

        // First is wanted item and last is empty
        ingredients_steps.remove(0);
        ingredients_steps.pop();

        // put first the closest to base ingredients and last closest to final item
        ingredients_steps.reverse();

        (base_ingredients, ingredients_steps)
    }
}
