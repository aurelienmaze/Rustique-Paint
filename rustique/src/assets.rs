use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/images/"]
pub struct Assets;

impl Assets {
    /// Charge une image embarquée et retourne ses données en tant que Vec<u8>
    pub fn get_image_bytes(filename: &str) -> Option<Vec<u8>> {
        Self::get(filename).map(|file| file.data.to_vec())
    }
    
    /// Charge une image embarquée et retourne un image::DynamicImage
    pub fn load_image(filename: &str) -> Option<image::DynamicImage> {
        Self::get_image_bytes(filename)
            .and_then(|bytes| image::load_from_memory(&bytes).ok())
    }
    
    /// Charge une texture pour egui depuis une image embarquée
    pub fn load_texture(ctx: &egui::Context, filename: &str, name: &str) -> Option<egui::TextureHandle> {
        Self::load_image(filename).map(|img| {
            let img_rgba = img.to_rgba8();
            let (width, height) = (img_rgba.width(), img_rgba.height());
            
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                img_rgba.as_flat_samples().as_slice()
            );
            
            ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR)
        })
    }
}
