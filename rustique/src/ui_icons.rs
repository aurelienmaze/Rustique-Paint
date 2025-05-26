use egui::{Color32, RichText, TextureHandle, Vec2, Rect, Pos2, Ui, Response, Sense, Widget};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref ICON_CACHE: Arc<Mutex<HashMap<String, Option<TextureHandle>>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

pub struct IconWidget {
    icon_name: String,
    fallback_emoji: String,
    size: Vec2,
    color: Color32,
}

impl IconWidget {
    pub fn new(icon_name: &str, fallback_emoji: &str, size: Vec2) -> Self {
        Self {
            icon_name: icon_name.to_string(),
            fallback_emoji: fallback_emoji.to_string(),
            size,
            color: Color32::WHITE,
        }
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    fn load_icon_texture(&self, ctx: &egui::Context) -> Option<TextureHandle> {
        let mut cache = ICON_CACHE.lock().unwrap();
        
        if let Some(cached) = cache.get(&self.icon_name) {
            return cached.clone();
        }

        let icon_path = format!("images/{}.png", self.icon_name);
        
        let texture = match image::open(&icon_path) {
            Ok(img) => {
                let img_rgba = img.to_rgba8();
                let (width, height) = (img_rgba.width(), img_rgba.height());
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    img_rgba.as_flat_samples().as_slice()
                );
                
                Some(ctx.load_texture(
                    &self.icon_name,
                    color_image,
                    egui::TextureOptions::LINEAR
                ))
            },
            Err(_) => None,
        };

        cache.insert(self.icon_name.clone(), texture.clone());
        texture
    }
}

impl Widget for IconWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        
        if let Some(texture) = self.load_icon_texture(ui.ctx()) {
            let tint = if self.color == Color32::WHITE {
                Color32::WHITE
            } else {
                self.color
            };
            
            ui.painter().image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                tint
            );
        } else {
            let text_size = self.size.y * 0.8;
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &self.fallback_emoji,
                egui::FontId::proportional(text_size),
                self.color,
            );
        }
        
        response
    }
}

pub struct ToolIcons;

impl ToolIcons {
    pub fn brush() -> IconWidget {
        IconWidget::new("brush_tool", "🖌️", Vec2::new(24.0, 24.0))
    }
    
    pub fn eraser() -> IconWidget {
        IconWidget::new("eraser_tool", "🧹", Vec2::new(24.0, 24.0))
    }
    
    pub fn paint_bucket() -> IconWidget {
        IconWidget::new("bucket_tool", "🪣", Vec2::new(24.0, 24.0))
    }
    
    pub fn color_picker() -> IconWidget {
        IconWidget::new("picker_tool", "🎨", Vec2::new(24.0, 24.0))
    }
    
    pub fn line() -> IconWidget {
        IconWidget::new("line_tool", "📏", Vec2::new(24.0, 24.0))
    }
    
    pub fn undo() -> IconWidget {
        IconWidget::new("undo_icon", "↶", Vec2::new(20.0, 20.0)).with_color(Color32::WHITE)
    }
    
    pub fn redo() -> IconWidget {
        IconWidget::new("redo_icon", "↷", Vec2::new(20.0, 20.0)).with_color(Color32::WHITE)
    }
    
    pub fn save() -> IconWidget {
        IconWidget::new("save_icon", "💾", Vec2::new(20.0, 20.0))
    }
    
    pub fn home() -> IconWidget {
        IconWidget::new("home_icon", "🏠", Vec2::new(20.0, 20.0)).with_color(Color32::WHITE)
    }
    
    pub fn layer_visible() -> IconWidget {
        IconWidget::new("layer_visible", "👁", Vec2::new(18.0, 18.0))
    }
    
    pub fn layer_hidden() -> IconWidget {
        IconWidget::new("layer_hidden", "🚫", Vec2::new(18.0, 18.0))
    }
    
    pub fn add() -> IconWidget {
        IconWidget::new("layer_add", "➕", Vec2::new(18.0, 18.0)).with_color(Color32::WHITE)
    }
    
    pub fn remove() -> IconWidget {
        IconWidget::new("layer_remove", "➖", Vec2::new(18.0, 18.0)).with_color(Color32::WHITE)
    }
    
    pub fn move_up() -> IconWidget {
        IconWidget::new("layer_up", "⬆", Vec2::new(18.0, 18.0)).with_color(Color32::WHITE)
    }
    
    pub fn move_down() -> IconWidget {
        IconWidget::new("layer_down", "⬇", Vec2::new(18.0, 18.0)).with_color(Color32::WHITE)
    }
    
    pub fn edit() -> IconWidget {
        IconWidget::new("layer_edit", "✏️", Vec2::new(16.0, 16.0))
    }

    pub fn brush_text() -> RichText {
        RichText::new("🖌️").size(20.0)
    }
    
    pub fn eraser_text() -> RichText {
        RichText::new("🧹").size(20.0)
    }
    
    pub fn paint_bucket_text() -> RichText {
        RichText::new("🪣").size(20.0)
    }
    
    pub fn color_picker_text() -> RichText {
        RichText::new("🎨").size(20.0)
    }
    
    pub fn line_text() -> RichText {
        RichText::new("📏").size(20.0)
    }
    
    pub fn undo_text() -> RichText {
        RichText::new("↶").size(18.0).color(Color32::WHITE)
    }
    
    pub fn redo_text() -> RichText {
        RichText::new("↷").size(18.0).color(Color32::WHITE)
    }
    
    pub fn save_text() -> RichText {
        RichText::new("💾").size(18.0)
    }
    
    pub fn home_text() -> RichText {
        RichText::new("🏠").size(18.0).color(Color32::WHITE)
    }
    
    pub fn layer_visible_text() -> RichText {
        RichText::new("👁").size(16.0)
    }
    
    pub fn layer_hidden_text() -> RichText {
        RichText::new("🚫").size(16.0)
    }
    
    pub fn add_text() -> RichText {
        RichText::new("➕").size(16.0).color(Color32::WHITE)
    }
    
    pub fn remove_text() -> RichText {
        RichText::new("➖").size(16.0).color(Color32::WHITE)
    }
    
    pub fn move_up_text() -> RichText {
        RichText::new("⬆").size(16.0).color(Color32::WHITE)
    }
    
    pub fn move_down_text() -> RichText {
        RichText::new("⬇").size(16.0).color(Color32::WHITE)
    }
    
    pub fn edit_text() -> RichText {
        RichText::new("✏️").size(14.0)
    }
}
