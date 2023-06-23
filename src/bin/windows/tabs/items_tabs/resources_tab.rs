use std::sync::mpsc;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use egui::{
    epaint::ahash::{HashMap, HashMapExt},
    Context, ImageButton, Layout, Ui, Vec2,
};
use futures::executor::block_on;
use tokio::sync::broadcast;

use crate::{
    database::models::{item::Item, recipe::Recipe},
    windows::main_window::{AsyncStatus, Image, ItemsRelations, MainWindow},
};

pub struct ResourcesTab {
    pool: Pool<ConnectionManager<PgConnection>>,
    new_item_tx: broadcast::Receiver<(Item, usize)>,
    new_ingredients_tx: mpsc::Sender<HashMap<Item, usize>>,
    new_ingredients_rx: mpsc::Receiver<HashMap<Item, usize>>,
    item_image_tx: mpsc::Sender<(Item, Image)>,
    item_image_rx: mpsc::Receiver<(Item, Image)>,
    ingredients: HashMap<Item, (usize, AsyncStatus<Image>)>,
}

impl ResourcesTab {
    pub fn new(
        pool: Pool<ConnectionManager<PgConnection>>,
        new_item_tx: broadcast::Receiver<(Item, usize)>,
    ) -> Self {
        let (new_ingredients_tx, new_ingredients_rx) = mpsc::channel();
        let (item_image_tx, item_image_rx) = mpsc::channel();
        let ingredients = Default::default();

        Self {
            pool,
            new_item_tx,
            new_ingredients_tx,
            new_ingredients_rx,
            item_image_tx,
            item_image_rx,
            ingredients,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, items: &ItemsRelations) {
        for _ in 0..self.new_item_tx.len() {
            let (item, quantity) = block_on(self.new_item_tx.recv()).unwrap();
            self.item_base_recipe(ui.ctx(), item, quantity);
        }

        self.new_ingredients_rx.try_iter().for_each(|ingredients| {
            ingredients.into_iter().for_each(|(ingredient, quantity)| {
                self.ingredients
                    .entry(ingredient.clone())
                    .and_modify(|current_quantity| current_quantity.0 += quantity)
                    .or_insert_with(|| {
                        Self::load_item_image(self.item_image_tx.clone(), &ingredient, ui.ctx());
                        (quantity, AsyncStatus::Loading)
                    });
            });
        });

        self.item_image_rx.try_iter().for_each(|(item, image)| {
            if let Some((_, loading_image)) = self.ingredients.get_mut(&item) {
                *loading_image = AsyncStatus::Ready(image);
            }
        });

        ui.horizontal_wrapped(|ui| {
            self.ingredients.iter().for_each(|(_, (quantity, image))| {
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
            });
        });
    }

    fn item_base_recipe(&self, ctx: &Context, item: Item, quantity: usize) {
        // test 8993
        // select * from items JOIN recipes on recipes.result_item_id = items.id JOIN items as items2 on recipes.ingredient_item_id = items2.id  where items.id = 8993;
        // diesel::joinable!(recipes -> items (result_item_id));
        // diesel::joinable!(recipes -> items (ingredient_item_id));
        use crate::database::schema::*;
        use diesel::prelude::*;

        let pool = self.pool.clone();
        let tx = self.new_ingredients_tx.clone();
        let ctx = ctx.clone();

        tokio::spawn(async move {
            let mut ingredients_quantity = HashMap::new();
            let mut items_to_make = vec![(item, quantity)];

            let mut connection = pool.get().unwrap();

            let (items_result, items_ingredient) =
                diesel::alias!(items as items_result, items as items_ingredient);

            while !items_to_make.is_empty() {
                let (item, quantity) = items_to_make.pop().unwrap();

                let result: Vec<(Item, Recipe, Item)> = items_result
                    .inner_join(
                        recipes::table
                            .on(items_result.field(items::id).eq(recipes::result_item_id)),
                    )
                    .inner_join(
                        items_ingredient.on(items_ingredient
                            .field(items::id)
                            .eq(recipes::ingredient_item_id)),
                    )
                    .filter(items_result.field(items::id).eq(item.id))
                    .load(&mut connection)
                    .unwrap();

                if result.is_empty() {
                    ingredients_quantity
                        .entry(item)
                        .and_modify(|current_quantity| *current_quantity += quantity)
                        .or_insert(quantity);
                } else {
                    items_to_make.extend(result.into_iter().map(
                        |(_, recipe, items_ingredient)| (items_ingredient, recipe.quantity as _),
                    ));
                }
            }

            tx.send(ingredients_quantity).unwrap();
            ctx.request_repaint();
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
