use std::{
    cmp,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
};

use diesel::{
    delete, insert_into,
    r2d2::{ConnectionManager, Pool},
    update, PgConnection,
};
use egui::{
    CentralPanel, Color32, ColorImage, Context, Frame, InputState, PointerButton, Pos2, Rect,
    Rounding, TextureHandle, Ui, Vec2,
};
use image::io::Reader;
use lombok::AllArgsConstructor;
use tracing::{trace_span, warn};

use crate::database::{
    models::{
        drop::Drop, item::Item, map::Map, monster::Monster, monster_sub_area::MonsterSubArea,
        recipe::Recipe, sub_area::SubArea, user_ingredient::UserIngredient, user_item::UserItem,
    },
    schema::maps,
};

use super::items_window::ItemsWindow;

#[derive(Clone)]
pub enum AsyncStatus<T> {
    Loading,
    Ready(T),
}

#[derive(AllArgsConstructor)]
pub struct MapMinMax {
    x_min: i16,
    x_max: i16,
    y_min: i16,
    y_max: i16,
}

#[derive(AllArgsConstructor, Clone)]
pub struct Image {
    pub handle: TextureHandle,
    pub used: bool,
}

impl Image {
    pub fn from_path(ctx: &Context, path: &str) -> Self {
        // Images start at 1
        let color_image = Self::load_image_from_path(path);
        let handle = ctx.load_texture(path.to_owned(), color_image, Default::default());

        Image::new(handle, true)
    }

    pub fn from_ui_and_index(ctx: &Context, index: u16, zoom: f32) -> Self {
        // Images start at 1
        let path = format!("src/resources/images/worldmap/{}/{}.jpg", zoom, index + 1);
        Self::from_path(ctx, &path)
    }

    pub fn item_from_image_id(ctx: &Context, id: i32) -> Self {
        let path = format!("src/resources/images/items/{id}.png");
        Self::from_path(ctx, &path)
    }

    pub fn monster_from_id(ctx: &Context, id: i32) -> Self {
        let path = format!("src/resources/images/monsters/{id}.png");
        Self::from_path(ctx, &path)
    }

    fn load_image_from_path(path: &str) -> ColorImage {
        let span = trace_span!("draw_map_body_loop");
        let _guard = span.enter();

        // println!("{path}");
        let image = Reader::open(path).unwrap().decode().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }
}

pub type ItemsRelations = BTreeMap<
    Rc<Item>, // item to craft
    (
        usize, // quantity
        AsyncStatus<
            HashMap<
                Rc<Item>, // one of the resources needed to make it
                (
                    usize,                                  // quantity needed
                    HashMap<Rc<Monster>, HashSet<SubArea>>, // monsters and their sub areas
                ),
            >,
        >,
    ),
>;

type Ingredients = (
    Item,
    usize,
    HashMap<Item, (usize, HashMap<Monster, HashSet<SubArea>>)>,
);

pub struct MainWindow {
    zoom_index: usize,
    map_position: Pos2,
    clicked_position: Option<Pos2>,
    maps_images: HashMap<(u16, usize), AsyncStatus<Image>>,
    images_number: (u8, u8),
    map_min_max: MapMinMax,
    sub_areas: HashMap<SubArea, Vec<Map>>,
    current_sub_area: Option<SubArea>,
    clicked_map: Option<(f32, f32)>,
    map_tx: Sender<(Image, u16, usize)>,
    map_rx: Receiver<(Image, u16, usize)>,
    item_rx: Receiver<(Item, usize)>,
    remove_item_rx: Receiver<(Item, usize, bool)>,
    item_ingredients_tx: Sender<Ingredients>,
    item_ingredients_rx: Receiver<Ingredients>,
    item_image_tx: Sender<(Item, Image)>,
    item_image_rx: Receiver<(Item, Image)>,
    items_images: HashMap<Rc<Item>, AsyncStatus<Image>>,
    monster_image_tx: Sender<(Monster, Image)>,
    monster_image_rx: Receiver<(Monster, Image)>,
    new_ingredient_rx: Receiver<(Item, isize)>,
    monsters_images: HashMap<Rc<Monster>, AsyncStatus<Image>>,
    items: ItemsRelations,
    ingredients_quantity: HashMap<Item, usize>,
    items_window: ItemsWindow,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl MainWindow {
    const IMAGE_SIZE: Vec2 = Vec2::new(250f32, 250f32);
    const FULL_IMAGE_SIZE: Vec2 = Vec2::new(10000f32, 8000f32);
    const ZOOMS: [f32; 5] = [0.2, 0.4, 0.6, 0.8, 1f32];
    const STARTING_ZOOM_INDEX: usize = 0;
    const MAPS_RECT: Rect = Self::init_map_rect();
    pub const ITEM_IMAGE_SIZE: Vec2 = Vec2 { x: 60f32, y: 60f32 };

    const fn init_map_rect() -> Rect {
        let min = Pos2::new(360f32, 320f32);
        let max = Pos2::new(9540f32, 7575f32);

        Rect { min, max }
    }

    pub fn new(
        _: &eframe::CreationContext<'_>,
        pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Self {
        let (map_tx, map_rx) = mpsc::channel();
        let (item_tx, item_rx) = mpsc::channel();
        let (remove_item_tx, remove_item_rx) = mpsc::channel();
        let (item_ingredients_tx, item_ingredients_rx) = mpsc::channel();
        let (item_image_tx, item_image_rx) = mpsc::channel();
        let (monster_image_tx, monster_image_rx) = mpsc::channel();
        let (new_ingredient_tx, new_ingredient_rx) = mpsc::channel();

        let mut connection = pool.get().unwrap();

        let zoom_index = Self::STARTING_ZOOM_INDEX;
        let images_number = Self::image_number_from_zoom(zoom_index);
        let map_min_max = {
            use diesel::dsl::{max, min};
            use diesel::prelude::*;

            let min_max = maps::table
                .select((
                    min(maps::x).assume_not_null(),
                    max(maps::x).assume_not_null(),
                    min(maps::y).assume_not_null(),
                    max(maps::y).assume_not_null(),
                ))
                .first::<(i16, i16, i16, i16)>(&mut connection)
                .unwrap();

            MapMinMax::new(
                min_max.0 as _,
                min_max.1 as _,
                min_max.2 as _,
                min_max.3 as _,
            )
        };
        let sub_areas = {
            use crate::database::schema::sub_areas;
            use diesel::prelude::*;

            let sub_areas = sub_areas::table
                .select(SubArea::as_select())
                .load(&mut connection)
                .unwrap();

            let maps = Map::belonging_to(&sub_areas)
                .select(Map::as_select())
                .load(&mut connection)
                .unwrap();

            let mut maps_per_sub_area: HashMap<SubArea, Vec<Map>> = maps
                .grouped_by(&sub_areas)
                .into_iter()
                .zip(sub_areas)
                .map(|(maps, sub_area)| (sub_area, maps))
                .collect();

            maps_per_sub_area.retain(|_, vec| !vec.is_empty());

            maps_per_sub_area
        };
        let ingredients_quantity = {
            use crate::database::schema::{items, user_ingredients, user_items};
            use diesel::prelude::*;

            let user_items: Vec<(UserItem, Item)> = user_items::table
                .inner_join(items::table)
                .load(&mut connection)
                .unwrap();

            user_items.into_iter().for_each(|(user_item, item)| {
                item_tx.send((item, user_item.quantity as _)).unwrap()
            });

            user_ingredients::table
                .inner_join(items::table)
                .load::<(UserIngredient, Item)>(&mut connection)
                .unwrap()
                .into_iter()
                .map(|(user_ingredient, item)| (item, user_ingredient.quantity as usize))
                .collect()
        };

        let current_sub_area = None;
        let clicked_map = None;

        let maps_images = HashMap::new();
        let items = BTreeMap::new();
        let items_images = HashMap::new();
        let monsters_images = HashMap::new();

        let items_window =
            ItemsWindow::new(pool.clone(), item_tx, new_ingredient_tx, remove_item_tx);

        Self {
            zoom_index,
            map_position: Pos2::ZERO,
            clicked_position: None,
            maps_images,
            images_number,
            map_min_max,
            sub_areas,
            current_sub_area,
            clicked_map,
            map_tx,
            map_rx,
            item_rx,
            remove_item_rx,
            item_ingredients_tx,
            item_ingredients_rx,
            item_image_tx,
            item_image_rx,
            items_images,
            monster_image_tx,
            monster_image_rx,
            new_ingredient_rx,
            monsters_images,
            items,
            ingredients_quantity,
            items_window,
            pool,
        }
    }

    fn draw_map_body_loop(&mut self, x: i32, y: i32, pos: Pos2, ui: &Ui) {
        let new_x = x - pos.x as i32;
        let new_y = y - pos.y as i32;

        let x_index = (new_x as f32 / Self::IMAGE_SIZE.x).floor() as i8;
        let y_index = (new_y as f32 / Self::IMAGE_SIZE.y).floor() as i8;

        if (0..self.images_number.0 as i8).contains(&x_index)
            && (0..self.images_number.1 as i8).contains(&y_index)
        {
            let index = y_index as u16 * self.images_number.0 as u16 + x_index as u16;
            if let Some(image_status) = self.maps_images.get_mut(&(index, self.zoom_index)) {
                if let AsyncStatus::Ready(image) = image_status {
                    let pos = Pos2::new(x as f32, y as f32);

                    ui.painter().image(
                        image.handle.id(),
                        Rect::from_two_pos(pos, pos + image.handle.size_vec2()),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1f32, 1f32)),
                        Color32::WHITE,
                    );

                    image.used = true;
                }
            } else {
                self.maps_images
                    .insert((index, self.zoom_index), AsyncStatus::Loading);
                self.load_map_image(ui.ctx().clone(), index);
            }
        }
    }

    fn central_panel_ui(&mut self, ui: &Ui) {
        let ctx = ui.ctx();
        let ui_contains_pointer = ui.ui_contains_pointer();
        let (pointer_pos, double_clicked, right_clicked) =
            ui.input(|input_state| self.on_input(input_state, ui_contains_pointer));

        if right_clicked {
            self.clicked_map = None;
        }

        let fullmap_position = self.map_position
            + self
                .clicked_position
                .map(|pos| ctx.pointer_latest_pos().unwrap() - pos)
                .unwrap_or(Vec2::ZERO);

        let pointer_pos_on_map = pointer_pos.map(|pos| (pos - fullmap_position).to_pos2());
        let pointer_pos_on_map_zoomed =
            pointer_pos_on_map.map(|pos| (pos.to_vec2() / Self::ZOOMS[self.zoom_index]).to_pos2());

        // Draw full map images
        self.reset_images_flags();

        let size = ui.available_size();
        let left = fullmap_position.x % Self::IMAGE_SIZE.x;
        let top = fullmap_position.y % Self::IMAGE_SIZE.y;
        let right = size.x;
        let bottom = size.y;
        for x in (left as i32..=right as i32).step_by(Self::IMAGE_SIZE.x as usize) {
            for y in (top as i32..=bottom as i32).step_by(Self::IMAGE_SIZE.y as usize) {
                self.draw_map_body_loop(x, y, fullmap_position, ui);
            }
        }

        self.check_images_flags();

        self.current_sub_area = None;

        if let Some((x_index, y_index)) = self.clicked_map {
            self.draw_and_filter_map(ui, fullmap_position, x_index, y_index);
        } else if let Some(pointer_pos_on_map_zoomed) = pointer_pos_on_map_zoomed {
            if Self::MAPS_RECT.contains(pointer_pos_on_map_zoomed) {
                let zoom = Self::ZOOMS[self.zoom_index];
                let rect_size = Vec2::new(
                    (Self::MAPS_RECT.width() * zoom)
                        / (self.map_min_max.x_max - self.map_min_max.x_min + 1) as f32,
                    (Self::MAPS_RECT.height() * zoom)
                        / (self.map_min_max.y_max - self.map_min_max.y_min + 1) as f32,
                );

                let offset = (
                    (Self::MAPS_RECT.left() * zoom) % rect_size.x,
                    (Self::MAPS_RECT.top() * zoom) % rect_size.y,
                );

                let x_index =
                    ((pointer_pos_on_map_zoomed.x * zoom - offset.0) / rect_size.x).floor() - 5f32;
                let y_index =
                    ((pointer_pos_on_map_zoomed.y * zoom - offset.1) / rect_size.y).floor() - 6f32;

                if double_clicked {
                    self.clicked_map = Some((x_index, y_index));
                }

                self.draw_and_filter_map(ui, fullmap_position, x_index, y_index);
            }
        }

        let mut sub_areas_to_draw = HashSet::new();
        self.items.iter().for_each(|(_, (_, ingredients))| {
            if let AsyncStatus::Ready(ingredients) = ingredients {
                ingredients.iter().for_each(|(_, (_, monsters))| {
                    monsters.iter().for_each(|(_, sub_areas)| {
                        sub_areas_to_draw.extend(sub_areas);
                    });
                });
            }
        });

        self.sub_areas
            .iter()
            .filter(|(sub_area, _)| sub_areas_to_draw.contains(sub_area))
            .for_each(|(_, maps)| {
                maps.iter().for_each(|map| {
                    self.map_rect_on_pos(ui, map.x as _, map.y as _, fullmap_position, None);
                });
            });
    }

    fn draw_and_filter_map(&mut self, ui: &Ui, fullmap_position: Pos2, x_index: f32, y_index: f32) {
        self.map_rect_on_index(
            ui,
            x_index,
            y_index,
            fullmap_position,
            Some(Color32::from_rgba_unmultiplied(0, 0, 139, 100)),
        );

        let sub_area = self.sub_areas.iter().find(|(_, maps)| {
            maps.iter().any(|map| {
                map.x == x_index as i16 + self.map_min_max.x_min
                    && map.y == y_index as i16 + self.map_min_max.y_min
            })
        });

        if let Some(sub_area) = sub_area {
            sub_area.1.iter().for_each(|map| {
                self.map_rect_on_pos(ui, map.x as _, map.y as _, fullmap_position, None);
            });

            if self.items.iter().any(|(_, (_, ingredients))| {
                if let AsyncStatus::Ready(ingredients) = ingredients {
                    return ingredients.iter().any(|(_, (_, monsters))| {
                        monsters
                            .iter()
                            .any(|(_, sub_areas)| sub_areas.contains(sub_area.0))
                    });
                }
                false
            }) {
                self.current_sub_area = Some(sub_area.0.clone());
            }
        }
    }

    fn map_rect_on_index(
        &self,
        ui: &Ui,
        x_index: f32,
        y_index: f32,
        fullmap_position: Pos2,
        color: Option<Color32>,
    ) {
        let x_index = x_index + 5f32;
        let y_index = y_index + 6f32;

        let zoom = Self::ZOOMS[self.zoom_index];
        let rect_size = Vec2::new(
            (Self::MAPS_RECT.width() * zoom)
                / (self.map_min_max.x_max - self.map_min_max.x_min + 1) as f32,
            (Self::MAPS_RECT.height() * zoom)
                / (self.map_min_max.y_max - self.map_min_max.y_min + 1) as f32,
        );

        let offset = (
            (Self::MAPS_RECT.left() * zoom) % rect_size.x,
            (Self::MAPS_RECT.top() * zoom) % rect_size.y,
        );

        let x = x_index * rect_size.x + fullmap_position.x + offset.0;
        let y = y_index * rect_size.y + fullmap_position.y + offset.1;

        let map_pos = Pos2::new(x, y);

        let rect = Rect::from_two_pos(map_pos, map_pos + rect_size);
        ui.painter().rect_filled(
            rect,
            Rounding::none(),
            color.unwrap_or(Color32::from_rgba_unmultiplied(60, 180, 255, 50)),
        );
    }

    fn map_rect_on_pos(
        &self,
        ui: &Ui,
        x_index: f32,
        y_index: f32,
        fullmap_position: Pos2,
        color: Option<Color32>,
    ) {
        self.map_rect_on_index(
            ui,
            x_index - self.map_min_max.x_min as f32,
            y_index - self.map_min_max.y_min as f32,
            fullmap_position,
            color,
        )
    }

    fn on_input(
        &mut self,
        input_state: &InputState,
        ui_contains_pointer: bool,
    ) -> (Option<Pos2>, bool, bool) {
        if ui_contains_pointer {
            if input_state.pointer.primary_pressed() {
                self.clicked_position = input_state.pointer.interact_pos();
            }

            if input_state
                .pointer
                .button_clicked(egui::PointerButton::Middle)
            {
                self.map_position = Pos2::ZERO;
            }

            let scroll_delta = input_state.scroll_delta.y;
            if scroll_delta > 0f32 {
                self.zoom_out(input_state.pointer.interact_pos().unwrap());
            } else if scroll_delta < 0f32 {
                self.zoom_in(input_state.pointer.interact_pos().unwrap());
            }
        }

        if input_state.pointer.primary_released() {
            if let Some(clicked_position) = self.clicked_position {
                self.map_position += input_state.pointer.interact_pos().unwrap() - clicked_position;
                self.clicked_position = None;
            }
        }

        let pointer_pos = input_state
            .pointer
            .interact_pos()
            .filter(|_| ui_contains_pointer);

        let double_clicked = input_state
            .pointer
            .button_double_clicked(PointerButton::Primary);
        let right_clicked = input_state.pointer.secondary_clicked() && ui_contains_pointer;
        (pointer_pos, double_clicked, right_clicked)
    }

    fn reset_images_flags(&mut self) {
        let span = trace_span!("reset_images_flags");
        let _guard = span.enter();

        self.maps_images.iter_mut().for_each(|(_, image_status)| {
            if let AsyncStatus::Ready(ref mut image) = image_status {
                image.used = false;
            }
        });
    }

    fn check_images_flags(&mut self) {
        let span = trace_span!("check_images_flags");
        let _guard = span.enter();

        self.maps_images.retain(|_, image_status| {
            if let AsyncStatus::Ready(ref image) = image_status {
                return image.used;
            }
            true
        });
    }

    fn load_map_image(&mut self, ctx: Context, index: u16) {
        let span = trace_span!("load_map_image");
        let _guard = span.enter();

        let tx = self.map_tx.clone();
        let zoom_index = self.zoom_index;
        let zoom = Self::ZOOMS[zoom_index];
        tokio::spawn(async move {
            let image = Image::from_ui_and_index(&ctx, index, zoom);
            tx.send((image, index, zoom_index)).unwrap();
            ctx.request_repaint();
        });
    }

    fn load_item_image(tx: Sender<(Item, Image)>, ctx: Context, item: Item) {
        let span = trace_span!("load_item_image");
        let _guard = span.enter();

        tokio::spawn(async move {
            let image = Image::item_from_image_id(&ctx, item.image_id);
            tx.send((item, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn load_monster_image(tx: Sender<(Monster, Image)>, ctx: Context, monster: Monster) {
        let span = trace_span!("load_item_image");
        let _guard = span.enter();

        tokio::spawn(async move {
            let image = Image::monster_from_id(&ctx, monster.id);
            tx.send((monster, image)).unwrap();
            ctx.request_repaint();
        });
    }

    fn check_for_new_map_images(&mut self) {
        let span = trace_span!("check_for_new_images");
        let _guard = span.enter();

        self.map_rx
            .try_iter()
            .for_each(|(image, index, zoom_index)| {
                if zoom_index == self.zoom_index {
                    self.maps_images
                        .insert((index, self.zoom_index), AsyncStatus::Ready(image));
                }
            });
    }

    fn check_for_new_items(&mut self, ctx: &Context) {
        self.item_rx.try_iter().for_each(|(item, quantity)| {
            if let Some((item_value, _)) = self.items.get_mut(&item) {
                *item_value += quantity;
            } else {
                let item_rc = Rc::new(item.clone());

                self.items_images
                    .entry(Rc::clone(&item_rc))
                    .or_insert_with(|| {
                        Self::load_item_image(
                            self.item_image_tx.clone(),
                            ctx.clone(),
                            item.clone(),
                        );
                        AsyncStatus::Loading
                    });

                self.items
                    .insert(item_rc, (quantity as _, AsyncStatus::Loading));
                Self::get_all_related(
                    self.item_ingredients_tx.clone(),
                    self.pool.clone(),
                    item.clone(),
                    quantity as _,
                );
            }

            use crate::database::schema::user_items;
            use diesel::prelude::*;

            let pool = self.pool.clone();

            tokio::spawn(async move {
                let mut connection = pool.get().unwrap();
                let user_item = UserItem::new(item.id, quantity as i16);

                insert_into(user_items::table)
                    .values(&user_item)
                    .on_conflict(user_items::item_id)
                    .do_update()
                    .set(&user_item)
                    .execute(&mut connection)
                    .unwrap();
            });
        });
    }

    fn check_for_removed_item(&mut self) {
        self.remove_item_rx
            .try_iter()
            .for_each(|(item, quantity, crafted)| {
                let (value, ingredients) = self.items.get_mut(&item).unwrap();

                // Cant remove more than what we have
                let quantity = cmp::min(quantity, *value);

                if crafted {
                    if let AsyncStatus::Ready(ingredients) = ingredients {
                        ingredients.iter().for_each(|(item, (needed, _))| {
                            if let Some(in_inventory) = self.ingredients_quantity.get_mut(item) {
                                let to_remove = cmp::min(*in_inventory, needed * quantity);
                                *in_inventory -= to_remove;

                                let pool = self.pool.clone();
                                let item_id = item.id;
                                if *in_inventory == 0 {
                                    self.ingredients_quantity.remove(item);

                                    tokio::spawn(async move {
                                        use crate::database::schema::user_ingredients;
                                        use diesel::prelude::*;

                                        let mut connection = pool.get().unwrap();

                                        delete(user_ingredients::table)
                                            .filter(user_ingredients::item_id.eq(item_id))
                                            .execute(&mut connection)
                                            .unwrap();
                                    });
                                } else {
                                    let user_ingredient =
                                        UserIngredient::new(item_id, *in_inventory as i16);
                                    tokio::spawn(async move {
                                        use crate::database::schema::user_ingredients;
                                        use diesel::prelude::*;

                                        let mut connection = pool.get().unwrap();

                                        update(user_ingredients::table)
                                            .filter(
                                                user_ingredients::item_id
                                                    .eq(user_ingredient.item_id),
                                            )
                                            .set(&user_ingredient)
                                            .execute(&mut connection)
                                            .unwrap();
                                    });
                                }
                            }
                        });
                    }
                }

                let pool = self.pool.clone();
                let item_id = item.id;

                if *value == quantity {
                    self.items.remove(&item);

                    tokio::spawn(async move {
                        use crate::database::schema::user_items;
                        use diesel::prelude::*;

                        let mut connection = pool.get().unwrap();

                        delete(user_items::table)
                            .filter(user_items::item_id.eq(item_id))
                            .execute(&mut connection)
                            .unwrap();
                    });
                } else {
                    *value -= quantity;

                    let user_item = UserItem::new(item_id, *value as i16);
                    tokio::spawn(async move {
                        use crate::database::schema::user_items;
                        use diesel::prelude::*;

                        let mut connection = pool.get().unwrap();

                        update(user_items::table)
                            .filter(user_items::item_id.eq(user_item.item_id))
                            .set(&user_item)
                            .execute(&mut connection)
                            .unwrap();
                    });
                }
            });
    }

    fn check_for_new_item_ingredients(&mut self, ctx: &Context) {
        self.item_ingredients_rx
            .try_iter()
            .for_each(|(item, _, ingredients)| {
                if let Some((_, loading_ingredients)) = self.items.get_mut(&item) {
                    let ingredients_rc: HashMap<_, _> = ingredients
                        .into_iter()
                        .map(|(ingredient, (quantity, monsters_sub_area))| {
                            let ingredient_rc = if let Some((ingredient, _)) =
                                self.items_images.get_key_value(&ingredient)
                            {
                                Rc::clone(ingredient)
                            } else {
                                Self::load_item_image(
                                    self.item_image_tx.clone(),
                                    ctx.clone(),
                                    ingredient.clone(),
                                );
                                let ingredient = Rc::new(ingredient);
                                self.items_images
                                    .insert(Rc::clone(&ingredient), AsyncStatus::Loading);
                                ingredient
                            };

                            let monsters_rc: HashMap<_, _> = monsters_sub_area
                                .into_iter()
                                .map(|(monster, sub_areas)| {
                                    let monster = if let Some((monster, _)) =
                                        self.monsters_images.get_key_value(&monster)
                                    {
                                        Rc::clone(monster)
                                    } else {
                                        Self::load_monster_image(
                                            self.monster_image_tx.clone(),
                                            ctx.clone(),
                                            monster.clone(),
                                        );
                                        let monster = Rc::new(monster);
                                        self.monsters_images
                                            .insert(Rc::clone(&monster), AsyncStatus::Loading);
                                        monster
                                    };

                                    (monster, sub_areas)
                                })
                                .collect();
                            (ingredient_rc, (quantity, monsters_rc))
                        })
                        .collect();
                    *loading_ingredients = AsyncStatus::Ready(ingredients_rc);
                } else {
                    warn!("new item ingredients empty item");
                }
            });
    }

    fn check_for_new_ingredient_in_inventory(&mut self) {
        self.new_ingredient_rx
            .try_iter()
            .for_each(|(item, quantity)| {
                let user_ingredient = UserIngredient::new(item.id, 0);

                let quantity = *self
                    .ingredients_quantity
                    .entry(item)
                    .and_modify(|old_quantity| {
                        *old_quantity = cmp::max(*old_quantity as isize + quantity, 0) as _;
                    })
                    .or_insert(cmp::max(quantity, 0) as _) as i16;

                let user_ingredient = UserIngredient {
                    quantity,
                    ..user_ingredient
                };

                let pool = self.pool.clone();

                use crate::database::schema::user_ingredients;
                use diesel::prelude::*;

                tokio::spawn(async move {
                    let mut connection = pool.get().unwrap();
                    if user_ingredient.quantity == 0 {
                        delete(user_ingredients::table)
                            .filter(user_ingredients::item_id.eq(user_ingredient.item_id))
                            .execute(&mut connection)
                            .unwrap();
                    } else {
                        insert_into(user_ingredients::table)
                            .values(&user_ingredient)
                            .on_conflict(user_ingredients::item_id)
                            .do_update()
                            .set(&user_ingredient)
                            .execute(&mut connection)
                            .unwrap();
                    }
                });
            });
    }

    fn check_for_new_items_images(&mut self) {
        self.item_image_rx.try_iter().for_each(|(item, image)| {
            if let Some(loading_image) = self.items_images.get_mut(&item) {
                *loading_image = AsyncStatus::Ready(image);
            }
        });
    }

    fn check_for_new_monsters_images(&mut self) {
        self.monster_image_rx
            .try_iter()
            .for_each(|(monster, image)| {
                if let Some(loading_image) = self.monsters_images.get_mut(&monster) {
                    *loading_image = AsyncStatus::Ready(image);
                }
            });
    }

    fn get_all_related(
        tx: Sender<Ingredients>,
        pool: Pool<ConnectionManager<PgConnection>>,
        item: Item,
        quantity: usize,
    ) {
        tokio::spawn(async move {
            use crate::database::schema::*;
            use diesel::prelude::*;

            let mut connection = pool.get().unwrap();
            let (items_result, items_ingredient) =
                diesel::alias!(items as items_result, items as items_ingredient);

            let mut ingredients_quantity = HashMap::new();
            let mut items_to_make = vec![(item.clone(), 1)];

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

            let mut result_hash_map: HashMap<Item, (usize, HashMap<Monster, HashSet<SubArea>>)> =
                HashMap::new();

            ingredients_quantity
                .iter()
                .for_each(|(ingredient, quantity)| {
                    let result: Vec<(SubArea, MonsterSubArea, Monster, Drop, Item)> =
                        sub_areas::table
                            .inner_join(
                                monsters_sub_areas::table
                                    .on(sub_areas::id.eq(monsters_sub_areas::sub_area_id)),
                            )
                            .inner_join(
                                monsters::table.on(monsters_sub_areas::monster_id.eq(monsters::id)),
                            )
                            .inner_join(drops::table.on(monsters::id.eq(drops::monster_id)))
                            .inner_join(items::table.on(drops::item_id.eq(items::id)))
                            .filter(items::id.eq(ingredient.id))
                            .load(&mut connection)
                            .unwrap();

                    result_hash_map.insert(ingredient.clone(), (*quantity, Default::default()));
                    let mut sub_areas_for_monsters: HashMap<Monster, HashSet<SubArea>> =
                        HashMap::new();

                    result.into_iter().for_each(|(sub_area, _, monster, _, _)| {
                        sub_areas_for_monsters
                            .entry(monster)
                            .and_modify(|sub_areas| {
                                sub_areas.insert(sub_area.clone());
                            })
                            .or_insert_with(|| {
                                let mut sub_areas = HashSet::new();
                                sub_areas.insert(sub_area);
                                sub_areas
                            });
                    });

                    result_hash_map
                        .entry(ingredient.clone())
                        .and_modify(|(_, monsters)| {
                            *monsters = sub_areas_for_monsters;
                        });
                });

            tx.send((item, quantity, result_hash_map)).unwrap();
        });
    }

    fn zoom_in(&mut self, pointer_pos: Pos2) {
        if self.zoom_index > 0 {
            self.update_zoom(self.zoom_index - 1, pointer_pos);
        }
    }

    fn zoom_out(&mut self, pointer_pos: Pos2) {
        if self.zoom_index < Self::ZOOMS.len() - 1 {
            self.update_zoom(self.zoom_index + 1, pointer_pos);
        }
    }

    fn update_zoom(&mut self, zoom_index: usize, pointer_pos: Pos2) {
        let old_zoom_index = self.zoom_index;

        self.maps_images.clear();
        self.zoom_index = zoom_index;
        self.images_number = Self::image_number_from_zoom(zoom_index);

        self.map_position = pointer_pos
            + ((self.map_position - pointer_pos) / Self::ZOOMS[old_zoom_index]
                * Self::ZOOMS[zoom_index]);
    }

    fn image_number_from_zoom(zoom_index: usize) -> (u8, u8) {
        let zoom = Self::ZOOMS[zoom_index];
        (
            ((Self::FULL_IMAGE_SIZE.x * zoom) / Self::IMAGE_SIZE.x).ceil() as u8,
            ((Self::FULL_IMAGE_SIZE.y * zoom) / Self::IMAGE_SIZE.y).ceil() as u8,
        )
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let span = trace_span!("update");
        let _guard = span.enter();

        self.check_for_new_items(ctx);
        self.check_for_removed_item();
        self.check_for_new_item_ingredients(ctx);
        self.check_for_new_ingredient_in_inventory();
        self.check_for_new_items_images();
        self.check_for_new_monsters_images();
        self.check_for_new_map_images();

        let frame = Frame::default().fill(Color32::from_rgb(30, 25, 25));
        CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| self.central_panel_ui(ui));

        self.items_window.show(
            ctx,
            &self.items,
            &self.ingredients_quantity,
            &self.items_images,
            &self.monsters_images,
            &self.current_sub_area,
        );
    }
}
