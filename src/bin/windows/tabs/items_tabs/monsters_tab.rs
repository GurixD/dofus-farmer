use std::{
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use egui::{ImageButton, Ui};

use crate::{
    database::models::{item::Item, monster::Monster, sub_area::SubArea},
    windows::{
        items_window::ItemsWindow,
        main_window::{AsyncStatus, Image, ItemsRelations},
    },
};

pub struct MonstersTab;

impl MonstersTab {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        items: &ItemsRelations,
        monsters_images: &HashMap<Rc<Monster>, AsyncStatus<Image>>,
        ingredient_quantity: &HashMap<Item, usize>,
        current_sub_area: &Option<SubArea>,
    ) {
        let mut still_needed_ingredients_total = HashMap::new();

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
                            still_needed_ingredients_total
                                .entry(ingredient)
                                .and_modify(|(needed_total, _)| {
                                    *needed_total += needed * quantity;
                                })
                                .or_insert_with(|| {
                                    (
                                        *needed * quantity,
                                        *ingredient_quantity.get(ingredient).unwrap_or(&0),
                                    )
                                });
                        }
                    });
            }
        });

        still_needed_ingredients_total.retain(|_, (needed, in_inventory)| in_inventory < needed);

        let mut all_monsters = BTreeSet::new();

        items.iter().for_each(|(_, (_, ingredients))| {
            if let AsyncStatus::Ready(ingredients) = ingredients {
                ingredients.iter().for_each(|(ingredient, (_, monsters))| {
                    if still_needed_ingredients_total.contains_key(ingredient) {
                        let monsters: BTreeSet<_> = monsters
                            .iter()
                            .filter_map(|(monster, sub_areas)| {
                                if let Some(sub_area) = current_sub_area {
                                    if !sub_areas.contains(sub_area) {
                                        return None;
                                    }
                                }

                                Some(monster.as_ref())
                            })
                            .collect();

                        all_monsters.extend(monsters);
                    }
                });
            }
        });

        ui.horizontal_wrapped(|ui| {
            all_monsters.iter().for_each(|&monster| {
                if let Some(AsyncStatus::Ready(image)) = monsters_images.get(monster) {
                    let button = ImageButton::new(egui::Image::from_texture(
                        ItemsWindow::get_sized_texture(&image),
                    ));
                    let response = ui.add(button);
                    response.on_hover_text(&monster.name);
                }
            });
        });
    }
}
