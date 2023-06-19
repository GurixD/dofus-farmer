use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{Context, Window};
use egui_dock::{DockArea, Style, Tree};
use tracing::trace_span;

use super::tabs::items_tabs::{
    item_tab_viewer::ItemTabsViewer, item_tabs_data::ItemTabsData, search_item_tabs::SearchItemTab,
    wish_list_tab::WishListTab,
};

pub struct ItemsWindow {
    tree: Tree<ItemTabsData>,
}

impl ItemsWindow {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        let search_item_tab = SearchItemTab::new(pool.clone());
        let wish_list_tab = WishListTab::new();

        let tree = Tree::new(vec![
            ItemTabsData::SearchItem(search_item_tab),
            ItemTabsData::WishList(wish_list_tab),
        ]);

        Self { tree }
    }

    pub fn show(&mut self, ctx: &Context) {
        Window::new("Items").vscroll(true).show(ctx, |ui| {
            let span = trace_span!("show items window inner");
            let _guard = span.enter();

            DockArea::new(&mut self.tree)
                .show_close_buttons(false)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut ItemTabsViewer {});
        });
    }
}
