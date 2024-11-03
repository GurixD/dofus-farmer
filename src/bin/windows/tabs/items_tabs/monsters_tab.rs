use std::{
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use egui::{ImageButton, Ui};

use crate::{
    database::models::{monster::Monster, sub_area::SubArea},
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
        current_sub_area: &Option<SubArea>,
    ) {
        let mut all_monsters = BTreeSet::new();

        items.iter().for_each(|(_, (_, ingredients))| {
            if let AsyncStatus::Ready(ingredients) = ingredients {
                ingredients.iter().for_each(|(_, (_, monsters))| {
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
