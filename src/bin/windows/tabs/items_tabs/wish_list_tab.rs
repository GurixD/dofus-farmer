use std::{collections::HashMap, rc::Rc, sync::mpsc::Sender};

use egui::{ImageButton, Layout, PointerButton, Ui, Vec2};
use egui_modal::Modal;

use crate::{
    database::models::{item::Item, sub_area::SubArea},
    windows::{
        items_window::ItemsWindow,
        main_window::{AsyncStatus, Image, ItemsRelations},
    },
};

pub struct WishListTab {
    remove_item_tx: Sender<(Item, usize, bool)>,
    modal_quantity: String,
    modal_craft: bool,
    modal_clicked_item: Option<Item>,
}

impl WishListTab {
    pub fn new(remove_item_tx: Sender<(Item, usize, bool)>) -> Self {
        let modal_quantity = Default::default();
        let modal_craft = false;
        let modal_clicked_item = Default::default();

        Self {
            remove_item_tx,
            modal_quantity,
            modal_craft,
            modal_clicked_item,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        items_images: &HashMap<Rc<Item>, AsyncStatus<Image>>,
        current_sub_area: &Option<SubArea>,
    ) {
        let quantity_modal = Modal::new(ui.ctx(), "wish list modal");

        quantity_modal.show(|ui| {
            ui.text_edit_singleline(&mut self.modal_quantity);
            ui.checkbox(&mut self.modal_craft, "Craft");
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    self.modal_clicked_item = None;
                    quantity_modal.close();
                } else if ui.button("Add").clicked() {
                    if let Ok(quantity) = self.modal_quantity.parse::<usize>() {
                        let item = self.modal_clicked_item.take();
                        self.remove_item_tx
                            .send((item.unwrap(), quantity, self.modal_craft))
                            .unwrap();
                        quantity_modal.close();
                    }
                }
            });
        });

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
                                    let button = ImageButton::new(egui::Image::from_texture(
                                        ItemsWindow::get_sized_texture(image),
                                    ));
                                    let response = ui.add(button).on_hover_text(&item.name);

                                    if response.clicked_by(PointerButton::Primary) {
                                        self.remove_item_tx
                                            .send((item.as_ref().clone(), 1, true))
                                            .unwrap();
                                    } else if response.clicked_by(PointerButton::Secondary) {
                                        self.remove_item_tx
                                            .send((item.as_ref().clone(), 1, false))
                                            .unwrap();
                                    } else if response.clicked_by(PointerButton::Middle) {
                                        self.modal_clicked_item = Some(item.as_ref().clone());
                                        self.modal_quantity = Default::default();
                                        quantity_modal.open();
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
