use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::mpsc::Sender,
};

use crate::{
    database::models::{item::Item, sub_area::SubArea},
    windows::{
        items_window::ItemsWindow,
        main_window::{AsyncStatus, Image, ItemsRelations},
    },
};

use egui::{ImageButton, Layout, PointerButton, Ui, Vec2};

pub struct ResourcesTab {
    new_ingredient_tx: Sender<(Item, isize)>,
}

impl ResourcesTab {
    pub fn new(new_ingredient_tx: Sender<(Item, isize)>) -> Self {
        Self { new_ingredient_tx }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        items_images: &HashMap<Rc<Item>, AsyncStatus<Image>>,
        ingredient_quantity: &HashMap<Item, usize>,
        current_sub_area: &Option<SubArea>,
    ) {
        let mut ingredients_total = BTreeMap::new();

        items.iter().for_each(|(_, (quantity, ingredients))| {
            if let AsyncStatus::Ready(ingredients) = ingredients {
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
                            ingredients_total
                                .entry(ingredient)
                                .and_modify(|(needed_total, _)| {
                                    *needed_total += needed * quantity;
                                })
                                .or_insert_with(|| {
                                    (
                                        *needed * quantity,
                                        ingredient_quantity.get(ingredient).unwrap_or(&0),
                                    )
                                });
                        }
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
                                    let button = ImageButton::new(egui::Image::from_texture(
                                        ItemsWindow::get_sized_texture(image),
                                    ));

                                    let response = ui.add(button).on_hover_text(&item.name);
                                    if response.clicked_by(PointerButton::Primary) {
                                        self.new_ingredient_tx
                                            .send((item.as_ref().clone(), 1))
                                            .unwrap();
                                    } else if response.clicked_by(PointerButton::Secondary) {
                                        self.new_ingredient_tx
                                            .send((item.as_ref().clone(), -1))
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
