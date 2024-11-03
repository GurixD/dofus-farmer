use std::sync::{
    self,
    mpsc::{Receiver, Sender},
};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{Context, ImageButton, TextEdit, Ui};
use egui_modal::Modal;
use itertools::Itertools;
use tokio::task::JoinHandle;
use tracing::trace_span;

use crate::{
    database::models::item::Item,
    windows::{
        items_window::ItemsWindow,
        main_window::{AsyncStatus, Image},
    },
};

pub struct SearchItemTab {
    search_bar_text: String,
    modal_quantity: String,
    modal_clicked_item: Option<Item>,
    items: Vec<(Item, AsyncStatus<Image>)>,
    pool: Pool<ConnectionManager<PgConnection>>,
    items_tx: Sender<(String, Vec<Item>)>,
    items_rx: Receiver<(String, Vec<Item>)>,
    item_image_tx: Sender<(usize, Image)>,
    item_image_rx: Receiver<(usize, Image)>,
    item_clicked_tx: Sender<(Item, usize)>,
    current_search_thread: Option<(String, JoinHandle<()>)>,
}

impl SearchItemTab {
    const CATEGORY_SEARCHED: [i16; 3] = [0, 1, 2];

    pub fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        item_clicked_tx: Sender<(Item, usize)>,
    ) -> Self {
        let search_bar_text = Default::default();
        let modal_quantity = Default::default();
        let modal_clicked_item = None;
        let items = Default::default();
        let (items_tx, items_rx) = sync::mpsc::channel();
        let (item_image_tx, item_image_rx) = sync::mpsc::channel();
        let current_search_thread = None;

        Self {
            search_bar_text,
            modal_quantity,
            modal_clicked_item,
            items,
            pool,
            items_tx,
            items_rx,
            item_image_tx,
            item_image_rx,
            item_clicked_tx,
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

        let quantity_modal = Modal::new(ui.ctx(), "search modal");

        quantity_modal.show(|ui| {
            ui.text_edit_singleline(&mut self.modal_quantity);
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    self.modal_clicked_item = None;
                    quantity_modal.close();
                } else if ui.button("Add").clicked() {
                    if let Ok(quantity) = self.modal_quantity.parse::<usize>() {
                        let item = self.modal_clicked_item.take();
                        self.item_clicked_tx
                            .send((item.unwrap(), quantity))
                            .unwrap();
                        quantity_modal.close();
                    }
                }
            });
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
                    let button = ImageButton::new(egui::Image::from_texture(
                        ItemsWindow::get_sized_texture(image),
                    ));
                    let response = ui.add(button);
                    let response = response.on_hover_text(&item.0.name);

                    if response.clicked() {
                        self.item_clicked_tx.send((item.0.clone(), 1)).unwrap();
                    } else if response.secondary_clicked() {
                        self.modal_clicked_item = Some(item.0.clone());
                        self.modal_quantity = Default::default();
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
