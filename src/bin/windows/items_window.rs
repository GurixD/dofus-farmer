use std::{collections::HashMap, rc::Rc, sync::mpsc::Sender};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{Context, Vec2, Window};
use egui_dock::{DockArea, NodeIndex, Style, Tree};
use tracing::trace_span;

use crate::database::models::{item::Item, monster::Monster, sub_area::SubArea};

use super::{
    main_window::{AsyncStatus, Image, ItemsRelations},
    tabs::items_tabs::{
        item_tab_viewer::ItemTabsViewer, item_tabs_data::ItemTabsData, monsters_tab::MonstersTab,
        resources_tab::ResourcesTab, search_item_tabs::SearchItemTab, wish_list_tab::WishListTab,
    },
};

pub struct ItemsWindow {
    tree: Tree<ItemTabsData>,
}

impl ItemsWindow {
    pub fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        item_clicked_tx: Sender<(Item, usize)>,
    ) -> Self {
        let wish_list_tab = WishListTab::new();
        let resources_tab = ResourcesTab::new();
        let search_item_tab = SearchItemTab::new(pool, item_clicked_tx);
        let monsters_tab = MonstersTab::new();

        let mut tree = Tree::new(vec![ItemTabsData::SearchItem(search_item_tab)]);
        let tabs = tree.split_right(
            NodeIndex::root(),
            0.27,
            vec![ItemTabsData::Resources(resources_tab)],
        );

        tree.split_right(tabs[1], 0.5, vec![ItemTabsData::Monsters(monsters_tab)]);
        tree.split_below(tabs[0], 0.5, vec![ItemTabsData::WishList(wish_list_tab)]);

        Self { tree }
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        items: &ItemsRelations,
        items_images: &HashMap<Rc<Item>, AsyncStatus<Image>>,
        monsters_images: &HashMap<Rc<Monster>, AsyncStatus<Image>>,
        current_sub_area: &Option<SubArea>,
    ) {
        Window::new("Items")
            .default_size(Vec2::new(1500f32, 1000f32))
            .vscroll(true)
            .show(ctx, |ui| {
                let span = trace_span!("show items window inner");
                let _guard = span.enter();

                let mut tab_viewer =
                    ItemTabsViewer::new(items, items_images, monsters_images, current_sub_area);

                DockArea::new(&mut self.tree)
                    .show_close_buttons(false)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut tab_viewer);
            });
    }
}
