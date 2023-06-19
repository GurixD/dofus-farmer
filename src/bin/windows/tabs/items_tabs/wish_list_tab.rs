use egui::Ui;

use crate::database::models::item::Item;

pub struct WishListTab {
    items: Vec<(Item, usize)>,
}

impl WishListTab {
    pub fn new() -> Self {
        let items = Default::default();

        Self { items }
    }

    pub fn show(&mut self, ui: &mut Ui) {}
}
