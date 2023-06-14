use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use diesel::PgConnection;
use egui::{
    CentralPanel, Color32, ColorImage, Context, Frame, InputState, Pos2, Rect, Rounding,
    TextureHandle, Ui, Vec2, Window,
};
use image::io::Reader;
use lombok::AllArgsConstructor;
use tracing::trace_span;

enum ImageStatus {
    Loading,
    Ready(Image),
}

#[derive(AllArgsConstructor)]
pub struct MapMinMax {
    x_min: i16,
    x_max: i16,
    y_min: i16,
    y_max: i16,
}

#[derive(AllArgsConstructor)]
pub struct Image {
    handle: TextureHandle,
    used: bool,
}

impl Image {
    pub fn from_ui_and_index(ctx: &Context, index: u16, zoom: f32) -> Self {
        // Images start at 1
        let color_image = Self::load_image_from_path(&format!(
            "src/resources/worldmap/{}/{}.jpg",
            zoom,
            index + 1
        ));
        let handle = ctx.load_texture(format!("map{index}"), color_image, Default::default());

        Image::new(handle, true)
    }

    fn load_image_from_path(path: &str) -> ColorImage {
        println!("Loading file {path}");
        let image = Reader::open(path).unwrap().decode().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }
}

pub struct MainWindow {
    zoom_index: usize,
    map_position: Pos2,
    clicked_position: Option<Pos2>,
    images: HashMap<(u16, usize), ImageStatus>,
    images_number: (u8, u8),
    map_min_max: MapMinMax,
    tx: Sender<(Image, u16, usize)>,
    rx: Receiver<(Image, u16, usize)>,
    connection: PgConnection,
}

impl MainWindow {
    const IMAGE_SIZE: Vec2 = Vec2::new(250f32, 250f32);
    const FULL_IMAGE_SIZE: Vec2 = Vec2::new(10000f32, 8000f32);
    const ZOOMS: [f32; 5] = [0.2, 0.4, 0.6, 0.8, 1f32];
    const STARTING_ZOOM_INDEX: usize = 4;
    const MAPS_RECT: Rect = Self::init_map_rect();

    const fn init_map_rect() -> Rect {
        let min = Pos2::new(360f32, 0f32);
        let max = Pos2::new(
            (min.x as i32 + Self::FULL_IMAGE_SIZE.x as i32) as f32,
            (min.y as i32 + Self::FULL_IMAGE_SIZE.y as i32) as f32,
        );

        Rect { min, max }
    }

    pub fn new(_: &eframe::CreationContext<'_>, mut connection: PgConnection) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let zoom_index = Self::STARTING_ZOOM_INDEX;
        let images_number = Self::create_zoom(zoom_index);
        let map_min_max = {
            use crate::database::schema::maps;
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

        Self {
            zoom_index,
            map_position: Pos2::ZERO,
            clicked_position: None,
            images: HashMap::new(),
            images_number,
            map_min_max,
            tx,
            rx,
            connection,
        }
    }

    fn draw_map_body_loop(&mut self, x: i32, y: i32, pos: Pos2, ui: &Ui) {
        let span = trace_span!("draw_map_body_loop");
        let _guard = span.enter();

        let new_x = x - pos.x as i32;
        let new_y = y - pos.y as i32;

        let x_index = (new_x as f32 / Self::IMAGE_SIZE.x).floor() as i8;
        let y_index = (new_y as f32 / Self::IMAGE_SIZE.y).floor() as i8;

        if (0..self.images_number.0 as i8).contains(&x_index)
            && (0..self.images_number.1 as i8).contains(&y_index)
        {
            let index = y_index as u16 * self.images_number.0 as u16 + x_index as u16;
            if let Some(image_status) = self.images.get_mut(&(index, self.zoom_index)) {
                if let ImageStatus::Ready(image) = image_status {
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
                self.images
                    .insert((index, self.zoom_index), ImageStatus::Loading);
                self.load_image(ui.ctx().clone(), index);
            }
        }
    }

    fn central_panel_ui(&mut self, ui: &Ui) {
        let span = trace_span!("central_panel_ui");
        let _guard = span.enter();

        let ctx = ui.ctx();
        let ui_contains_pointer = ui.ui_contains_pointer();
        let pointer_pos = ui.input(|input_state| self.on_input(input_state, ui_contains_pointer));

        let fullmap_position = self.map_position
            + self
                .clicked_position
                .map(|pos| ctx.pointer_latest_pos().unwrap() - pos)
                .unwrap_or(Vec2::ZERO);

        let pointer_pos_on_map = ctx
            .pointer_latest_pos()
            .map(|pos| (pos - fullmap_position).to_pos2());
        let pointer_pos_on_map_zoomed =
            pointer_pos_on_map.map(|pos| (pos.to_vec2() / Self::ZOOMS[self.zoom_index]).to_pos2());

        // println!("{:?}", cursor_pos_on_map);

        // Draw full map images
        self.reset_images_flags();

        let size = ui.available_size();
        println!("{size:?}");

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

        // Draw map rect on pointer
        if let Some(pointer_pos) = pointer_pos {
            if let Some(pointer_pos_on_map_zoomed) = pointer_pos_on_map_zoomed {
                // if Self::MAPS_RECT.contains(pointer_pos_on_map)
                {
                    let zoom = Self::ZOOMS[self.zoom_index];
                    let rect_size = Vec2::new(
                        (Self::MAPS_RECT.width() * zoom)
                            / (self.map_min_max.x_max - self.map_min_max.x_min) as f32,
                        (Self::MAPS_RECT.height() * zoom)
                            / (self.map_min_max.y_max - self.map_min_max.y_min) as f32,
                    );

                    let offset = (
                        Self::MAPS_RECT.left() % rect_size.x,
                        Self::MAPS_RECT.top() % rect_size.y,
                    );

                    let x = ((pointer_pos_on_map_zoomed.x - offset.0) / rect_size.x).floor()
                        * rect_size.x
                        + fullmap_position.x
                        + offset.0;
                    let y = ((pointer_pos_on_map_zoomed.y - offset.1) / rect_size.y).floor()
                        * rect_size.y
                        + fullmap_position.y
                        + offset.1;

                    let map_pos = Pos2::new(x, y);

                    let rect = Rect::from_two_pos(map_pos, map_pos + rect_size);
                    ui.painter().rect_filled(
                        rect,
                        Rounding::none(),
                        Color32::from_rgba_unmultiplied(60, 180, 255, 100),
                    );
                }
            }
        }
    }

    fn on_input(&mut self, input_state: &InputState, ui_contains_pointer: bool) -> Option<Pos2> {
        let span = trace_span!("read inputs");
        let _guard = span.enter();

        if ui_contains_pointer {
            let span = trace_span!("ui_contains_pointer");
            let _guard = span.enter();

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
                self.zoom_out();
            } else if scroll_delta < 0f32 {
                self.zoom_in();
            }
        }

        if input_state.pointer.primary_released() {
            if let Some(clicked_position) = self.clicked_position {
                self.map_position += input_state.pointer.interact_pos().unwrap() - clicked_position;
                self.clicked_position = None;
            }
        }

        input_state
            .pointer
            .interact_pos()
            .filter(|_| ui_contains_pointer)
    }

    fn reset_images_flags(&mut self) {
        let span = trace_span!("reset_images_flags");
        let _guard = span.enter();

        self.images.iter_mut().for_each(|(_, image_status)| {
            if let ImageStatus::Ready(ref mut image) = image_status {
                image.used = false;
            }
        });
    }

    fn check_images_flags(&mut self) {
        let span = trace_span!("check_images_flags");
        let _guard = span.enter();

        self.images.retain(|_, image_status| {
            if let ImageStatus::Ready(ref image) = image_status {
                return image.used;
            }
            true
        });
    }

    fn load_image(&mut self, ctx: Context, index: u16) {
        let span = trace_span!("load_image");
        let _guard = span.enter();

        let tx = self.tx.clone();
        let zoom_index = self.zoom_index;
        let zoom = Self::ZOOMS[zoom_index];
        tokio::spawn(async move {
            let span = trace_span!("load_image inner async");
            let _guard = span.enter();

            let image = Image::from_ui_and_index(&ctx, index, zoom);
            let _ = tx.send((image, index, zoom_index));
            ctx.request_repaint();
        });
    }

    fn check_for_new_images(&mut self) {
        let span = trace_span!("check_for_new_images");
        let _guard = span.enter();

        self.rx.try_iter().for_each(|(image, index, zoom_index)| {
            if zoom_index == self.zoom_index {
                self.images
                    .insert((index, self.zoom_index), ImageStatus::Ready(image));
            }
        });
    }

    fn zoom_in(&mut self) {
        if self.zoom_index > 0 {
            self.update_zoom(self.zoom_index - 1);
        }
    }

    fn zoom_out(&mut self) {
        if self.zoom_index < Self::ZOOMS.len() - 1 {
            self.update_zoom(self.zoom_index + 1);
        }
    }

    fn update_zoom(&mut self, zoom_index: usize) {
        self.images.clear();
        self.zoom_index = zoom_index;
        self.images_number = Self::create_zoom(zoom_index);
    }

    fn create_zoom(zoom_index: usize) -> (u8, u8) {
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

        let _images_loaded_length = self
            .images
            .iter()
            .filter(|(_, image_status)| {
                if let ImageStatus::Ready(_) = image_status {
                    return true;
                }

                false
            })
            .count();

        let _images_loading_length = self
            .images
            .iter()
            .filter(|(_, image_status)| {
                if let ImageStatus::Ready(_) = image_status {
                    return false;
                }

                true
            })
            .count();

        // println!("{_images_loaded_length} images loaded, {_images_loading_length} images loading");
        self.check_for_new_images();

        let frame = Frame::default().fill(Color32::from_rgb(30, 25, 25));
        CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| self.central_panel_ui(ui));

        Window::new("hello window").show(ctx, |_ui| {
            //
        });
    }
}
