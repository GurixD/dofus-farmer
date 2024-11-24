use std::{collections::HashMap, sync::mpsc::Sender};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{load::SizedTexture, Context, Vec2, Window};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use tracing::trace_span;

use crate::database::models::{
    item::{Item, ItemList},
    monster::Monster,
    sub_area::SubArea,
};

use super::{
    image::Image,
    main_window::{AsyncStatus, ItemsRelations, MainWindow},
    tabs::items_tabs::{
        item_tab_viewer::ItemTabsViewer, item_tabs_data::ItemTabsData, monsters_tab::MonstersTab,
        resources_tab::ResourcesTab, search_item_tabs::SearchItemTab, wish_list_tab::WishListTab,
    },
};

pub struct ItemsWindow {
    dock_state: DockState<ItemTabsData>,
}

impl ItemsWindow {
    pub fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        item_clicked_tx: Sender<(Item, i16)>,
        new_ingredient_tx: Sender<(Item, isize)>,
        remove_item_tx: Sender<(Item, i16, bool)>,
    ) -> Self {
        let wish_list_tab = WishListTab::new(remove_item_tx);
        let resources_tab = ResourcesTab::new(new_ingredient_tx);
        let search_item_tab = SearchItemTab::new(pool, item_clicked_tx);
        let monsters_tab = MonstersTab::new();

        let mut dock_state = DockState::new(vec![ItemTabsData::SearchItem(search_item_tab)]);
        let surface = dock_state.main_surface_mut();
        let tabs = surface.split_right(
            NodeIndex::root(),
            0.27,
            vec![ItemTabsData::Resources(resources_tab)],
        );

        surface.split_right(tabs[1], 0.5, vec![ItemTabsData::Monsters(monsters_tab)]);
        surface.split_below(tabs[0], 0.5, vec![ItemTabsData::WishList(wish_list_tab)]);

        Self { dock_state }
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        items: &ItemsRelations,
        ingredients_quantity: &ItemList,
        calculated_inventory: &ItemList,
        items_images: &HashMap<Item, AsyncStatus<Image>>,
        monsters_images: &HashMap<Monster, AsyncStatus<Image>>,
        current_sub_area: &Option<SubArea>,
    ) {
        Window::new("Items")
            .default_size(Vec2::new(1500f32, 1000f32))
            .vscroll(true)
            .show(ctx, |ui| {
                let span = trace_span!("show items window inner");
                let _guard = span.enter();

                let mut tab_viewer = ItemTabsViewer::new(
                    items,
                    ingredients_quantity,
                    calculated_inventory,
                    items_images,
                    monsters_images,
                    current_sub_area,
                );

                DockArea::new(&mut self.dock_state)
                    .show_close_buttons(false)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut tab_viewer);
            });
    }

    pub(crate) fn get_sized_texture(image: &Image) -> SizedTexture {
        SizedTexture {
            id: image.handle.id(),
            size: MainWindow::ITEM_IMAGE_SIZE,
        }
    }
}
