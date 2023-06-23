use std::{
    collections::{HashMap, HashSet},
    sync::{
        self,
        mpsc::{Receiver, Sender},
    },
};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{Context, ImageButton, PointerButton, TextEdit, Ui};
use egui_modal::Modal;
use itertools::Itertools;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::trace_span;

use crate::{
    database::models::{
        drop::Drop, item::Item, monster::Monster, monster_sub_area::MonsterSubArea, recipe::Recipe,
        sub_area::SubArea,
    },
    windows::main_window::{AsyncStatus, Image, MainWindow},
};

pub struct SearchItemTab {
    search_bar_text: String,
    items: Vec<(Item, AsyncStatus<Image>)>,
    pool: Pool<ConnectionManager<PgConnection>>,
    items_tx: Sender<(String, Vec<Item>)>,
    items_rx: Receiver<(String, Vec<Item>)>,
    item_image_tx: Sender<(usize, Image)>,
    item_image_rx: Receiver<(usize, Image)>,
    new_item_tx: broadcast::Sender<(Item, usize)>,
    current_search_thread: Option<(String, JoinHandle<()>)>,
}

impl SearchItemTab {
    const CATEGORY_SEARCHED: [i16; 3] = [0, 1, 2];

    pub fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        new_item_tx: broadcast::Sender<(Item, usize)>,
    ) -> Self {
        let search_bar_text = Default::default();
        let items = Default::default();
        let (items_tx, items_rx) = sync::mpsc::channel();
        let (item_image_tx, item_image_rx) = sync::mpsc::channel();
        let current_search_thread = None;

        Self {
            search_bar_text,
            items,
            pool,
            items_tx,
            items_rx,
            item_image_tx,
            item_image_rx,
            new_item_tx,
            current_search_thread,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let span = trace_span!("show items window");
        let _guard = span.enter();

        if let Ok(items) = self.items_rx.try_recv() {
            self.items = items
                .1
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    self.load_item_image(ui.ctx(), &item, index);
                    (item, AsyncStatus::Loading)
                })
                .collect_vec();
        }

        self.item_image_rx.try_iter().for_each(|(index, image)| {
            self.items[index].1 = AsyncStatus::Ready(image);
        });

        let quantity_modal = Modal::new(ui.ctx(), "test modal");

        // quantity_modal.show_dialog();
        quantity_modal.show(|ui| {
            ui.label("Hello world!");
            if ui.button("Close").clicked() {
                quantity_modal.close();
            }
        });

        ui.horizontal_top(|ui| {
            let response = TextEdit::singleline(&mut self.search_bar_text)
                .hint_text("Search")
                .show(ui)
                .response;

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.search_event(ui.ctx());
            }

            if ui.button("Search").clicked() {
                self.search_event(ui.ctx());
            }
        });

        ui.label(self.items.len().to_string());

        ui.horizontal_wrapped(|ui| {
            self.items.iter().for_each(|item| {
                if let AsyncStatus::Ready(ref image) = item.1 {
                    let button = ImageButton::new(image.handle.id(), MainWindow::ITEM_IMAGE_SIZE);
                    let response = ui.add(button);
                    let response = response.on_hover_text(&item.0.name);

                    // if response.clicked_by(PointerButton::Primary) {
                    //     self.get_all_related(item.0.clone(), 1);
                    // }
                    if response.clicked_by(egui::PointerButton::Primary) {
                        self.new_item_tx.send((item.0.clone(), 1)).unwrap();
                    } else if response.clicked_by(egui::PointerButton::Secondary) {
                        quantity_modal.open();
                    }
                }
            });
        });
    }

    fn load_item_image(&self, ctx: &Context, item: &Item, index: usize) {
        let tx = self.item_image_tx.clone();
        let ctx = ctx.clone();
        let image_id = item.image_id;

        tokio::spawn(async move {
            let image = Image::item_from_image_id(&ctx, image_id as _);
            tx.send((index, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn search_event(&mut self, ctx: &Context) {
        if let Some(ref current_search) = self.current_search_thread {
            if current_search.0 != self.search_bar_text {
                current_search.1.abort();
                self.search_items(ctx);
            }
        } else {
            self.search_items(ctx);
        }
    }

    fn search_items(&mut self, ctx: &Context) {
        use crate::database::functions::f_unaccent;
        use crate::database::schema::{items, items::*};
        use diesel::prelude::*;

        let tx = self.items_tx.clone();
        let pool = self.pool.clone();
        let search_text = self.search_bar_text.clone();
        let ctx = ctx.clone();

        self.current_search_thread = Some((
            search_text.clone(),
            tokio::spawn(async move {
                let mut connection = pool.get().unwrap();

                let items = items::table
                    .filter(f_unaccent(name).ilike(f_unaccent(format!("%{search_text}%"))))
                    .filter(category.eq_any(Self::CATEGORY_SEARCHED))
                    .filter(image_id.ne(89042))
                    .limit(100)
                    .offset(0)
                    .load::<Item>(&mut connection)
                    .unwrap();

                tx.send((search_text, items)).unwrap();
                ctx.request_repaint();
            }),
        ));
    }
}
