use std::{
    cmp,
    collections::{BTreeMap, HashMap},
    sync::mpsc::Sender,
};

use crate::{
    database::models::{
        item::{Item, ItemList},
        sub_area::SubArea,
    },
    windows::{
        image::Image,
        items_window::ItemsWindow,
        main_window::{AsyncStatus, ItemsRelations},
    },
};

use egui::{ImageButton, Key, Layout, PointerButton, Ui, Vec2};
use egui_modal::Modal;

pub struct ResourcesTab {
    new_ingredient_tx: Sender<(Item, isize)>,
    modal_clicked_item: Option<Item>,
    modal_quantity: String,
    current_page: usize,
    max_page: usize,
}

impl ResourcesTab {
    pub fn new(new_ingredient_tx: Sender<(Item, isize)>) -> Self {
        let modal_clicked_item = Default::default();
        let modal_quantity = Default::default();
        let current_page = 1;
        let max_page = 10; // todo

        Self {
            new_ingredient_tx,
            modal_clicked_item,
            modal_quantity,
            current_page,
            max_page,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        items_images: &HashMap<Item, AsyncStatus<Image>>,
        _ingredients_quantity: &ItemList,
        calculated_inventory: &ItemList,
        current_sub_area: &Option<SubArea>,
    ) {
        if ui.input(|reader| reader.key_pressed(Key::ArrowLeft)) {
            self.current_page = self.current_page.checked_sub(1).unwrap_or(1);
        }
        if ui.input(|reader| reader.key_pressed(Key::ArrowRight)) {
            self.current_page += 1;
        }

        self.current_page = cmp::max(cmp::min(self.current_page, self.max_page), 1);

        // Same as ingredient_quantity but with quantity needed
        let mut showed_ingredients = BTreeMap::new();

        self.max_page = 1;
        items.iter().for_each(|(_, (quantity, ingredients))| {
            if let AsyncStatus::Ready((ingredients, steps)) = ingredients {
                self.max_page = cmp::max(self.max_page, steps.len() + 1);

                if self.current_page == 1 {
                    ingredients
                        .iter()
                        .for_each(|(ingredient, (needed, monsters))| {
                            let show_this = if let Some(sub_area) = current_sub_area {
                                monsters
                                    .iter()
                                    .any(|(_, sub_areas)| sub_areas.contains(sub_area))
                            } else {
                                true
                            };
                            if show_this {
                                showed_ingredients
                                    .entry(ingredient)
                                    .and_modify(|(needed_total, _)| {
                                        *needed_total += needed * quantity;
                                    })
                                    .or_insert_with(|| {
                                        (
                                            *needed * quantity,
                                            calculated_inventory.get(ingredient).unwrap_or(&0),
                                        )
                                    });
                            }
                        });
                } else if let Some(steps) = steps.get(self.current_page - 2) {
                    steps.iter().for_each(|(item_step, needed)| {
                        showed_ingredients
                            .entry(item_step)
                            .and_modify(|(current_needed, _)| {
                                *current_needed += needed * quantity;
                            })
                            .or_insert_with(|| {
                                (
                                    *needed * quantity,
                                    calculated_inventory.get(item_step).unwrap_or(&0),
                                )
                            });
                    });
                }
            }
        });

        let quantity_modal = Modal::new(ui.ctx(), "resources modal");

        quantity_modal.show(|ui| {
            ui.text_edit_singleline(&mut self.modal_quantity);
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    self.modal_clicked_item = None;
                    quantity_modal.close();
                } else if ui.button("Add").clicked() {
                    if let Ok(quantity) = self.modal_quantity.parse::<isize>() {
                        let item = self.modal_clicked_item.take();
                        self.new_ingredient_tx
                            .send((item.unwrap(), quantity))
                            .unwrap();
                        quantity_modal.close();
                    }
                }
            });
        });

        ui.vertical_centered(|ui| ui.label(format!("{}/{}", self.current_page, self.max_page)));
        ui.horizontal_wrapped(|ui| {
            showed_ingredients
                .iter()
                .for_each(|(&item, (needed, &in_inventory))| {
                    if let Some(AsyncStatus::Ready(image)) = items_images.get(item) {
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
                                        self.new_ingredient_tx.send((item.clone(), 1)).unwrap();
                                    } else if response.clicked_by(PointerButton::Secondary) {
                                        self.modal_clicked_item = Some(item.clone());
                                        self.modal_quantity = Default::default();
                                        quantity_modal.open();
                                    } else if response.clicked_by(PointerButton::Middle) {
                                        self.new_ingredient_tx
                                            .send((
                                                item.clone(),
                                                *needed as isize - in_inventory as isize,
                                            ))
                                            .unwrap();
                                    }

                                    ui.label(format!("{in_inventory}/{needed}"));
                                });
                            },
                        );
                    }
                });
        });
    }
}
