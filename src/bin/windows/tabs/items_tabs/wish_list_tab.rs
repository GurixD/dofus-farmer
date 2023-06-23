use std::{collections::HashMap, rc::Rc};

use egui::{ImageButton, Layout, Ui, Vec2};

use crate::{
    database::models::item::Item,
    windows::main_window::{AsyncStatus, Image, ItemsRelations, MainWindow},
};

pub struct WishListTab {}

impl WishListTab {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        items_images: &HashMap<Rc<Item>, AsyncStatus<Image>>,
    ) {
        ui.horizontal_wrapped(|ui| {
            items.iter().for_each(|(item, (quantity, _ingredient))| {
                if let Some(AsyncStatus::Ready(image)) = items_images.get(item) {
                    ui.allocate_ui_with_layout(
                        Vec2::new(100f32, 150f32),
                        Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.group(|ui| {
                                let button = ImageButton::new(
                                    image.handle.id(),
                                    MainWindow::ITEM_IMAGE_SIZE,
                                );
                                let response = ui.add(button);
                                response.on_hover_text(&item.name);
                                ui.label(quantity.to_string());
                            });
                        },
                    );
                }
            });
        });
    }
}
