use egui::{ColorImage, Context, TextureHandle};
use image::ImageReader;
use lombok::AllArgsConstructor;
use tracing::trace_span;

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

    pub fn map_from_ui_and_index(ctx: &Context, index: u16, zoom: f32) -> Self {
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

        let image = ImageReader::open(path).unwrap().decode().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }
}