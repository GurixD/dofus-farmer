use std::{collections::HashMap, sync::mpsc::Sender};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use lombok::AllArgsConstructor;

use crate::database::models::{
    item::{Item, ItemList},
    map::Map,
    sub_area::SubArea,
    user_ingredient::UserIngredient,
    user_item::UserItem,
};

use super::data_loader::DataLoader;

#[derive(AllArgsConstructor)]
pub struct DatabaseLoader {
    pool: Pool<ConnectionManager<PgConnection>>,
    item_tx: Sender<(Item, i16)>,
}

impl DataLoader for DatabaseLoader {
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
}
