use std::{collections::HashMap, sync::mpsc::Sender};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

use crate::database::models::{
    item::{Item, ItemList},
    map::Map,
    sub_area::SubArea,
};

pub trait DataLoader {
    fn new(pool: Pool<ConnectionManager<PgConnection>>, item_tx: Sender<(Item, i16)>) -> Self;
    fn load_all_sub_areas(&self) -> HashMap<SubArea, Vec<Map>>;
    fn load_initial_ingredients_quantity(&self) -> ItemList;
    fn item_has_recipe(&self, item: &Item) -> bool;
    fn get_calculated_inventory(&self, inventory: &ItemList) -> ItemList;
}
