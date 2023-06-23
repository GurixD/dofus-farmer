use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{Context, Vec2, Window};
use egui_dock::{DockArea, Style, Tree};
use tracing::trace_span;

use super::{
    main_window::ItemsRelations,
    tabs::items_tabs::{
        item_tab_viewer::ItemTabsViewer, item_tabs_data::ItemTabsData, monsters_tab::MonstersTab,
        resources_tab::ResourcesTab, search_item_tabs::SearchItemTab, wish_list_tab::WishListTab,
    },
};

pub struct ItemsWindow {
    tree: Tree<ItemTabsData>,
}

impl ItemsWindow {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        let (new_item_tx, new_item_rx) = tokio::sync::broadcast::channel(1024);

        let wish_list_tab = WishListTab::new(new_item_rx);
        let resources_tab = ResourcesTab::new(pool.clone(), new_item_tx.subscribe());
        let search_item_tab = SearchItemTab::new(pool, new_item_tx);
        let monsters_tab = MonstersTab::new();

        let tree = Tree::new(vec![
            ItemTabsData::SearchItem(search_item_tab),
            ItemTabsData::WishList(wish_list_tab),
            ItemTabsData::Resources(resources_tab),
            ItemTabsData::Monsters(monsters_tab),
        ]);

        Self { tree }
    }

    pub fn show(&mut self, ctx: &Context, items: &ItemsRelations) {
        Window::new("Items")
            .default_size(Vec2::new(1000f32, 1000f32))
            .vscroll(true)
            .show(ctx, |ui| {
                let span = trace_span!("show items window inner");
                let _guard = span.enter();

                let mut tab_viewer = ItemTabsViewer::new(items);

                DockArea::new(&mut self.tree)
                    .show_close_buttons(false)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut tab_viewer);
            });
    }
}
