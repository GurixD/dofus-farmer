use egui::Ui;

use crate::windows::main_window::ItemsRelations;

pub struct MonstersTab;

impl MonstersTab {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &Ui, items: &ItemsRelations) {}
}
