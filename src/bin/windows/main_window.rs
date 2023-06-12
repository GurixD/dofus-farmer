use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use egui::{
    CentralPanel, Color32, ColorImage, Context, Frame, Pos2, Rect, TextureHandle, Ui, Vec2, Window,
};
use image::io::Reader;
use lombok::AllArgsConstructor;

pub enum ImageStatus {
    Loading,
    Ready(Image),
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
        let handle = ctx.load_texture(&format!("map{index}"), color_image, Default::default());

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
    position: Pos2,
    clicked_position: Option<Pos2>,
    images: HashMap<(u16, usize), ImageStatus>,
    image_number: (u8, u8),
    tx: Sender<(Image, u16, usize)>,
    rx: Receiver<(Image, u16, usize)>,
}

impl MainWindow {
    const IMAGE_SIZE: Vec2 = Vec2::new(250f32, 250f32);
    const FULL_IMAGE_SIZE: Vec2 = Vec2::new(10000f32, 8000f32);
    const ZOOMS: [f32; 5] = [0.2, 0.4, 0.6, 0.8, 1f32];
    const STARTING_ZOOM_INDEX: usize = 0;

    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn body_loop(&mut self, x: i32, y: i32, pos: Pos2, ui: &Ui) {
        let new_x = x - pos.x as i32;
        let new_y = y - pos.y as i32;

        let x_index = (new_x as f32 / Self::IMAGE_SIZE.x).floor() as i8;
        let y_index = (new_y as f32 / Self::IMAGE_SIZE.y).floor() as i8;

        if (0..self.image_number.0 as i8).contains(&x_index)
            && (0..self.image_number.1 as i8).contains(&y_index)
        {
            let index = y_index as u16 * self.image_number.0 as u16 + x_index as u16;
            if let Some(image_status) = self.images.get_mut(&(index, self.zoom_index)) {
                if let ImageStatus::Ready(image) = image_status {
                    let pos = Pos2::new(x as f32, y as f32);

                    // println!("drawing {} in {:?}", image.name(), pos);
                    ui.painter().image(
                        image.handle.id(),
                        Rect::from_two_pos(pos, pos + Self::IMAGE_SIZE),
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
        let ctx = ui.ctx();
        ui.input(|input_state| {
            if ui.ui_contains_pointer() {
                if input_state.pointer.primary_pressed() {
                    self.clicked_position = input_state.pointer.interact_pos();
                }

                if input_state
                    .pointer
                    .button_clicked(egui::PointerButton::Middle)
                {
                    self.position = Pos2::ZERO;
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
                    self.position += ctx.pointer_interact_pos().unwrap() - clicked_position;
                    self.clicked_position = None;
                }
            }
        });

        let pos = self.position
            + self
                .clicked_position
                .map(|pos| ctx.pointer_interact_pos().unwrap() - pos)
                .unwrap_or(Vec2::ZERO);

        self.images.iter_mut().for_each(|(_, image_status)| {
            if let ImageStatus::Ready(ref mut image) = image_status {
                image.used = false;
            }
        });

        let top_left = Pos2::ZERO;

        let size = ui.available_size();
        let rect = Rect::from_two_pos(top_left, top_left + size);

        let left = pos.x % Self::IMAGE_SIZE.x;
        let top = pos.y % Self::IMAGE_SIZE.y;
        let right = rect.right();
        let bottom = rect.bottom();
        for x in (left as i32..=right as i32).step_by(Self::IMAGE_SIZE.x as usize) {
            for y in (top as i32..=bottom as i32).step_by(Self::IMAGE_SIZE.y as usize) {
                self.body_loop(x, y, pos, ui);
            }
        }

        self.images.retain(|_, image_status| {
            if let ImageStatus::Ready(ref image) = image_status {
                return image.used;
            }
            true
        });
    }

    fn load_image(&mut self, ctx: Context, index: u16) {
        let tx = self.tx.clone();
        let zoom_index = self.zoom_index;
        let zoom = Self::ZOOMS[zoom_index];
        tokio::spawn(async move {
            let image = Image::from_ui_and_index(&ctx, index, zoom);
            let _ = tx.send((image, index, zoom_index));
            ctx.request_repaint();
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
        self.image_number = Self::create_zoom(zoom_index);
    }

    fn create_zoom(zoom_index: usize) -> (u8, u8) {
        let zoom = Self::ZOOMS[zoom_index];
        (
            ((Self::FULL_IMAGE_SIZE.x * zoom) / Self::IMAGE_SIZE.x).ceil() as u8,
            ((Self::FULL_IMAGE_SIZE.y * zoom) / Self::IMAGE_SIZE.y).ceil() as u8,
        )
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let zoom_index = Self::STARTING_ZOOM_INDEX;
        let image_number = Self::create_zoom(zoom_index);

        println!("{image_number:?}");

        Self {
            zoom_index,
            position: Pos2::ZERO,
            clicked_position: None,
            images: HashMap::new(),
            image_number,
            tx,
            rx,
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let images_loaded_length = self
            .images
            .iter()
            .filter(|(_, image_status)| {
                if let ImageStatus::Ready(_) = image_status {
                    return true;
                }

                false
            })
            .count();

        let images_loading_length = self
            .images
            .iter()
            .filter(|(_, image_status)| {
                if let ImageStatus::Ready(_) = image_status {
                    return false;
                }

                true
            })
            .count();

        // println!("{images_loaded_length} images loaded, {images_loading_length} images loading");
        self.rx.try_iter().for_each(|(image, index, zoom_index)| {
            if zoom_index == self.zoom_index {
                self.images
                    .insert((index, self.zoom_index), ImageStatus::Ready(image));
            }
        });

        let frame = Frame::default().fill(Color32::from_rgb(30, 25, 25));
        CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| self.central_panel_ui(ui));

        Window::new("hello window").show(ctx, |_ui| {
            //
        });
    }
}
