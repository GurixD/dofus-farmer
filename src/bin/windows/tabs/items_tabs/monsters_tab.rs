use std::{
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use egui::{ImageButton, Ui};

use crate::{
    database::models::monster::Monster,
    windows::main_window::{AsyncStatus, Image, ItemsRelations, MainWindow},
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
    ) {
        let mut all_monsters = BTreeSet::new();

        items.iter().for_each(|(_, (_, ingredients))| {
            if let AsyncStatus::Ready(ingredients) = ingredients {
                ingredients.iter().for_each(|(_, (_, _, monsters))| {
                    let monsters: BTreeSet<_> = monsters
                        .iter()
                        .map(|(monster, _)| monster.as_ref())
                        .collect();

                    all_monsters.extend(monsters);
                });
            }
        });

        ui.horizontal_wrapped(|ui| {
            all_monsters.iter().for_each(|&monster| {
                if let Some(AsyncStatus::Ready(image)) = monsters_images.get(monster) {
                    let button = ImageButton::new(image.handle.id(), MainWindow::ITEM_IMAGE_SIZE);
                    let response = ui.add(button);
                    response.on_hover_text(&monster.name);
                }
            });
        });
    }
}
