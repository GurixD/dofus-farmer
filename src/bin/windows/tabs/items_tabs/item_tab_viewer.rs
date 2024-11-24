use std::collections::HashMap;

use egui::Ui;
use egui_dock::TabViewer;

use crate::{
    database::models::{
        item::{Item, ItemList},
        monster::Monster,
        sub_area::SubArea,
    },
    windows::{
        image::Image,
        main_window::{AsyncStatus, ItemsRelations},
    },
};

use super::item_tabs_data::ItemTabsData;

pub struct ItemTabsViewer<'a> {
    items: &'a ItemsRelations,
    ingredients_quantity: &'a ItemList,
    calculated_inventory: &'a ItemList,
    items_images: &'a HashMap<Item, AsyncStatus<Image>>,
    monsters_images: &'a HashMap<Monster, AsyncStatus<Image>>,
    current_sub_area: &'a Option<SubArea>,
}

impl<'a> ItemTabsViewer<'a> {
    pub fn new(
        items: &'a ItemsRelations,
        ingredients_quantity: &'a ItemList,
        calculated_inventory: &'a ItemList,
        items_images: &'a HashMap<Item, AsyncStatus<Image>>,
        monsters_images: &'a HashMap<Monster, AsyncStatus<Image>>,
        current_sub_area: &'a Option<SubArea>,
    ) -> Self {
        Self {
            items,
            ingredients_quantity,
            calculated_inventory,
            items_images,
            monsters_images,
            current_sub_area,
        }
    }
}

impl<'a> TabViewer for ItemTabsViewer<'a> {
    type Tab = ItemTabsData;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            ItemTabsData::SearchItem(tab) => tab.show(ui),
            ItemTabsData::WishList(tab) => {
                tab.show(ui, self.items, self.items_images, self.current_sub_area)
            }
            ItemTabsData::Resources(tab) => tab.show(
                ui,
                self.items,
                self.items_images,
                self.ingredients_quantity,
                self.calculated_inventory,
                self.current_sub_area,
            ),
            ItemTabsData::Monsters(tab) => tab.show(
                ui,
                self.items,
                self.monsters_images,
                self.ingredients_quantity,
                self.current_sub_area,
            ),
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            ItemTabsData::SearchItem(_) => "Search".into(),
            ItemTabsData::WishList(_) => "Wish list".into(),
            ItemTabsData::Resources(_) => "Resources".into(),
            ItemTabsData::Monsters(_) => "Monsters".into(),
        }
    }
}
