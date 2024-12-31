use std::collections::HashMap;

use crate::database::models::{item::ItemList, map::Map, sub_area::SubArea};

pub trait DataLoader {
    fn load_all_sub_areas(&self) -> HashMap<SubArea, Vec<Map>>;
    fn load_initial_ingredients_quantity(&self) -> ItemList;
}
