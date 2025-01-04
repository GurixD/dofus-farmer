use std::{collections::HashMap, sync::mpsc::Sender};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::Context;

use crate::{
    database::models::{
        item::{Item, ItemList},
        map::Map,
        monster::Monster,
        sub_area::SubArea,
    },
    gui::image::Image,
};

pub trait DataLoader: Send {
    fn new(pool: Pool<ConnectionManager<PgConnection>>, item_tx: Sender<(Item, i16)>) -> Self;
    fn load_all_sub_areas(&self) -> HashMap<SubArea, Vec<Map>>;
    fn load_initial_ingredients_quantity(&self) -> ItemList;
    fn item_has_recipe(&self, item: &Item) -> bool;
    fn get_calculated_inventory(&self, inventory: &ItemList) -> ItemList;
    fn map_from_ui_and_index(
        &self,
        ctx: Context,
        index: u16,
        tx: Sender<(Image, u16, usize)>,
        zoom: f32,
        zoom_index: usize,
    );
    fn item_from_id(&self, tx: Sender<(Item, Image)>, ctx: Context, item: Item);
    fn load_monster_image(&self, tx: Sender<(Monster, Image)>, ctx: Context, monster: Monster);
}
