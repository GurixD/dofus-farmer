use std::{collections::HashMap, rc::Rc};

use egui::Ui;
use egui_dock::TabViewer;

use crate::{
    database::models::{item::Item, monster::Monster},
    windows::main_window::{AsyncStatus, Image, ItemsRelations},
};

use super::item_tabs_data::ItemTabsData;

pub struct ItemTabsViewer<'a> {
    items: &'a ItemsRelations,
    items_images: &'a HashMap<Rc<Item>, AsyncStatus<Image>>,
    monsters_images: &'a HashMap<Rc<Monster>, AsyncStatus<Image>>,
}

impl<'a> ItemTabsViewer<'a> {
    pub fn new(
        items: &'a ItemsRelations,
        items_images: &'a HashMap<Rc<Item>, AsyncStatus<Image>>,
        monsters_images: &'a HashMap<Rc<Monster>, AsyncStatus<Image>>,
    ) -> Self {
        Self {
            items,
            items_images,
            monsters_images,
        }
    }
}

impl<'a> TabViewer for ItemTabsViewer<'a> {
    type Tab = ItemTabsData;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            ItemTabsData::SearchItem(tab) => tab.show(ui),
            ItemTabsData::WishList(tab) => tab.show(ui, self.items, self.items_images),
            ItemTabsData::Resources(tab) => tab.show(ui, self.items, self.items_images),
            ItemTabsData::Monsters(tab) => tab.show(ui, self.items, self.monsters_images),
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
