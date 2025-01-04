use std::{collections::HashMap, path::Path, sync::mpsc::Sender};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::Context;
use lombok::AllArgsConstructor;

use crate::{
    database::models::{
        item::{Item, ItemList}, map::Map, monster::Monster, sub_area::SubArea, user_ingredient::UserIngredient, user_item::UserItem
    },
    gui::image::Image,
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
        let mut connection = self.pool.get().unwrap();
        let mut calculated_inventory = ItemList::new();
        inventory.iter().for_each(|(ingredient, quantity)| {
            calculated_inventory.add_item(ingredient, quantity);
            if self.item_has_recipe(ingredient) {
                let (base_ingredients, steps) =
                    ingredient.get_full_recipe(quantity, &mut connection);
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
}
