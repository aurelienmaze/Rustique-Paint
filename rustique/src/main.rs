mod main_menu;
mod localization;
mod brush_system;
mod ui_theme;
mod ui_icons;
mod assets;

use eframe::egui;
use egui::{Color32, TextureHandle, TextureOptions, Rect, Pos2, Vec2, Stroke, RichText};
use image::{ImageBuffer, Rgba, ImageFormat};
use std::collections::VecDeque;
use rfd::FileDialog;
use std::time::Instant;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};

use main_menu::MainMenu;
use localization::{Language, get_text};
use brush_system::BrushManager;
use assets::Assets;
use ui_theme::RustiqueTheme;
use ui_icons::ToolIcons;

const MAX_UNDO_STEPS: usize = 20;
const CHECKERBOARD_SIZE: usize = 8;
const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 800.0;
const MAX_SAVED_COLORS: usize = 16;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
enum Tool {
    Brush,
    Eraser,
    PaintBucket,
    ColorPicker,
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    Gif,
    WebP,
    Rustiq,
    Unknown,
}

impl FileFormat {
    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => FileFormat::Png,
            "jpg" | "jpeg" => FileFormat::Jpeg,
            "bmp" => FileFormat::Bmp,
            "tiff" | "tif" => FileFormat::Tiff,
            "gif" => FileFormat::Gif,
            "webp" => FileFormat::WebP,
            "rustiq" => FileFormat::Rustiq,
            _ => FileFormat::Unknown,
        }
    }
    
    fn get_image_format(&self) -> Option<ImageFormat> {
        match self {
            FileFormat::Png => Some(ImageFormat::Png),
            FileFormat::Jpeg => Some(ImageFormat::Jpeg),
            FileFormat::Bmp => Some(ImageFormat::Bmp),
            FileFormat::Tiff => Some(ImageFormat::Tiff),
            FileFormat::Gif => Some(ImageFormat::Gif),
            FileFormat::WebP => Some(ImageFormat::WebP),
            _ => None,
        }
    }
}

enum AppState {
    MainMenu(MainMenu),
    Canvas(PaintApp),
}

#[derive(Clone, PartialEq)]
struct Layer {
    name: String,
    data: Vec<Option<Color32>>,
    visible: bool,
}

#[derive(Serialize, Deserialize)]
struct LayerData {
    name: String,
    data: Vec<Option<[u8; 4]>>,
    visible: bool,
}

#[derive(Serialize, Deserialize)]
struct RustiqueFile {
    width: usize,
    height: usize,
    layers: Vec<LayerData>,
    active_layer_index: usize,
    primary_color: [u8; 4],
    secondary_color: [u8; 4],
    saved_colors: Vec<[u8; 4]>,
    brush_size: i32,
    eraser_size: i32,
}

#[derive(Clone)]
struct CanvasState {
    width: usize,
    height: usize,
    layers: Vec<Layer>,
    active_layer_index: usize,
}

impl CanvasState {
    fn new(width: usize, height: usize) -> Self {
        let default_layer = Layer {
            name: "Background".to_string(),
            data: vec![None; width * height],
            visible: true,
        };
        
        Self {
            width,
            height,
            layers: vec![default_layer],
            active_layer_index: 0,
        }
    }
    
    #[inline]
    fn get(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height {
            for layer_index in (0..self.layers.len()).rev() {
                let layer = &self.layers[layer_index];
                if layer.visible {
                    let idx = y * self.width + x;
                    if let Some(color) = layer.data[idx] {
                        return Some(color);
                    }
                }
            }
        }
        None
    }
    
    #[inline]
    fn get_from_active_layer(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height && self.active_layer_index < self.layers.len() {
            let idx = y * self.width + x;
            self.layers[self.active_layer_index].data[idx]
        } else {
            None
        }
    }
    
    #[inline]
    fn set(&mut self, x: usize, y: usize, color: Option<Color32>) {
        if x < self.width && y < self.height && self.active_layer_index < self.layers.len() {
            let idx = y * self.width + x;
            self.layers[self.active_layer_index].data[idx] = color;
        }
    }
}

#[derive(Clone)]
struct CanvasChange {
    x: usize,
    y: usize,
    layer_index: usize,
    old_color: Option<Color32>,
    new_color: Option<Color32>,
}

enum SaveDialog {
    Hidden,
    AskingSave {
        return_to_menu: bool,
    },
}

struct PaintApp {
    current_state: CanvasState,
    undo_stack: Vec<Vec<CanvasChange>>,
    redo_stack: Vec<Vec<CanvasChange>>,
    current_changes: Vec<CanvasChange>,
    current_tool: Tool,
    primary_color: Color32,
    secondary_color: Color32,
    using_secondary_color: bool,
    saved_colors: Vec<Color32>,
    brush_size: i32,
    eraser_size: i32,
    brush_manager: BrushManager,
    last_position: Option<(i32, i32)>,
    is_drawing: bool,
    last_action_time: Instant,
    texture: Option<TextureHandle>,
    texture_dirty: bool,
    zoom: f32,
    pan: Vec2,
    line_start: Option<(i32, i32)>,
    line_end: Option<(i32, i32)>,
    is_drawing_line: bool,
    is_first_click_line: bool,
    has_unsaved_changes: bool,
    last_save_path: Option<String>,
    save_dialog: SaveDialog,
    language: Language,
    current_pressure: f32,
    pressure_smoothing: f32,
    pressure_enabled: bool,
    last_cursor_pos: Option<Pos2>,
    last_cursor_time: Option<f64>,
    velocity_sensitivity: f32,
    max_velocity_for_min_pressure: f32,
}

impl PaintApp {
    fn new(width: u32, height: u32, language: Language) -> Self {
        let initial_state = CanvasState::new(width as usize, height as usize);
        Self {
            current_state: initial_state,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_changes: Vec::new(),
            current_tool: Tool::Brush,
            primary_color: Color32::BLACK,
            secondary_color: Color32::WHITE,
            using_secondary_color: false,
            saved_colors: Vec::new(),
            brush_size: 3,
            eraser_size: 3,
            brush_manager: BrushManager::new(),
            last_position: None,
            is_drawing: false,
            last_action_time: Instant::now(),
            texture: None,
            texture_dirty: true,
            zoom: 1.0,
            pan: Vec2::ZERO,
            line_start: None,
            line_end: None,
            is_drawing_line: false,
            is_first_click_line: true,
            has_unsaved_changes: false,
            last_save_path: None,
            save_dialog: SaveDialog::Hidden,
            language,
            current_pressure: 1.0,
            pressure_smoothing: 0.8,
            pressure_enabled: false,
            last_cursor_pos: None,
            last_cursor_time: None,
            velocity_sensitivity: 0.9,
            max_velocity_for_min_pressure: 1900.0,
        }
    }

    fn from_rustiq_file(file: RustiqueFile, language: Language) -> Self {
        let mut canvas = CanvasState {
            width: file.width,
            height: file.height,
            layers: Vec::with_capacity(file.layers.len()),
            active_layer_index: file.active_layer_index,
        };
        
        for layer_data in file.layers {
            let mut layer = Layer {
                name: layer_data.name,
                data: Vec::with_capacity(layer_data.data.len()),
                visible: layer_data.visible,
            };
            
            for pixel_opt in layer_data.data {
                if let Some(rgba) = pixel_opt {
                    layer.data.push(Some(Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])));
                } else {
                    layer.data.push(None);
                }
            }
            
            canvas.layers.push(layer);
        }
        
        let primary_color = Color32::from_rgba_unmultiplied(
            file.primary_color[0],
            file.primary_color[1],
            file.primary_color[2],
            file.primary_color[3]
        );
        
        let secondary_color = Color32::from_rgba_unmultiplied(
            file.secondary_color[0],
            file.secondary_color[1],
            file.secondary_color[2],
            file.secondary_color[3]
        );
        
        let mut saved_colors = Vec::with_capacity(file.saved_colors.len());
        for color_data in file.saved_colors {
            saved_colors.push(Color32::from_rgba_unmultiplied(
                color_data[0],
                color_data[1],
                color_data[2],
                color_data[3]
            ));
        }
        
        Self {
            current_state: canvas,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_changes: Vec::new(),
            current_tool: Tool::Brush,
            primary_color,
            secondary_color,
            using_secondary_color: false,
            saved_colors,
            brush_size: file.brush_size,
            eraser_size: file.eraser_size,
            brush_manager: BrushManager::new(),
            last_position: None,
            is_drawing: false,
            last_action_time: Instant::now(),
            texture: None,
            texture_dirty: true,
            zoom: 1.0,
            pan: Vec2::ZERO,
            line_start: None,
            line_end: None,
            is_drawing_line: false,
            is_first_click_line: true,
            has_unsaved_changes: false,
            last_save_path: None,
            save_dialog: SaveDialog::Hidden,
            language,
            current_pressure: 1.0,
            pressure_smoothing: 0.8,
            pressure_enabled: false,
            last_cursor_pos: None,
            last_cursor_time: None,
            velocity_sensitivity: 0.9,
            max_velocity_for_min_pressure: 1900.0,
        }
    }

    fn detect_format(path: &str) -> FileFormat {
        PathBuf::from(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(FileFormat::from_extension)
            .unwrap_or(FileFormat::Unknown)
    }
    
    
    fn update_pressure_from_velocity(&mut self, cursor_pos: Pos2, current_time: f64) {
        if !self.pressure_enabled {
            self.current_pressure = 1.0;
            return;
        }
        
        let pressure = if let (Some(last_pos), Some(last_time)) = (self.last_cursor_pos, self.last_cursor_time) {
            let delta_time = (current_time - last_time).max(0.001);
            let distance = (cursor_pos - last_pos).length();
            let velocity = distance / delta_time as f32;
            
            let normalized_velocity = (velocity / self.max_velocity_for_min_pressure).min(1.0);
            let base_pressure = 1.0 - (normalized_velocity * self.velocity_sensitivity);
            
            base_pressure.clamp(0.1, 1.0)
        } else {
            1.0
        };
        
        let smoothing_factor = self.pressure_smoothing;
        self.current_pressure = self.current_pressure * (1.0 - smoothing_factor) + pressure * smoothing_factor;
        
        self.last_cursor_pos = Some(cursor_pos);
        self.last_cursor_time = Some(current_time);
    }
    
    fn get_effective_pressure(&self) -> f32 {
        if self.pressure_enabled {
            self.current_pressure
        } else {
            1.0
        }
    }
    
    fn save_file(&mut self, path: &str) -> Result<(), String> {
        let format = Self::detect_format(path);
        
        match format {
            FileFormat::Rustiq => self.save_as_rustiq(path),
            FileFormat::Unknown => {
                Err(format!("{}: {}", get_text("format_not_supported", self.language), path))
            },
            _ => {
                if let Some(image_format) = format.get_image_format() {
                    self.save_as_image(path, image_format)
                } else {
                    Err(format!("{}: {}", get_text("format_not_supported", self.language), path))
                }
            }
        }
    }
    
    fn save_as_image(&mut self, path: &str, format: ImageFormat) -> Result<(), String> {
        let width = self.current_state.width;
        let height = self.current_state.height;
        let mut img = ImageBuffer::new(width as u32, height as u32);
        
        for y in 0..height {
            for x in 0..width {
                let color = self.current_state.get(x, y).unwrap_or(Color32::TRANSPARENT);
                img.put_pixel(x as u32, y as u32, Rgba([color.r(), color.g(), color.b(), color.a()]));
            }
        }

        match img.save_with_format(path, format) {
            Ok(_) => {
                self.has_unsaved_changes = false;
                self.last_save_path = Some(path.to_string());
                Ok(())
            },
            Err(e) => Err(format!("{}: {}", get_text("error_saving_image", self.language), e)),
        }
    }
    
    fn open_file(path: &str, language: Language) -> Result<Self, String> {
        let format = Self::detect_format(path);
        
        match format {
            FileFormat::Rustiq => {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        match serde_json::from_str::<RustiqueFile>(&content) {
                            Ok(file) => {
                                let mut app = Self::from_rustiq_file(file, language);
                                app.last_save_path = Some(path.to_string());
                                Ok(app)
                            },
                            Err(e) => Err(format!("{}: {}", get_text("error_reading_rustiq", language), e))
                        }
                    },
                    Err(e) => Err(format!("{}: {}", get_text("error_reading_file", language), e))
                }
            },
            FileFormat::Unknown => {
                Err(format!("{}: {}", get_text("format_not_supported", language), path))
            },
            _ => {
                match image::open(path) {
                    Ok(img) => {
                        let width = img.width() as usize;
                        let height = img.height() as usize;
                        let mut canvas = CanvasState::new(width, height);
                        
                        let rgba_img = img.to_rgba8();
                        for y in 0..height {
                            for x in 0..width {
                                let pixel = rgba_img.get_pixel(x as u32, y as u32);
                                if pixel[3] > 0 {
                                    let color = Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]);
                                    canvas.set(x, y, Some(color));
                                }
                            }
                        }
                        
                        let app = Self {
                            current_state: canvas,
                            undo_stack: Vec::new(),
                            redo_stack: Vec::new(),
                            current_changes: Vec::new(),
                            current_tool: Tool::Brush,
                            primary_color: Color32::BLACK,
                            secondary_color: Color32::WHITE,
                            using_secondary_color: false,
                            saved_colors: Vec::new(),
                            brush_size: 3,
                            eraser_size: 3,
                            brush_manager: BrushManager::new(),
                            last_position: None,
                            is_drawing: false,
                            last_action_time: Instant::now(),
                            texture: None,
                            texture_dirty: true,
                            zoom: 1.0,
                            pan: Vec2::ZERO,
                            line_start: None,
                            line_end: None,
                            is_drawing_line: false,
                            is_first_click_line: true,
                            has_unsaved_changes: false,
                            last_save_path: Some(path.to_string()),
                            save_dialog: SaveDialog::Hidden,
                            language,
                            current_pressure: 1.0,
                            pressure_smoothing: 0.3,
                            pressure_enabled: true,
                            last_cursor_pos: None,
                            last_cursor_time: None,
                            velocity_sensitivity: 0.8,
                            max_velocity_for_min_pressure: 800.0,
                        };
                        
                        Ok(app)
                    },
                    Err(e) => Err(format!("{}: {}", get_text("unable_to_open_image", language), e))
                }
            }
        }
    }
    
    fn save_as_rustiq(&mut self, path: &str) -> Result<(), String> {
        let mut layers = Vec::with_capacity(self.current_state.layers.len());
        
        for layer in &self.current_state.layers {
            let mut layer_data = Vec::with_capacity(layer.data.len());
            
            for &pixel_opt in &layer.data {
                match pixel_opt {
                    Some(color) => {
                        layer_data.push(Some([color.r(), color.g(), color.b(), color.a()]));
                    },
                    None => {
                        layer_data.push(None);
                    }
                }
            }
            
            layers.push(LayerData {
                name: layer.name.clone(),
                data: layer_data,
                visible: layer.visible,
            });
        }
        
        let mut saved_colors = Vec::with_capacity(self.saved_colors.len());
        for &color in &self.saved_colors {
            saved_colors.push([color.r(), color.g(), color.b(), color.a()]);
        }
        
        let rustiq_file = RustiqueFile {
            width: self.current_state.width,
            height: self.current_state.height,
            layers,
            active_layer_index: self.current_state.active_layer_index,
            primary_color: [self.primary_color.r(), self.primary_color.g(), self.primary_color.b(), self.primary_color.a()],
            secondary_color: [self.secondary_color.r(), self.secondary_color.g(), self.secondary_color.b(), self.secondary_color.a()],
            saved_colors,
            brush_size: self.brush_size,
            eraser_size: self.eraser_size,
        };
        
        let json = match serde_json::to_string(&rustiq_file) {
            Ok(json) => json,
            Err(e) => return Err(format!("Erreur de sérialisation: {}", e)),
        };
        
        match fs::File::create(path) {
            Ok(mut file) => {
                match file.write_all(json.as_bytes()) {
                    Ok(_) => {
                        self.has_unsaved_changes = false;
                        self.last_save_path = Some(path.to_string());
                        Ok(())
                    },
                    Err(e) => Err(format!("Erreur d'écriture: {}", e)),
                }
            },
            Err(e) => Err(format!("Erreur de création du fichier: {}", e)),
        }
    }
    
    fn quick_save(&mut self) -> Result<(), String> {
        if let Some(path) = &self.last_save_path {
            let path_clone = path.clone();
            self.save_file(&path_clone)
        } else {
            Err(get_text("no_previous_path", self.language))
        }
    }

    fn add_layer(&mut self, name: String) {
        self.current_state.layers.push(Layer {
            name,
            data: vec![None; self.current_state.width * self.current_state.height],
            visible: true,
        });
        self.current_state.active_layer_index = self.current_state.layers.len() - 1;
        self.texture_dirty = true;
        self.has_unsaved_changes = true;
    }
    
    fn remove_layer(&mut self, index: usize) {
        if self.current_state.layers.len() > 1 && index < self.current_state.layers.len() {
            self.current_state.layers.remove(index);
            if self.current_state.active_layer_index >= self.current_state.layers.len() {
                self.current_state.active_layer_index = self.current_state.layers.len() - 1;
            }
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn move_layer_up(&mut self, index: usize) {
        if index > 0 && index < self.current_state.layers.len() {
            self.current_state.layers.swap(index, index - 1);
            if self.current_state.active_layer_index == index {
                self.current_state.active_layer_index -= 1;
            } else if self.current_state.active_layer_index == index - 1 {
                self.current_state.active_layer_index += 1;
            }
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn move_layer_down(&mut self, index: usize) {
        if index < self.current_state.layers.len() - 1 {
            self.current_state.layers.swap(index, index + 1);
            if self.current_state.active_layer_index == index {
                self.current_state.active_layer_index += 1;
            } else if self.current_state.active_layer_index == index + 1 {
                self.current_state.active_layer_index -= 1;
            }
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn toggle_layer_visibility(&mut self, index: usize) {
        if index < self.current_state.layers.len() {
            self.current_state.layers[index].visible = !self.current_state.layers[index].visible;
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }
    
    fn set_active_layer(&mut self, index: usize) {
        if index < self.current_state.layers.len() {
            self.current_state.active_layer_index = index;
        }
    }
    
    fn rename_layer(&mut self, index: usize, name: String) {
        if index < self.current_state.layers.len() {
            self.current_state.layers[index].name = name;
            self.has_unsaved_changes = true;
        }
    }
    
    fn add_saved_color(&mut self, color: Color32) {
        if !self.saved_colors.contains(&color) {
            if self.saved_colors.len() >= MAX_SAVED_COLORS {
                self.saved_colors.remove(0);
            }
            self.saved_colors.push(color);
        }
    }
    
    fn remove_saved_color(&mut self, index: usize) {
        if index < self.saved_colors.len() {
            self.saved_colors.remove(index);
        }
    }
    
    fn record_change(&mut self, x: usize, y: usize, new_color: Option<Color32>) {
        if x < self.current_state.width && y < self.current_state.height {
            let old_color = self.current_state.get_from_active_layer(x, y);
            if old_color != new_color {
                self.current_changes.push(CanvasChange {
                    x, 
                    y, 
                    layer_index: self.current_state.active_layer_index,
                    old_color, 
                    new_color
                });
                self.current_state.set(x, y, new_color);
                self.has_unsaved_changes = true;
            }
        }
    }

    fn save_state(&mut self) {
        if !self.current_changes.is_empty() {
            self.undo_stack.push(std::mem::take(&mut self.current_changes));
            self.current_changes = Vec::new();
            if self.undo_stack.len() > MAX_UNDO_STEPS {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.is_drawing = false;
            self.has_unsaved_changes = true;
        }
    }

    fn undo(&mut self) {
        if let Some(changes) = self.undo_stack.pop() {
            let mut redo_changes = Vec::with_capacity(changes.len());
            
            for change in changes.iter().rev() {
                let layer_index_backup = self.current_state.active_layer_index;
                self.current_state.active_layer_index = change.layer_index;
                
                redo_changes.push(CanvasChange {
                    x: change.x,
                    y: change.y,
                    layer_index: change.layer_index,
                    old_color: change.old_color,
                    new_color: change.new_color,
                });
                
                self.current_state.set(change.x, change.y, change.old_color);
                self.current_state.active_layer_index = layer_index_backup;
            }
            
            self.redo_stack.push(redo_changes);
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }

    fn redo(&mut self) {
        if let Some(changes) = self.redo_stack.pop() {
            let mut undo_changes = Vec::with_capacity(changes.len());
            
            for change in changes.iter().rev() {
                let layer_index_backup = self.current_state.active_layer_index;
                self.current_state.active_layer_index = change.layer_index;
                
                let current_color = self.current_state.get_from_active_layer(change.x, change.y);
                undo_changes.push(CanvasChange {
                    x: change.x,
                    y: change.y,
                    layer_index: change.layer_index,
                    old_color: current_color,
                    new_color: change.new_color,
                });
                
                self.current_state.set(change.x, change.y, change.new_color);
                self.current_state.active_layer_index = layer_index_backup;
            }
            
            self.undo_stack.push(undo_changes);
            self.texture_dirty = true;
            self.has_unsaved_changes = true;
        }
    }

    fn draw_line(&mut self, start: (i32, i32), end: (i32, i32), _color: Color32) {
        let color = if self.using_secondary_color { self.secondary_color } else { self.primary_color };
        
        let current_size = if self.current_tool == Tool::Eraser {
            self.eraser_size as f32
        } else {
            self.brush_size as f32
        };
        
        if self.brush_manager.current_size != current_size {
            self.brush_manager.current_size = current_size;
        }
        
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        let pressure = self.get_effective_pressure();
        let is_eraser = self.current_tool == Tool::Eraser;
        
        let mut changes = Vec::new();
        {
            let mut record_change = |x: usize, y: usize, new_color: Option<Color32>| {
                if is_eraser {
                    changes.push((x, y, None));
                } else {
                    changes.push((x, y, new_color));
                }
            };
            
            self.brush_manager.draw_line(start, end, color, pressure, &mut record_change);
        }
        
        for (x, y, color) in changes {
            self.record_change(x, y, color);
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }
    
    fn draw_smooth_line(&mut self, start: (i32, i32), end: (i32, i32), _color: Color32) {
        if self.brush_manager.current_size != self.brush_size as f32 {
            self.brush_manager.current_size = self.brush_size as f32;
        }
        
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        let (x0, y0) = start;
        let (x1, y1) = end;
        
        let dx = (x1 - x0) as f32;
        let dy = (y1 - y0) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance < 1.0 {
            self.draw_point(x0, y0, false);
            return;
        }
        
        let active_brush = self.brush_manager.active_brush();
        let base_spacing = active_brush.spacing * self.brush_manager.current_size;
        
        let spacing = match active_brush.brush_type {
            brush_system::BrushType::Flat | brush_system::BrushType::Bright => {
                (base_spacing * 0.2).max(0.2)
            },
            brush_system::BrushType::Rigger => {
                (base_spacing * 0.1).max(0.1)
            },
            brush_system::BrushType::Fan => {
                (base_spacing * 0.6).max(0.6)
            },
            brush_system::BrushType::Mop => {
                (base_spacing * 0.3).max(0.3)
            },
            _ => {
                (base_spacing * 0.3).max(0.3)
            }
        };
        
        let num_points = (distance / spacing).ceil() as i32;
        
        for i in 0..=num_points {
            let t = if num_points > 0 { i as f32 / num_points as f32 } else { 0.0 };
            
            let x = x0 as f32 + dx * t;
            let y = y0 as f32 + dy * t;
            
            self.draw_smooth_point(x, y, false);
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }
    
    fn draw_smooth_point(&mut self, x: f32, y: f32, use_secondary: bool) {
        let color = if use_secondary { self.secondary_color } else { self.primary_color };
        
        let current_size = if self.current_tool == Tool::Eraser {
            self.eraser_size as f32
        } else {
            self.brush_size as f32
        };
        
        if self.brush_manager.current_size != current_size {
            self.brush_manager.current_size = current_size;
        }
        
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;
        
        let wx = x - x.floor();
        let wy = y - y.floor();
        
        let weights = [
            (1.0 - wx) * (1.0 - wy),
            wx * (1.0 - wy),
            (1.0 - wx) * wy,
            wx * wy,
        ];
        
        let positions = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
        
        for (i, &(px, py)) in positions.iter().enumerate() {
            if weights[i] > 0.0 {
                let weighted_color = Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(), 
                    color.b(),
                    ((color.a() as f32) * weights[i]) as u8
                );
                self.draw_weighted_point(px, py, weighted_color);
            }
        }
    }
    
    fn draw_weighted_point(&mut self, x: i32, y: i32, weighted_color: Color32) {
        let current_size = if self.current_tool == Tool::Eraser {
            self.eraser_size as f32
        } else {
            self.brush_size as f32
        };
        
        if self.brush_manager.current_size != current_size {
            self.brush_manager.current_size = current_size;
        }
        
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        self.brush_manager.update_angle(x as f32, y as f32);
        
        let base_size = self.brush_manager.current_size;
        let active_brush = self.brush_manager.active_brush();
        
        let mask_size = match active_brush.brush_type {
            brush_system::BrushType::Flat | brush_system::BrushType::Bright => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
            brush_system::BrushType::Fan | brush_system::BrushType::Angle => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
            brush_system::BrushType::Rigger => {
                ((base_size * 1.5).max(5.0) as usize) | 1
            },
            brush_system::BrushType::Mop => {
                ((base_size * 2.5).max(9.0) as usize) | 1
            },
            brush_system::BrushType::Round => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
            brush_system::BrushType::Filbert => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
        };
        
        let mask = self.brush_manager.generate_brush_mask(mask_size);
        
        let center = mask_size as i32 / 2;
        let width = self.current_state.width as i32;
        let height = self.current_state.height as i32;
        
        for dy in 0..mask_size as i32 {
            for dx in 0..mask_size as i32 {
                let nx = x + dx - center;
                let ny = y + dy - center;
                
                if nx >= 0 && nx < width && ny >= 0 && ny < height {
                    let mask_value = mask[(dy as usize) * mask_size + (dx as usize)];
                    
                    if mask_value > 0.0 {
                        let final_alpha = ((weighted_color.a() as f32) * mask_value) as u8;
                        let new_color = if self.current_tool == Tool::Eraser {
                            None
                        } else if final_alpha > 0 {
                            Some(Color32::from_rgba_unmultiplied(
                                weighted_color.r(), 
                                weighted_color.g(), 
                                weighted_color.b(), 
                                final_alpha
                            ))
                        } else {
                            None
                        };
                        
                        self.record_change(nx as usize, ny as usize, new_color);
                    }
                }
            }
        }
        
        self.texture_dirty = true;
    }

    fn draw_point(&mut self, x: i32, y: i32, _use_secondary: bool) {
        let color = if self.using_secondary_color { self.secondary_color } else { self.primary_color };
        
        let current_size = if self.current_tool == Tool::Eraser {
            self.eraser_size as f32
        } else {
            self.brush_size as f32
        };
        
        if self.brush_manager.current_size != current_size {
            self.brush_manager.current_size = current_size;
        }
        
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        let pressure = self.get_effective_pressure();
        let is_eraser = self.current_tool == Tool::Eraser;
        
        let mut changes = Vec::new();
        {
            let mut record_change = |x: usize, y: usize, new_color: Option<Color32>| {
                if is_eraser {
                    changes.push((x, y, None));
                } else {
                    changes.push((x, y, new_color));
                }
            };
            
            self.brush_manager.draw_point(x, y, color, pressure, &mut record_change);
        }
        
        for (x, y, color) in changes {
            self.record_change(x, y, color);
        }
        
        self.texture_dirty = true;
    }
    
    fn draw_point_with_color(&mut self, x: i32, y: i32, fill_color: Option<Color32>) {
        let color = fill_color.unwrap_or(Color32::TRANSPARENT);
        
        let current_size = if self.current_tool == Tool::Eraser {
            self.eraser_size as f32
        } else {
            self.brush_size as f32
        };
        
        if self.brush_manager.current_size != current_size {
            self.brush_manager.current_size = current_size;
        }
        
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        self.brush_manager.update_angle(x as f32, y as f32);
        
        let base_size = self.brush_manager.current_size;
        let active_brush = self.brush_manager.active_brush();
        
        let mask_size = match active_brush.brush_type {
            brush_system::BrushType::Flat | brush_system::BrushType::Bright => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
            brush_system::BrushType::Fan | brush_system::BrushType::Angle => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
            brush_system::BrushType::Rigger => {
                ((base_size * 1.5).max(5.0) as usize) | 1
            },
            brush_system::BrushType::Mop => {
                ((base_size * 2.5).max(9.0) as usize) | 1
            },
            brush_system::BrushType::Round => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
            brush_system::BrushType::Filbert => {
                ((base_size * 2.0).max(7.0) as usize) | 1
            },
        };
        
        let mask = self.brush_manager.generate_brush_mask(mask_size);
        
        let center = mask_size as i32 / 2;
        let width = self.current_state.width as i32;
        let height = self.current_state.height as i32;
        
        for dy in 0..mask_size as i32 {
            for dx in 0..mask_size as i32 {
                let nx = x + dx - center;
                let ny = y + dy - center;
                
                if nx >= 0 && nx < width && ny >= 0 && ny < height {
                    let mask_value = mask[(dy as usize) * mask_size + (dx as usize)];
                    
                    if mask_value > 0.0 {
                        let final_color = if self.current_tool == Tool::Eraser || fill_color.is_none() {
                            None
                        } else {
                            let alpha = (color.a() as f32 * mask_value) as u8;
                            if alpha > 0 {
                                Some(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha))
                            } else {
                                None
                            }
                        };
                        
                        self.record_change(nx as usize, ny as usize, final_color);
                    }
                }
            }
        }
        
        self.texture_dirty = true;
    }

    fn paint_bucket(&mut self, x: usize, y: usize, _use_secondary: bool) {
        if x >= self.current_state.width || y >= self.current_state.height {
            return;
        }
        
        if self.current_state.active_layer_index < self.current_state.layers.len() &&
           !self.current_state.layers[self.current_state.active_layer_index].visible {
            return;
        }
        
        let target_color = self.current_state.get_from_active_layer(x, y);
        let color = if self.using_secondary_color { self.secondary_color } else { self.primary_color };
        let fill_color = if self.current_tool == Tool::Eraser {
            None
        } else {
            Some(color)
        };
        
        if target_color == fill_color {
            return;
        }
        
        let mut queue = VecDeque::with_capacity(1024);
        let mut visited = vec![false; self.current_state.width * self.current_state.height];
        queue.push_back((x, y));
        
        while let Some((cx, cy)) = queue.pop_front() {
            let idx = cy * self.current_state.width + cx;
            if visited[idx] || self.current_state.get_from_active_layer(cx, cy) != target_color {
                continue;
            }
            
            visited[idx] = true;
            self.record_change(cx, cy, fill_color);
            
            if cx > 0 { queue.push_back((cx - 1, cy)); }
            if cx + 1 < self.current_state.width { queue.push_back((cx + 1, cy)); }
            if cy > 0 { queue.push_back((cx, cy - 1)); }
            if cy + 1 < self.current_state.height { queue.push_back((cx, cy + 1)); }
        }
        
        self.last_action_time = Instant::now();
        self.texture_dirty = true;
    }

    fn pick_color(&mut self, x: usize, y: usize, _use_secondary: bool) {
        if let Some(color) = self.current_state.get(x, y) {
            if self.using_secondary_color {
                self.secondary_color = color;
            } else {
                self.primary_color = color;
            }
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        if self.texture_dirty {
            let width = self.current_state.width;
            let height = self.current_state.height;
            
            let mut image_data = vec![0_u8; width * height * 4];
            
            for y in 0..height {
                for x in 0..width {
                    let color = if let Some(pixel) = self.current_state.get(x, y) {
                        pixel
                    } else {
                        let checker_x = x / CHECKERBOARD_SIZE;
                        let checker_y = y / CHECKERBOARD_SIZE;
                        if (checker_x + checker_y) % 2 == 0 {
                            Color32::from_gray(200)
                        } else {
                            Color32::from_gray(160)
                        }
                    };
                    
                    let idx = (y * width + x) * 4;
                    image_data[idx] = color.r();
                    image_data[idx + 1] = color.g();
                    image_data[idx + 2] = color.b();
                    image_data[idx + 3] = color.a();
                }
            }
            
            let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &image_data);
            self.texture = Some(ctx.load_texture("canvas", color_image, TextureOptions::NEAREST));
            
            self.texture_dirty = false;
        }
    }
    
    fn show_save_dialog(&mut self, return_to_menu: bool) {
        self.save_dialog = SaveDialog::AskingSave { 
            return_to_menu
        };
    }
    
    fn set_language(&mut self, language: Language) {
        self.language = language;
    }
}

enum PendingAction {
    None,
    ReturnToMenu,
    HandleLayerAction(LayerAction),
    UndoAction,
    RedoAction,
}

enum LayerAction {
    ToggleVisibility(usize),
    SetActive(usize),
    Edit(usize),
}

struct MyApp {
    state: AppState,
    error_message: Option<String>,
    show_error: bool,
    rename_layer_index: Option<usize>,
    rename_layer_name: String,
    pending_action: PendingAction,
    language: Language,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::MainMenu(MainMenu::new(Language::French)),
            error_message: None,
            show_error: false,
            rename_layer_index: None,
            rename_layer_name: String::new(),
            pending_action: PendingAction::None,
            language: Language::French,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        RustiqueTheme::apply_theme(ctx);
        
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);
        
        if self.show_error {
            egui::Window::new(get_text("error", self.language))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(self.error_message.as_deref().unwrap_or(&get_text("an_error_occurred", self.language)));
                    if ui.button("OK").clicked() {
                        self.show_error = false;
                    }
                });
        }
        
        if let Some(layer_idx) = self.rename_layer_index {
            egui::Window::new(get_text("rename_layer", self.language))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.text_edit_singleline(&mut self.rename_layer_name);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() && !self.rename_layer_name.is_empty() {
                            if let AppState::Canvas(paint_app) = &mut self.state {
                                paint_app.rename_layer(layer_idx, self.rename_layer_name.clone());
                            }
                            self.rename_layer_index = None;
                        }
                        if ui.button(get_text("cancel", self.language)).clicked() {
                            self.rename_layer_index = None;
                        }
                    });
                });
        }
        
        match &self.pending_action {
            PendingAction::ReturnToMenu => {
                self.state = AppState::MainMenu(MainMenu::new(self.language));
                self.pending_action = PendingAction::None;
            },
            PendingAction::HandleLayerAction(action) => {
                if let AppState::Canvas(paint_app) = &mut self.state {
                    match action {
                        LayerAction::ToggleVisibility(idx) => {
                            paint_app.toggle_layer_visibility(*idx);
                        },
                        LayerAction::SetActive(idx) => {
                            paint_app.set_active_layer(*idx);
                        },
                        LayerAction::Edit(idx) => {
                            if let Some(layer) = paint_app.current_state.layers.get(*idx) {
                                self.rename_layer_index = Some(*idx);
                                self.rename_layer_name = layer.name.clone();
                            }
                        }
                    }
                }
                self.pending_action = PendingAction::None;
            },
            PendingAction::UndoAction => {
                if let AppState::Canvas(paint_app) = &mut self.state {
                    paint_app.undo();
                }
                self.pending_action = PendingAction::None;
            },
            PendingAction::RedoAction => {
                if let AppState::Canvas(paint_app) = &mut self.state {
                    paint_app.redo();
                }
                self.pending_action = PendingAction::None;
            },
            PendingAction::None => {}
        }
        
        match &mut self.state {
            AppState::MainMenu(menu) => {
                if let Some(result) = menu.show(ctx) {
                    match result {
                        main_menu::MenuResult::Action(action) => {
                            match action {
                                main_menu::MenuAction::NewCanvas(width, height) => {
                                    self.state = AppState::Canvas(PaintApp::new(width, height, self.language));
                                },
                                main_menu::MenuAction::OpenFile => {
                                    if let Some(path) = FileDialog::new()
                                        .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                        .add_filter("PNG Image", &["png"])
                                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                                        .add_filter("BMP Image", &["bmp"])
                                        .add_filter("TIFF Image", &["tiff", "tif"])
                                        .add_filter("GIF Image", &["gif"])
                                        .add_filter("WebP Image", &["webp"])
                                        .add_filter("Rustique File", &["rustiq"])
                                        .set_directory("/")
                                        .pick_file() {
                                        match PaintApp::open_file(path.to_str().unwrap(), self.language) {
                                            Ok(app) => self.state = AppState::Canvas(app),
                                            Err(e) => {
                                                self.error_message = Some(e);
                                                self.show_error = true;
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        main_menu::MenuResult::LanguageChanged(language) => {
                            self.language = language;
                            if let AppState::Canvas(paint_app) = &mut self.state {
                                paint_app.set_language(language);
                            }
                            self.state = AppState::MainMenu(MainMenu::new(language));
                        }
                    }
                }
            }
            AppState::Canvas(paint_app) => {
                if ctrl {
                    if ctx.input(|i| i.key_pressed(egui::Key::Z)) && !shift {
                        self.pending_action = PendingAction::UndoAction;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::Y)) || 
                        shift && ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                        self.pending_action = PendingAction::RedoAction;
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                        if let Some(_) = &paint_app.last_save_path {
                            match paint_app.quick_save() {
                                Ok(_) => {},
                                Err(e) => {
                                    self.error_message = Some(e);
                                    self.show_error = true;
                                }
                            }
                        } else {
                            if let Some(path) = FileDialog::new()
                                .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                .add_filter("PNG Image", &["png"])
                                .add_filter("JPEG Image", &["jpg", "jpeg"])
                                .add_filter("BMP Image", &["bmp"])
                                .add_filter("TIFF Image", &["tiff", "tif"])
                                .add_filter("GIF Image", &["gif"])
                                .add_filter("WebP Image", &["webp"])
                                .add_filter("Rustique File", &["rustiq"])
                                .set_directory("/")
                                .save_file() {
                                match paint_app.save_file(path.to_str().unwrap()) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        self.error_message = Some(e);
                                        self.show_error = true;
                                    }
                                }
                            }
                        }
                    }
                }
                
                match &mut paint_app.save_dialog {
                    SaveDialog::Hidden => {},
                    SaveDialog::AskingSave { return_to_menu } => {
                        let return_to_menu_val = *return_to_menu;
                        egui::Window::new(get_text("save_changes", self.language))
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.label(get_text("want_to_save_changes", self.language));
                                ui.horizontal(|ui| {
                                    if ui.button(get_text("yes", self.language)).clicked() {
                                        let result = if let Some(path) = FileDialog::new()
                                            .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                            .add_filter("PNG Image", &["png"])
                                            .add_filter("JPEG Image", &["jpg", "jpeg"])
                                            .add_filter("BMP Image", &["bmp"])
                                            .add_filter("TIFF Image", &["tiff", "tif"])
                                            .add_filter("GIF Image", &["gif"])
                                            .add_filter("WebP Image", &["webp"])
                                            .add_filter("Rustique File", &["rustiq"])
                                            .set_directory("/")
                                            .save_file() {
                                            paint_app.save_file(path.to_str().unwrap())
                                        } else {
                                            Ok(())
                                        };
                                        
                                        match result {
                                            Ok(_) => {
                                                paint_app.save_dialog = SaveDialog::Hidden;
                                                if return_to_menu_val {
                                                    self.pending_action = PendingAction::ReturnToMenu;
                                                }
                                            },
                                            Err(e) => {
                                                self.error_message = Some(e);
                                                self.show_error = true;
                                            }
                                        }
                                    }
                                    if ui.button(get_text("no", self.language)).clicked() {
                                        paint_app.save_dialog = SaveDialog::Hidden;
                                        if return_to_menu_val {
                                            self.pending_action = PendingAction::ReturnToMenu;
                                        }
                                    }
                                    if ui.button(get_text("cancel", self.language)).clicked() {
                                        paint_app.save_dialog = SaveDialog::Hidden;
                                    }
                                });
                            });
                    }
                }
                
                paint_app.update_texture(ctx);

                egui::SidePanel::left("layers_panel")
                    .resizable(true)
                    .default_width(200.0)
                    .show(ctx, |ui| {
                        RustiqueTheme::panel_frame().show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(RustiqueTheme::heading_text(&get_text("layers", self.language), 16.0));
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                
                                ui.horizontal(|ui| {
                                    let btn_size = Vec2::new(32.0, 28.0);
                                    
                                    let add_btn = ui.add(
                                        egui::Button::new("")
                                            .fill(RustiqueTheme::ACCENT_PRIMARY)
                                            .stroke(egui::Stroke::new(1.0, RustiqueTheme::ACCENT_PRIMARY))
                                            .rounding(RustiqueTheme::rounding_small())
                                            .min_size(btn_size)
                                    );
                                    ui.put(add_btn.rect, ToolIcons::add());
                                    if add_btn.clicked() {
                                        paint_app.add_layer(format!("Layer {}", paint_app.current_state.layers.len() + 1));
                                    }
                                    
                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                    
                                    let remove_btn = ui.add(
                                        egui::Button::new("")
                                            .fill(if paint_app.current_state.layers.len() > 1 { 
                                                RustiqueTheme::ERROR 
                                            } else { 
                                                RustiqueTheme::SURFACE_PRIMARY 
                                            })
                                            .stroke(egui::Stroke::new(1.0, if paint_app.current_state.layers.len() > 1 { 
                                                RustiqueTheme::ERROR 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }))
                                            .rounding(RustiqueTheme::rounding_small())
                                            .min_size(btn_size)
                                    );
                                    ui.put(remove_btn.rect, ToolIcons::remove());
                                    if remove_btn.clicked() && paint_app.current_state.layers.len() > 1 {
                                        paint_app.remove_layer(paint_app.current_state.active_layer_index);
                                    }
                                    
                                    ui.add_space(RustiqueTheme::SPACING_SM);
                                    
                                    let up_btn = ui.add(
                                        egui::Button::new("")
                                            .fill(RustiqueTheme::SURFACE_SECONDARY)
                                            .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                            .rounding(RustiqueTheme::rounding_small())
                                            .min_size(btn_size)
                                    );
                                    ui.put(up_btn.rect, ToolIcons::move_up());
                                    if up_btn.clicked() {
                                        paint_app.move_layer_up(paint_app.current_state.active_layer_index);
                                    }
                                    
                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                    
                                    let down_btn = ui.add(
                                        egui::Button::new("")
                                            .fill(RustiqueTheme::SURFACE_SECONDARY)
                                            .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                            .rounding(RustiqueTheme::rounding_small())
                                            .min_size(btn_size)
                                    );
                                    ui.put(down_btn.rect, ToolIcons::move_down());
                                    if down_btn.clicked() {
                                        paint_app.move_layer_down(paint_app.current_state.active_layer_index);
                                    }
                                });
                                
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                
                                let layers_info: Vec<(usize, String, bool, bool)> = paint_app.current_state.layers
                                    .iter()
                                    .enumerate()
                                    .map(|(i, layer)| (i, layer.name.clone(), layer.visible, i == paint_app.current_state.active_layer_index))
                                    .collect();
                                
                                for (i, name, visible, is_active) in layers_info.iter().rev() {
                                    RustiqueTheme::card_frame().show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let visibility_btn = ui.add(
                                                egui::Button::new("")
                                                    .fill(Color32::TRANSPARENT)
                                                    .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                                    .rounding(RustiqueTheme::rounding_small())
                                                    .min_size(Vec2::new(28.0, 28.0))
                                            );
                                            
                                            ui.put(visibility_btn.rect, if *visible { 
                                                ToolIcons::layer_visible() 
                                            } else { 
                                                ToolIcons::layer_hidden() 
                                            });
                                            
                                            if visibility_btn.clicked() {
                                                self.pending_action = PendingAction::HandleLayerAction(
                                                    LayerAction::ToggleVisibility(*i)
                                                );
                                            }
                                            
                                            ui.add_space(RustiqueTheme::SPACING_XS);
                                            
                                            let layer_btn = ui.add(
                                                egui::Button::new(RustiqueTheme::body_text(name))
                                                    .fill(if *is_active { 
                                                        RustiqueTheme::ACCENT_PRIMARY.linear_multiply(0.3) 
                                                    } else { 
                                                        Color32::TRANSPARENT 
                                                    })
                                                    .stroke(egui::Stroke::new(
                                                        if *is_active { 2.0 } else { 1.0 },
                                                        if *is_active { 
                                                            RustiqueTheme::ACCENT_PRIMARY 
                                                        } else { 
                                                            RustiqueTheme::BORDER_LIGHT 
                                                        }
                                                    ))
                                                    .rounding(RustiqueTheme::rounding_small())
                                                    .min_size(Vec2::new(ui.available_width() - 60.0, 28.0))
                                            );
                                            
                                            if layer_btn.clicked() {
                                                self.pending_action = PendingAction::HandleLayerAction(
                                                    LayerAction::SetActive(*i)
                                                );
                                            }
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let edit_btn = ui.add(
                                                    egui::Button::new("")
                                                        .fill(Color32::TRANSPARENT)
                                                        .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                                        .rounding(RustiqueTheme::rounding_small())
                                                        .min_size(Vec2::new(24.0, 24.0))
                                                );
                                                ui.put(edit_btn.rect, ToolIcons::edit());
                                                if edit_btn.clicked() {
                                                    self.pending_action = PendingAction::HandleLayerAction(
                                                        LayerAction::Edit(*i)
                                                    );
                                                }
                                            });
                                        });
                                    });
                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                }
                            });
                        });
                    });

                egui::SidePanel::left("toolbar")
                    .resizable(false)
                    .exact_width(60.0)
                    .show(ctx, |ui| {
                        RustiqueTheme::panel_frame().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                
                                let tool_size = Vec2::new(40.0, 40.0);
                                
                                let brush_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(if paint_app.current_tool == Tool::Brush { 
                                            RustiqueTheme::ACCENT_PRIMARY 
                                        } else { 
                                            RustiqueTheme::SURFACE_PRIMARY 
                                        })
                                        .stroke(egui::Stroke::new(
                                            if paint_app.current_tool == Tool::Brush { 2.0 } else { 1.0 },
                                            if paint_app.current_tool == Tool::Brush { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(tool_size)
                                );
                                ui.put(brush_btn.rect, ToolIcons::brush());
                                if brush_btn.clicked() {
                                    paint_app.current_tool = Tool::Brush;
                                }
                                brush_btn.on_hover_text("Brush Tool");
                                
                                ui.add_space(RustiqueTheme::SPACING_XS);
                                
                                let eraser_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(if paint_app.current_tool == Tool::Eraser { 
                                            RustiqueTheme::ACCENT_PRIMARY 
                                        } else { 
                                            RustiqueTheme::SURFACE_PRIMARY 
                                        })
                                        .stroke(egui::Stroke::new(
                                            if paint_app.current_tool == Tool::Eraser { 2.0 } else { 1.0 },
                                            if paint_app.current_tool == Tool::Eraser { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(tool_size)
                                );
                                ui.put(eraser_btn.rect, ToolIcons::eraser());
                                if eraser_btn.clicked() {
                                    paint_app.current_tool = Tool::Eraser;
                                }
                                eraser_btn.on_hover_text("Eraser Tool");
                                
                                ui.add_space(RustiqueTheme::SPACING_XS);
                                
                                let bucket_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(if paint_app.current_tool == Tool::PaintBucket { 
                                            RustiqueTheme::ACCENT_PRIMARY 
                                        } else { 
                                            RustiqueTheme::SURFACE_PRIMARY 
                                        })
                                        .stroke(egui::Stroke::new(
                                            if paint_app.current_tool == Tool::PaintBucket { 2.0 } else { 1.0 },
                                            if paint_app.current_tool == Tool::PaintBucket { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(tool_size)
                                );
                                ui.put(bucket_btn.rect, ToolIcons::paint_bucket());
                                if bucket_btn.clicked() {
                                    paint_app.current_tool = Tool::PaintBucket;
                                }
                                bucket_btn.on_hover_text("Paint Bucket");
                                
                                ui.add_space(RustiqueTheme::SPACING_XS);
                                
                                let picker_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(if paint_app.current_tool == Tool::ColorPicker { 
                                            RustiqueTheme::ACCENT_PRIMARY 
                                        } else { 
                                            RustiqueTheme::SURFACE_PRIMARY 
                                        })
                                        .stroke(egui::Stroke::new(
                                            if paint_app.current_tool == Tool::ColorPicker { 2.0 } else { 1.0 },
                                            if paint_app.current_tool == Tool::ColorPicker { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(tool_size)
                                );
                                ui.put(picker_btn.rect, ToolIcons::color_picker());
                                if picker_btn.clicked() {
                                    paint_app.current_tool = Tool::ColorPicker;
                                }
                                picker_btn.on_hover_text("Color Picker");
                                
                                ui.add_space(RustiqueTheme::SPACING_XS);
                                
                                let line_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(if paint_app.current_tool == Tool::Line { 
                                            RustiqueTheme::ACCENT_PRIMARY 
                                        } else { 
                                            RustiqueTheme::SURFACE_PRIMARY 
                                        })
                                        .stroke(egui::Stroke::new(
                                            if paint_app.current_tool == Tool::Line { 2.0 } else { 1.0 },
                                            if paint_app.current_tool == Tool::Line { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(tool_size)
                                );
                                ui.put(line_btn.rect, ToolIcons::line());
                                if line_btn.clicked() {
                                    paint_app.current_tool = Tool::Line;
                                }
                                line_btn.on_hover_text("Line Tool");
                                
                                ui.add_space(RustiqueTheme::SPACING_MD);
                                
                                ui.separator();
                                
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                
                                let color_size = 35.0;
                                let primary_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(paint_app.primary_color)
                                        .stroke(egui::Stroke::new(
                                            if !paint_app.using_secondary_color { 3.0 } else { 1.0 }, 
                                            if !paint_app.using_secondary_color { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::TEXT_PRIMARY 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(Vec2::new(color_size, color_size))
                                );
                                if primary_btn.clicked() {
                                    paint_app.using_secondary_color = false;
                                }
                                primary_btn.on_hover_text("Primary Color (Click to use)");
                                
                                ui.add_space(RustiqueTheme::SPACING_XS);
                                
                                let secondary_btn = ui.add(
                                    egui::Button::new("")
                                        .fill(paint_app.secondary_color)
                                        .stroke(egui::Stroke::new(
                                            if paint_app.using_secondary_color { 3.0 } else { 1.0 }, 
                                            if paint_app.using_secondary_color { 
                                                RustiqueTheme::ACCENT_PRIMARY 
                                            } else { 
                                                RustiqueTheme::BORDER_LIGHT 
                                            }
                                        ))
                                        .rounding(RustiqueTheme::rounding_small())
                                        .min_size(Vec2::new(color_size * 0.8, color_size * 0.8))
                                );
                                if secondary_btn.clicked() {
                                    paint_app.using_secondary_color = true;
                                }
                                secondary_btn.on_hover_text("Secondary Color (Click to use)");
                            });
                        });
                    });

                egui::SidePanel::right("properties_panel")
                    .resizable(true)
                    .default_width(250.0)
                    .show(ctx, |ui| {
                        RustiqueTheme::panel_frame().show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.label(RustiqueTheme::heading_text(&get_text("properties", self.language), 16.0));
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false; 2])
                                    .show(ui, |ui| {
                                        RustiqueTheme::card_frame().show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.label(RustiqueTheme::body_text(&get_text("brush_system", self.language)));
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                if paint_app.brush_manager.brush_selector_grid(ui, ctx, self.language) {
                                                    paint_app.brush_manager.current_size = paint_app.brush_size as f32;
                                                }
                                            });
                                        });
                                        
                                        ui.add_space(RustiqueTheme::SPACING_MD);
                                        
                                        RustiqueTheme::card_frame().show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.label(RustiqueTheme::body_text(&get_text("tool_settings", self.language)));
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.horizontal(|ui| {
                                                    ui.label(RustiqueTheme::muted_text(&get_text("brush_size", self.language)));
                                                    ui.add(egui::DragValue::new(&mut paint_app.brush_size)
                                                        .speed(0.1)
                                                        .clamp_range(1..=500)
                                                        .suffix("px"));
                                                });
                                                
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.horizontal(|ui| {
                                                    ui.label(RustiqueTheme::muted_text(&get_text("eraser_size", self.language)));
                                                    ui.add(egui::DragValue::new(&mut paint_app.eraser_size)
                                                        .speed(0.1)
                                                        .clamp_range(1..=500)
                                                        .suffix("px"));
                                                });
                                                
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.horizontal(|ui| {
                                                    ui.label(RustiqueTheme::muted_text(&get_text("zoom", self.language)));
                                                    ui.add(egui::Slider::new(&mut paint_app.zoom, 0.1..=10.0)
                                                        .logarithmic(true)
                                                        .suffix("x"));
                                                });
                                                
                                                ui.add_space(RustiqueTheme::SPACING_MD);
                                                ui.separator();
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.label(RustiqueTheme::body_text(&get_text("pressure_settings", self.language)));
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.horizontal(|ui| {
                                                    ui.checkbox(&mut paint_app.pressure_enabled, 
                                                               &get_text("enable_pressure", self.language));
                                                });
                                                
                                                if paint_app.pressure_enabled {
                                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                                    
                                                    ui.horizontal(|ui| {
                                                        ui.label(RustiqueTheme::muted_text(&format!("{}:", get_text("smoothing", self.language))));
                                                        ui.add(egui::Slider::new(&mut paint_app.pressure_smoothing, 0.0..=1.0)
                                                            .suffix("%"));
                                                    });
                                                    
                                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                                    
                                                    ui.horizontal(|ui| {
                                                        ui.label(RustiqueTheme::muted_text(&format!("{}:", get_text("velocity_sensitivity", self.language))));
                                                        ui.add(egui::Slider::new(&mut paint_app.velocity_sensitivity, 0.0..=1.0)
                                                            .suffix("%"));
                                                    });
                                                    
                                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                                    
                                                    ui.horizontal(|ui| {
                                                        ui.label(RustiqueTheme::muted_text(&format!("{}:", get_text("max_velocity", self.language))));
                                                        ui.add(egui::Slider::new(&mut paint_app.max_velocity_for_min_pressure, 100.0..=2000.0)
                                                            .suffix("px/s"));
                                                    });
                                                    
                                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                                    
                                                    ui.horizontal(|ui| {
                                                        ui.label(RustiqueTheme::muted_text(&format!("{}: {:.2}", get_text("current", self.language), paint_app.current_pressure)));
                                                        
                                                        let bar_rect = ui.allocate_space(egui::Vec2::new(100.0, 10.0)).1;
                                                        ui.painter().rect_filled(bar_rect, 2.0, egui::Color32::DARK_GRAY);
                                                        let fill_width = bar_rect.width() * paint_app.current_pressure;
                                                        let fill_rect = egui::Rect::from_min_size(
                                                            bar_rect.min, 
                                                            egui::Vec2::new(fill_width, bar_rect.height())
                                                        );
                                                        let color = if paint_app.current_pressure < 0.5 {
                                                            egui::Color32::from_rgb(255, 100, 100)
                                                        } else {
                                                            egui::Color32::from_rgb(100, 255, 100)
                                                        };
                                                        ui.painter().rect_filled(fill_rect, 2.0, color);
                                                    });
                                                    
                                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                                    
                                                    let active_brush = paint_app.brush_manager.active_brush_mut();
                                                    
                                                    ui.horizontal(|ui| {
                                                        ui.checkbox(&mut active_brush.pressure_affects_size, 
                                                                   &get_text("affects_size", self.language));
                                                    });
                                                    
                                                    if active_brush.pressure_affects_size {
                                                        ui.horizontal(|ui| {
                                                            ui.label(RustiqueTheme::muted_text(&format!("{}:", get_text("min_size", self.language))));
                                                            ui.add(egui::Slider::new(&mut active_brush.pressure_size_min, 0.1..=1.0)
                                                                .suffix("%"));
                                                        });
                                                    }
                                                    
                                                    ui.horizontal(|ui| {
                                                        ui.checkbox(&mut active_brush.pressure_affects_opacity, 
                                                                   &get_text("affects_opacity", self.language));
                                                    });
                                                    
                                                    if active_brush.pressure_affects_opacity {
                                                        ui.horizontal(|ui| {
                                                            ui.label(RustiqueTheme::muted_text(&format!("{}:", get_text("min_opacity", self.language))));
                                                            ui.add(egui::Slider::new(&mut active_brush.pressure_opacity_min, 0.0..=1.0)
                                                                .suffix("%"));
                                                        });
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.add_space(RustiqueTheme::SPACING_MD);
                                        
                                        RustiqueTheme::card_frame().show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.label(RustiqueTheme::body_text(&get_text("colors", self.language)));
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.horizontal(|ui| {
                                                    ui.label(RustiqueTheme::muted_text(&get_text("primary", self.language)));
                                                    ui.color_edit_button_srgba(&mut paint_app.primary_color);
                                                    let add_primary_btn = ui.add(
                                                        egui::Button::new("")
                                                            .fill(RustiqueTheme::SURFACE_SECONDARY)
                                                            .rounding(RustiqueTheme::rounding_small())
                                                            .min_size(Vec2::new(24.0, 24.0))
                                                    );
                                                    ui.put(add_primary_btn.rect, ToolIcons::add());
                                                    if add_primary_btn.clicked() {
                                                        paint_app.add_saved_color(paint_app.primary_color);
                                                    }
                                                });
                                                
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                ui.horizontal(|ui| {
                                                    ui.label(RustiqueTheme::muted_text(&get_text("secondary", self.language)));
                                                    ui.color_edit_button_srgba(&mut paint_app.secondary_color);
                                                    let add_secondary_btn = ui.add(
                                                        egui::Button::new("")
                                                            .fill(RustiqueTheme::SURFACE_SECONDARY)
                                                            .rounding(RustiqueTheme::rounding_small())
                                                            .min_size(Vec2::new(24.0, 24.0))
                                                    );
                                                    ui.put(add_secondary_btn.rect, ToolIcons::add());
                                                    if add_secondary_btn.clicked() {
                                                        paint_app.add_saved_color(paint_app.secondary_color);
                                                    }
                                                });
                                            });
                                        });
                                        
                                        if !paint_app.saved_colors.is_empty() {
                                            ui.add_space(RustiqueTheme::SPACING_MD);
                                            
                                            RustiqueTheme::card_frame().show(ui, |ui| {
                                                ui.vertical(|ui| {
                                                    ui.label(RustiqueTheme::body_text(&get_text("saved_colors", self.language)));
                                                    ui.add_space(RustiqueTheme::SPACING_XS);
                                                    
                                                    let available_width = ui.available_width();
                                                    let color_size = 28.0;
                                                    let spacing = 4.0;
                                                    let colors_per_row = ((available_width + spacing) / (color_size + spacing)).floor() as usize;
                                                    let colors_count = paint_app.saved_colors.len();
                                                    
                                                    for i in 0..(colors_count / colors_per_row + (if colors_count % colors_per_row > 0 { 1 } else { 0 })) {
                                                        ui.horizontal(|ui| {
                                                            for j in 0..colors_per_row {
                                                                let idx = i * colors_per_row + j;
                                                                if idx < colors_count {
                                                                    let color = paint_app.saved_colors[idx];
                                                                    let btn = ui.add(
                                                                        egui::Button::new("")
                                                                            .fill(color)
                                                                            .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                                                            .rounding(RustiqueTheme::rounding_small())
                                                                            .min_size(Vec2::new(color_size, color_size))
                                                                    );
                                                                    
                                                                    if btn.clicked() {
                                                                        paint_app.primary_color = color;
                                                                    }
                                                                    if btn.clicked_by(egui::PointerButton::Secondary) {
                                                                        paint_app.secondary_color = color;
                                                                    }
                                                                    if btn.clicked_by(egui::PointerButton::Middle) {
                                                                        paint_app.remove_saved_color(idx);
                                                                        break;
                                                                    }
                                                                    
                                                                    btn.on_hover_text("Left: Primary | Right: Secondary | Middle: Delete");
                                                                }
                                                            }
                                                        });
                                                    }
                                                });
                                            });
                                        }
                                        
                                        ui.add_space(RustiqueTheme::SPACING_MD);
                                        
                                        RustiqueTheme::card_frame().show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.label(RustiqueTheme::body_text("Quick Actions"));
                                                ui.add_space(RustiqueTheme::SPACING_XS);
                                                
                                                if ui.add(
                                                    egui::Button::new(RichText::new("💾 Save File").size(14.0))
                                                        .fill(RustiqueTheme::SURFACE_SECONDARY)
                                                        .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                                        .rounding(RustiqueTheme::rounding_medium())
                                                        .min_size(Vec2::new(ui.available_width(), 32.0))
                                                ).clicked() {
                                                    if let Some(path) = FileDialog::new()
                                                        .add_filter("All Supported Files", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "gif", "webp", "rustiq"])
                                                        .add_filter("PNG Image", &["png"])
                                                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                                                        .add_filter("BMP Image", &["bmp"])
                                                        .add_filter("TIFF Image", &["tiff", "tif"])
                                                        .add_filter("GIF Image", &["gif"])
                                                        .add_filter("WebP Image", &["webp"])
                                                        .add_filter("Rustique File", &["rustiq"])
                                                        .set_directory("/")
                                                        .save_file() {
                                                        match paint_app.save_file(path.to_str().unwrap()) {
                                                            Ok(_) => {},
                                                            Err(e) => {
                                                                self.error_message = Some(e);
                                                                self.show_error = true;
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        });
                                    });
                            });
                        });
                    });

                let (undo_clicked, redo_clicked, return_to_menu_clicked) = egui::TopBottomPanel::top("top_panel")
                    .frame(RustiqueTheme::panel_frame())
                    .show(ctx, |ui| {
                        let mut return_clicked = false;
                        let mut undo_clicked = false;
                        let mut redo_clicked = false;
                        
                        ui.horizontal(|ui| {
                            ui.add_space(RustiqueTheme::SPACING_SM);
                            
                            let btn_size = Vec2::new(40.0, 32.0);
                            
                            let home_btn = ui.add(
                                egui::Button::new("")
                                    .fill(RustiqueTheme::SURFACE_SECONDARY)
                                    .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                    .rounding(RustiqueTheme::rounding_medium())
                                    .min_size(btn_size)
                            );
                            ui.put(home_btn.rect, ToolIcons::home());
                            if home_btn.clicked() {
                                return_clicked = true;
                            }
                            
                            ui.add_space(RustiqueTheme::SPACING_LG);
                            
                            let undo_btn = ui.add(
                                egui::Button::new("")
                                    .fill(RustiqueTheme::SURFACE_SECONDARY)
                                    .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                    .rounding(RustiqueTheme::rounding_medium())
                                    .min_size(btn_size)
                            );
                            ui.put(undo_btn.rect, ToolIcons::undo());
                            if undo_btn.clicked() {
                                undo_clicked = true;
                            }
                            
                            ui.add_space(RustiqueTheme::SPACING_XS);
                            
                            let redo_btn = ui.add(
                                egui::Button::new("")
                                    .fill(RustiqueTheme::SURFACE_SECONDARY)
                                    .stroke(egui::Stroke::new(1.0, RustiqueTheme::BORDER_LIGHT))
                                    .rounding(RustiqueTheme::rounding_medium())
                                    .min_size(btn_size)
                            );
                            ui.put(redo_btn.rect, ToolIcons::redo());
                            if redo_btn.clicked() {
                                redo_clicked = true;
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(RustiqueTheme::SPACING_SM);
                                ui.label(RustiqueTheme::muted_text("Ctrl+Z: Undo | Ctrl+Y: Redo | Ctrl+S: Save"));
                            });
                        });
                        (undo_clicked, redo_clicked, return_clicked)
                    }).inner;
                
                if undo_clicked {
                    self.pending_action = PendingAction::UndoAction;
                }
                
                if redo_clicked {
                    self.pending_action = PendingAction::RedoAction;
                }
                
                if return_to_menu_clicked {
                    if paint_app.has_unsaved_changes {
                        paint_app.show_save_dialog(true);
                    } else {
                        self.pending_action = PendingAction::ReturnToMenu;
                    }
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    let available_size = ui.available_size();
                    let canvas_width = paint_app.current_state.width as f32;
                    let canvas_height = paint_app.current_state.height as f32;
                    let scale = (available_size.x / canvas_width).min(available_size.y / canvas_height);
                    let scaled_size = Vec2::new(canvas_width * scale * paint_app.zoom, canvas_height * scale * paint_app.zoom);
                    let canvas_rect = Rect::from_center_size(
                        ui.available_rect_before_wrap().center() + paint_app.pan,
                        scaled_size,
                    );

                    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

                    if let Some(texture) = &paint_app.texture {
                        painter.image(texture.id(), canvas_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                    }

                    let to_canvas = egui::emath::RectTransform::from_to(
                        canvas_rect,
                        Rect::from_min_size(Pos2::ZERO, Vec2::new(canvas_width, canvas_height)),
                    );

                    if response.dragged_by(egui::PointerButton::Middle) {
                        paint_app.pan += response.drag_delta();
                    }

                    if paint_app.current_tool == Tool::Line {
                        if response.clicked() && !response.clicked_by(egui::PointerButton::Middle) {
                            let is_secondary = response.clicked_by(egui::PointerButton::Secondary);
                            if let Some(pos) = response.interact_pointer_pos() {
                                let canvas_pos = to_canvas.transform_pos(pos);
                                let x = canvas_pos.x as i32;
                                let y = canvas_pos.y as i32;
                                
                                if paint_app.is_first_click_line {
                                    paint_app.line_start = Some((x, y));
                                    paint_app.line_end = Some((x, y));
                                    paint_app.is_drawing_line = true;
                                    paint_app.is_first_click_line = false;
                                } else {
                                    if let (Some(start), Some(_)) = (paint_app.line_start, paint_app.line_end) {
                                        let color = if is_secondary { paint_app.secondary_color } else { paint_app.primary_color };
                                        paint_app.draw_line(start, (x, y), color);
                                        paint_app.is_drawing_line = false;
                                        paint_app.line_start = None;
                                        paint_app.line_end = None;
                                        paint_app.is_first_click_line = true;
                                        paint_app.save_state();
                                    }
                                }
                            }
                        }
                        
                        if paint_app.is_drawing_line && !paint_app.is_first_click_line {
                            if let Some(pos) = response.hover_pos() {
                                let canvas_pos = to_canvas.transform_pos(pos);
                                paint_app.line_end = Some((canvas_pos.x as i32, canvas_pos.y as i32));
                                
                                if let (Some(start), Some(end)) = (paint_app.line_start, paint_app.line_end) {
                                    let start_pos = Pos2::new(
                                        start.0 as f32 * canvas_rect.width() / canvas_width + canvas_rect.min.x,
                                        start.1 as f32 * canvas_rect.height() / canvas_height + canvas_rect.min.y
                                    );
                                    let end_pos = Pos2::new(
                                        end.0 as f32 * canvas_rect.width() / canvas_width + canvas_rect.min.x,
                                        end.1 as f32 * canvas_rect.height() / canvas_height + canvas_rect.min.y
                                    );
                                    
                                    let is_secondary = response.ctx.input(|i| i.pointer.button_down(egui::PointerButton::Secondary));
                                    let color = if is_secondary { paint_app.secondary_color } else { paint_app.primary_color };
                                    let size = paint_app.brush_size as f32;
                                    
                                    painter.line_segment([start_pos, end_pos], Stroke::new(size * paint_app.zoom, color));
                                }
                            }
                        }
                        
                        if response.clicked_by(egui::PointerButton::Middle) {
                            paint_app.is_drawing_line = false;
                            paint_app.line_start = None;
                            paint_app.line_end = None;
                            paint_app.is_first_click_line = true;
                        }
                        
                        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            paint_app.is_drawing_line = false;
                            paint_app.line_start = None;
                            paint_app.line_end = None;
                            paint_app.is_first_click_line = true;
                        }
                    } else {
                        if (response.clicked_by(egui::PointerButton::Primary) ||
                            response.clicked_by(egui::PointerButton::Secondary)) && 
                           !response.clicked_by(egui::PointerButton::Middle) {
                            paint_app.is_drawing = true;
                            paint_app.save_state();
                        }

                        if (response.dragged() || response.clicked()) && 
                           !(response.dragged_by(egui::PointerButton::Middle) || 
                             response.clicked_by(egui::PointerButton::Middle)) {
                            if let Some(pos) = response.interact_pointer_pos() {
                                
                                let current_time = response.ctx.input(|i| i.time);
                                paint_app.update_pressure_from_velocity(pos, current_time);
                                
                                let canvas_pos = to_canvas.transform_pos(pos);
                                let x = canvas_pos.x as usize;
                                let y = canvas_pos.y as usize;
                                let is_secondary = response.dragged_by(egui::PointerButton::Secondary) || 
                                                 response.clicked_by(egui::PointerButton::Secondary);
                                
                                if x < paint_app.current_state.width && y < paint_app.current_state.height {
                                    match paint_app.current_tool {
                                        Tool::PaintBucket => paint_app.paint_bucket(x, y, is_secondary),
                                        Tool::ColorPicker => paint_app.pick_color(x, y, is_secondary),
                                        _ => {
                                            let (x, y) = (canvas_pos.x as i32, canvas_pos.y as i32);
                                            if let Some(last_pos) = paint_app.last_position {
                                                paint_app.draw_line(last_pos, (x, y), 
                                                                  if is_secondary { paint_app.secondary_color } 
                                                                  else { paint_app.primary_color });
                                            } else {
                                                paint_app.draw_point(x, y, is_secondary);
                                            }
                                            paint_app.last_position = Some((x, y));
                                        }
                                    }
                                    paint_app.is_drawing = true;
                                }
                            }
                        } else {
                            paint_app.save_state();
                            paint_app.last_position = None;
                            paint_app.last_cursor_pos = None;
                            paint_app.last_cursor_time = None;
                        }
                    }

                    let delta = ui.input(|i| i.scroll_delta.y);
                    if delta != 0.0 {
                        let zoom_speed = 0.001;
                        let old_zoom = paint_app.zoom;
                        paint_app.zoom *= 1.0 + delta * zoom_speed;
                        paint_app.zoom = paint_app.zoom.clamp(0.1, 10.0);
                        
                        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                            let center = ui.available_rect_before_wrap().center();
                            let mouse_offset = mouse_pos - center - paint_app.pan;
                            let zoom_factor = paint_app.zoom / old_zoom;
                            paint_app.pan += mouse_offset * (1.0 - zoom_factor);
                        }
                    }
                });
            }
        }
    }
}

fn load_app_icon() -> Option<eframe::IconData> {
    if let Some(icon_bytes) = Assets::get_image_bytes("rustique_icon.png") {
        if let Ok(icon_image) = image::load_from_memory(&icon_bytes) {
            let icon_rgba = icon_image.to_rgba8();
            let (width, height) = (icon_rgba.width(), icon_rgba.height());
            
            return Some(eframe::IconData {
                rgba: icon_rgba.into_raw(),
                width,
                height,
            });
        }
    }
    
    None
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        icon_data: load_app_icon(),
        ..Default::default()
    };
    eframe::run_native(
        "Rustique Paint",
        native_options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}