use egui::Ui;
use egui_dock::TabViewer;

use crate::windows::main_window::ItemsRelations;

use super::item_tabs_data::ItemTabsData;

pub struct ItemTabsViewer<'a> {
    pub items: &'a ItemsRelations,
}

impl<'a> ItemTabsViewer<'a> {
    pub fn new(items: &'a ItemsRelations) -> Self {
        Self { items }
    }
}

impl<'a> TabViewer for ItemTabsViewer<'a> {
    type Tab = ItemTabsData;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            ItemTabsData::SearchItem(tab) => tab.show(ui),
            ItemTabsData::WishList(tab) => tab.show(ui, &self.items),
            ItemTabsData::Resources(tab) => tab.show(ui, &self.items),
            ItemTabsData::Monsters(tab) => tab.show(ui, &self.items),
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
