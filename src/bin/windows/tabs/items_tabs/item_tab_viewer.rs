use egui_dock::TabViewer;

use super::item_tabs_data::ItemTabsData;

pub struct ItemTabsViewer;

impl TabViewer for ItemTabsViewer {
    type Tab = ItemTabsData;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // enum tab then inner_enum.show
        match tab {
            ItemTabsData::SearchItem(tab) => tab.show(ui),
            ItemTabsData::WishList(tab) => tab.show(ui),
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            ItemTabsData::SearchItem(_) => "Search".into(),
            ItemTabsData::WishList(_) => "Wish list".into(),
        }
    }
}
