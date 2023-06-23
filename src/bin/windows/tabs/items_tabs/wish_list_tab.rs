use std::{collections::BTreeMap, sync::mpsc};

use egui::{Context, ImageButton, Layout, Ui, Vec2};
use futures::executor::block_on;
use tokio::sync::broadcast;

use crate::{
    database::models::item::Item,
    windows::main_window::{AsyncStatus, Image, ItemsRelations, MainWindow},
};

pub struct WishListTab {
    items: BTreeMap<Item, (usize, Option<AsyncStatus<Image>>)>,
    image_tx: mpsc::Sender<(Item, Image)>,
    image_rx: mpsc::Receiver<(Item, Image)>,
    new_item_rx: broadcast::Receiver<(Item, usize)>,
}

impl WishListTab {
    pub fn new(new_item_rx: broadcast::Receiver<(Item, usize)>) -> Self {
        let items = Default::default();
        let (image_tx, image_rx) = mpsc::channel();

        Self {
            items,
            image_tx,
            image_rx,
            new_item_rx,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, items: &ItemsRelations) {
        for _ in 0..self.new_item_rx.len() {
            let (item, quantity) = block_on(self.new_item_rx.recv()).unwrap();
            self.items
                .entry(item)
                .and_modify(|item| {
                    item.0 += quantity;
                })
                .or_insert_with(|| (quantity, None));
        }

        self.image_rx.try_iter().for_each(|(item, image)| {
            self.items.entry(item).and_modify(|item| {
                item.1 = Some(AsyncStatus::Ready(image));
            });
        });

        ui.horizontal_wrapped(|ui| {
            self.items.iter_mut().for_each(|(item, (quantity, image))| {
                if let Some(image) = image {
                    if let AsyncStatus::Ready(image) = image {
                        ui.allocate_ui_with_layout(
                            Vec2::new(100f32, 150f32),
                            Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.group(|ui| {
                                    let button = ImageButton::new(
                                        image.handle.id(),
                                        MainWindow::ITEM_IMAGE_SIZE,
                                    );
                                    ui.add(button);
                                    ui.label(quantity.to_string());
                                });
                            },
                        );
                    }
                } else {
                    *image = Some(AsyncStatus::Loading);
                    Self::load_item_image(self.image_tx.clone(), item, ui.ctx());
                }
            });
        });
    }

    fn load_item_image(tx: mpsc::Sender<(Item, Image)>, item: &Item, ctx: &Context) {
        let ctx = ctx.clone();
        let image_id = item.image_id;
        let item = item.clone();

        tokio::spawn(async move {
            let image = Image::item_from_image_id(&ctx, image_id as _);
            tx.send((item, image)).unwrap();
            ctx.request_repaint();
        });
    }
}
