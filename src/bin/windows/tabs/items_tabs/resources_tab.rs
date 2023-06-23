use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use crate::{
    database::models::item::Item,
    windows::main_window::{AsyncStatus, Image, ItemsRelations, MainWindow},
};

use egui::{ImageButton, Layout, Ui, Vec2};

pub struct ResourcesTab {}

impl ResourcesTab {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        items_images: &HashMap<Rc<Item>, AsyncStatus<Image>>,
    ) {
        let mut ingredients_total = BTreeMap::new();

        items.iter().for_each(|(_, (quantity, ingredients))| {
            if let AsyncStatus::Ready(ingredients) = ingredients {
                ingredients
                    .iter()
                    .for_each(|(ingredient, (needed, in_inventory, _))| {
                        ingredients_total
                            .entry(ingredient)
                            .and_modify(|(needed_total, in_inventory_total)| {
                                *needed_total += needed * quantity;
                                *in_inventory_total += in_inventory;
                            })
                            .or_insert_with(|| (*needed * quantity, *in_inventory));
                    });
            }
        });

        ui.horizontal_wrapped(|ui| {
            ingredients_total
                .iter()
                .for_each(|(&item, (needed, in_inventory))| {
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
                                    ui.label(format!("{in_inventory}/{needed}"));
                                });
                            },
                        );
                    }
                });
        });
    }
}
