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
    pub fn from_ui_and_index(ctx: &Context, index: u16) -> Self {
        // Images start at 1
        let color_image =
            Self::load_image_from_path(&format!("src/resources/worldmap/1/{}.jpg", index + 1));
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
    images: HashMap<u16, ImageStatus>,
    image_number: u16,
    tx: Sender<(Image, u16)>,
    rx: Receiver<(Image, u16)>,
}

impl MainWindow {
    const IMAGE_SIZE: Vec2 = Vec2::new(250f32, 250f32);
    const NUMBER_IMAGE_WIDTH: u16 = 40;
    const MARGIN: f32 = Self::IMAGE_SIZE.x * 2f32;
    const FULL_IMAGE_SIZE: Vec2 = Vec2::new(10000f32, 8000f32);
    const ZOOMS: [f32; 5] = [0.2, 0.4, 0.6, 0.8, 1f32];

    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn body_loop(&mut self, x: i32, y: i32, pos: Pos2, ui: &Ui) {
        let new_x = x - pos.x as i32;
        let new_y = y - pos.y as i32;

        let x_index = (new_x as f32 / Self::IMAGE_SIZE.x).floor() as i8;
        let y_index = (new_y as f32 / Self::IMAGE_SIZE.y).floor() as i8;

        if (0..40).contains(&x_index) && (0..32).contains(&y_index) {
            let index = y_index as u16 * Self::NUMBER_IMAGE_WIDTH + x_index as u16;
            if index <= self.image_number {
                // println!(
                //     "{}, {} => {}, {} => {}, {} => {} => {}",
                //     x, y, new_x, new_y, x_index, y_index, index, new_index
                // );

                if let Some(image_status) = self.images.get_mut(&index) {
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
                    self.images.insert(index, ImageStatus::Loading);
                    self.load_image(ui.ctx().clone(), index);
                }
            }
        }
    }

    fn load_image(&mut self, ctx: Context, index: u16) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let image = Image::from_ui_and_index(&ctx, index);
            let _ = tx.send((image, index));
            ctx.request_repaint();
        });
    }

    fn central_panel_ui(&mut self, ui: &Ui) {
        let ctx = ui.ctx();
        ui.input(|input_state| {
            if input_state.pointer.primary_pressed() && ui.ui_contains_pointer() {
                self.clicked_position = input_state.pointer.interact_pos();
            } else if input_state.pointer.primary_released() {
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

        let left = (pos.x % Self::IMAGE_SIZE.x) - Self::MARGIN;
        let top = (pos.y % Self::IMAGE_SIZE.y) - Self::MARGIN;
        let right = rect.right() + Self::MARGIN;
        let bottom = rect.bottom() + Self::MARGIN;

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
}

impl Default for MainWindow {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            zoom_index: 4,
            position: Pos2::ZERO,
            clicked_position: None,
            images: HashMap::new(),
            image_number: 40 * 32,
            tx,
            rx,
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok((image, index)) = self.rx.try_recv() {
            self.images.insert(index, ImageStatus::Ready(image));
        }

        let frame = Frame::default().fill(Color32::from_rgb(30, 25, 25));
        CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| self.central_panel_ui(ui));

        Window::new("hello window").show(ctx, |ui| {
            //
        });
    }
}
