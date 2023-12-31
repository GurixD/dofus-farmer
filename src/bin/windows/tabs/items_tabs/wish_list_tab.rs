use std::{collections::HashMap, rc::Rc, sync::mpsc::Sender};

use egui::{ImageButton, Layout, PointerButton, Ui, Vec2};

use crate::{
    database::models::{item::Item, sub_area::SubArea},
    windows::main_window::{AsyncStatus, Image, ItemsRelations, MainWindow},
};

pub struct WishListTab {
    remove_item_tx: Sender<(Item, usize, bool)>,
}

impl WishListTab {
    pub fn new(remove_item_tx: Sender<(Item, usize, bool)>) -> Self {
        Self { remove_item_tx }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        items_images: &HashMap<Rc<Item>, AsyncStatus<Image>>,
        current_sub_area: &Option<SubArea>,
    ) {
        ui.horizontal_wrapped(|ui| {
            items.iter().for_each(|(item, (quantity, ingredients))| {
                if let Some(AsyncStatus::Ready(image)) = items_images.get(item) {
                    let show_this = current_sub_area
                        .clone()
                        .map(|sub_area| {
                            if let AsyncStatus::Ready(ingredients) = ingredients {
                                return ingredients.iter().any(|(_, (_, monsters))| {
                                    monsters
                                        .iter()
                                        .any(|(_, sub_areas)| sub_areas.contains(&sub_area))
                                });
                            }

                            false
                        })
                        .unwrap_or(true);

                    if show_this {
                        ui.allocate_ui_with_layout(
                            Vec2::new(100f32, 150f32),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.group(|ui| {
                                    let button = ImageButton::new(
                                        image.handle.id(),
                                        MainWindow::ITEM_IMAGE_SIZE,
                                    );
                                    let response = ui.add(button).on_hover_text(&item.name);

                                    if response.clicked_by(PointerButton::Primary) {
                                        self.remove_item_tx
                                            .send((item.as_ref().clone(), 1, true))
                                            .unwrap();
                                    } else if response.clicked_by(PointerButton::Secondary) {
                                        self.remove_item_tx
                                            .send((item.as_ref().clone(), 1, false))
                                            .unwrap();
                                    }

                                    ui.label(quantity.to_string());
                                });
                            },
                        );
                    }
                }
            });
        });
    }
}
