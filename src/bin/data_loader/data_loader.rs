use std::collections::HashMap;

use crate::database::models::{map::Map, sub_area::SubArea};

pub trait DataLoader {
    fn load_all_sub_areas(&self) -> HashMap<SubArea, Vec<Map>>;
}
