use eframe::egui;
use egui::{Color32, Vec2, Pos2, Rect, Stroke};
use std::f32::consts::PI;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushType {
    Round,
    Flat,
    Bright,
    Filbert,
    Fan,
    Angle,
    Mop,
    Rigger,
}

impl BrushType {
    pub fn get_name(&self, language: crate::localization::Language) -> String {
        use crate::localization::get_text;
        match self {
            BrushType::Round => get_text("brush_round", language),
            BrushType::Flat => get_text("brush_flat", language),
            BrushType::Bright => get_text("brush_bright", language),
            BrushType::Filbert => get_text("brush_filbert", language),
            BrushType::Fan => get_text("brush_fan", language),
            BrushType::Angle => get_text("brush_angle", language),
            BrushType::Mop => get_text("brush_mop", language),
            BrushType::Rigger => get_text("brush_rigger", language),
        }
    }
    
    pub fn all_types() -> Vec<BrushType> {
        vec![
            BrushType::Round,
            BrushType::Flat,
            BrushType::Bright,
            BrushType::Filbert,
            BrushType::Fan,
            BrushType::Angle,
            BrushType::Mop,
            BrushType::Rigger,
        ]
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BrushProperties {
    pub brush_type: BrushType,
    pub size: f32,
    pub stretch_factor: f32,
    pub angle_sensitivity: f32,
    pub pressure_sensitivity: f32,
    pub blend_mode: BlendMode,
    pub spacing: f32,
    pub hardness: f32,
    pub base_rotation: f32,
    pub pressure_affects_size: bool,
    pub pressure_affects_opacity: bool,
    pub pressure_size_min: f32,
    pub pressure_opacity_min: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
}

impl Default for BrushProperties {
    fn default() -> Self {
        Self {
            brush_type: BrushType::Round,
            size: 10.0,
            stretch_factor: 1.0,
            angle_sensitivity: 0.0,
            pressure_sensitivity: 0.5,
            blend_mode: BlendMode::Normal,
            spacing: 0.05,
            hardness: 1.0,
            base_rotation: 0.0,
            pressure_affects_size: true,
            pressure_affects_opacity: true,
            pressure_size_min: 0.2,
            pressure_opacity_min: 0.1,
        }
    }
}

impl BrushProperties {
    pub fn from_type(brush_type: BrushType) -> Self {
        let mut properties = Self::default();
        properties.brush_type = brush_type;
        
        match brush_type {
            BrushType::Round => {
                properties.stretch_factor = 1.0;
                properties.angle_sensitivity = 0.0;
                properties.hardness = 1.0;
                properties.spacing = 0.05;
            },
            BrushType::Flat => {
                properties.stretch_factor = 4.0;
                properties.angle_sensitivity = 0.8;
                properties.hardness = 1.0;
                properties.spacing = 0.05;
            },
            BrushType::Bright => {
                properties.stretch_factor = 3.0;
                properties.angle_sensitivity = 0.7;
                properties.hardness = 1.0;
                properties.spacing = 0.05;
            },
            BrushType::Filbert => {
                properties.stretch_factor = 2.5;
                properties.angle_sensitivity = 0.6;
                properties.hardness = 0.8;
                properties.spacing = 0.05;
            },
            BrushType::Fan => {
                properties.stretch_factor = 3.0;
                properties.angle_sensitivity = 0.8; 
                properties.hardness = 1.0;
                properties.spacing = 0.08;
            },
            BrushType::Angle => {
                properties.stretch_factor = 2.0;
                properties.angle_sensitivity = 0.8;
                properties.base_rotation = PI / 4.0;
                properties.hardness = 1.0;
                properties.spacing = 0.05;
            },
            BrushType::Mop => {
                properties.stretch_factor = 1.2;
                properties.angle_sensitivity = 0.1;
                properties.hardness = 0.5;
                properties.spacing = 0.03;
            },
            BrushType::Rigger => {
                properties.stretch_factor = 0.5;
                properties.angle_sensitivity = 0.2;
                properties.hardness = 1.0;
                properties.spacing = 0.02;
            },
        }
        
        properties
    }
}

pub struct BrushManager {
    pub brushes: Vec<BrushProperties>,
    pub active_brush_index: usize,
    pub current_angle: f32,
    pub last_position: Option<(f32, f32)>,
    pub current_size: f32,
}

impl Default for BrushManager {
    fn default() -> Self {
        let mut brushes = Vec::new();
        
        for brush_type in BrushType::all_types() {
            brushes.push(BrushProperties::from_type(brush_type));
        }
        
        Self {
            brushes,
            active_brush_index: 0,
            current_angle: 0.0,
            last_position: None,
            current_size: 3.0,
        }
    }
}

impl BrushManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn active_brush(&self) -> &BrushProperties {
        &self.brushes[self.active_brush_index]
    }
    
    pub fn active_brush_mut(&mut self) -> &mut BrushProperties {
        &mut self.brushes[self.active_brush_index]
    }
    
    pub fn update_angle(&mut self, x: f32, y: f32) {
        if let Some((prev_x, prev_y)) = self.last_position {
            let dx = x - prev_x;
            let dy = y - prev_y;
            
            if dx * dx + dy * dy > 0.25 {
                let new_angle = dy.atan2(dx);
                let angle_diff = new_angle - self.current_angle;
                
                let normalized_diff = if angle_diff > std::f32::consts::PI {
                    angle_diff - 2.0 * std::f32::consts::PI
                } else if angle_diff < -std::f32::consts::PI {
                    angle_diff + 2.0 * std::f32::consts::PI
                } else {
                    angle_diff
                };
                
                let smoothing_factor = 0.2;
                self.current_angle += normalized_diff * smoothing_factor;
            }
        }
        
        self.last_position = Some((x, y));
    }
    
    pub fn generate_brush_mask(&self, size: usize) -> Vec<f32> {
        let active = self.active_brush();
        let mut mask = vec![0.0; size * size];
        let center = size as f32 / 2.0;
        let radius = center;
        let effective_angle = self.current_angle + active.base_rotation;
        let cos_a = effective_angle.cos();
        let sin_a = effective_angle.sin();
        
        for y in 0..size {
            for x in 0..size {
                let rx = (x as f32 - center) / radius;
                let ry = (y as f32 - center) / radius;
                let mut value: f32 = 0.0;
                
                match active.brush_type {
                    BrushType::Round => {
                        let dist = (rx * rx + ry * ry).sqrt();
                        if dist <= 1.0 {
                            value = 1.0;
                        }
                    },
                    
                    BrushType::Flat => {
                        let rx_rot = rx * cos_a - ry * sin_a;
                        let ry_rot = rx * sin_a + ry * cos_a;
                        
                        if rx_rot.abs() <= 0.2 && ry_rot.abs() <= 1.0 {
                            value = 1.0;
                        }
                    },
                    
                    BrushType::Bright => {
                        let rx_rot = rx * cos_a - ry * sin_a;
                        let ry_rot = rx * sin_a + ry * cos_a;
                        
                        if rx_rot.abs() <= 0.3 && ry_rot.abs() <= 0.8 {
                            value = 1.0;
                        }
                    },
                    
                    BrushType::Filbert => {
                        let rx_rot = rx * cos_a - ry * sin_a;
                        let ry_rot = rx * sin_a + ry * cos_a;
                        
                        let ellipse_a = 0.6;
                        let ellipse_b = 1.0;
                        let ellipse_dist = (rx_rot * rx_rot) / (ellipse_a * ellipse_a) + 
                                          (ry_rot * ry_rot) / (ellipse_b * ellipse_b);
                        
                        if ellipse_dist <= 1.0 {
                            value = 1.0;
                        }
                    },
                    
                    BrushType::Fan => {
                        let angle_from_center = ry.atan2(rx) + std::f32::consts::PI;
                        let dist = (rx * rx + ry * ry).sqrt();
                        
                        if dist <= 1.0 {
                            let fan_segments = 5.0;
                            let segment_width = std::f32::consts::PI * 0.9 / fan_segments;
                            let normalized_angle = (angle_from_center % (std::f32::consts::PI * 2.0)) - std::f32::consts::PI * 0.55;
                            
                            for i in 0..5 {
                                let segment_center = i as f32 * segment_width;
                                let distance_from_segment = (normalized_angle - segment_center).abs();
                                
                                if distance_from_segment < segment_width * 0.4 {
                                    value = 1.0;
                                    break;
                                }
                            }
                        }
                    },
                    
                    BrushType::Angle => {
                        let rx_rot = rx * cos_a - ry * sin_a;
                        let ry_rot = rx * sin_a + ry * cos_a;
                        
                        if ry_rot.abs() <= 0.7 {
                            let left_edge = -0.8;
                            let right_edge = 0.4;
                            
                            if rx_rot >= left_edge && rx_rot <= right_edge {
                                value = 1.0;
                            }
                        }
                    },
                    
                    BrushType::Mop => {
                        let dist = (rx * rx + ry * ry).sqrt();
                        if dist <= 1.0 {
                            value = 1.0;
                        }
                    },
                    
                    BrushType::Rigger => {
                        let rx_rot = rx * cos_a - ry * sin_a;
                        let ry_rot = rx * sin_a + ry * cos_a;
                        
                        if rx_rot.abs() <= 0.08 && ry_rot.abs() <= 0.9 {
                            value = 1.0;
                        }
                    },
                }
                
                if value > 0.0 && active.hardness < 1.0 {
                    value = value.powf(1.0 / active.hardness.max(0.1));
                }
                
                mask[y * size + x] = value.clamp(0.0, 1.0);
            }
        }
        
        mask
    }
    
    pub fn draw_point(&mut self, x: i32, y: i32, color: Color32, pressure: f32, record_change: &mut dyn FnMut(usize, usize, Option<Color32>)) {
        let active = self.active_brush();
        let clamped_pressure = pressure.clamp(0.0, 1.0);
        
        let effective_size = if active.pressure_affects_size {
            let size_factor = active.pressure_size_min + (1.0 - active.pressure_size_min) * clamped_pressure;
            (self.current_size as f32 * size_factor).max(1.0) as usize * 2 + 1
        } else {
            self.current_size as usize * 2 + 1
        };
        
        let effective_opacity = if active.pressure_affects_opacity {
            active.pressure_opacity_min + (1.0 - active.pressure_opacity_min) * clamped_pressure
        } else {
            1.0
        };
        
        self.update_angle(x as f32, y as f32);
        let mask = self.generate_brush_mask(effective_size);
        let center = effective_size as i32 / 2;
        
        for dy in 0..effective_size as i32 {
            for dx in 0..effective_size as i32 {
                let nx = x + dx - center;
                let ny = y + dy - center;
                let mask_value = mask[(dy as usize) * effective_size + (dx as usize)];
                
                if mask_value > 0.0 && nx >= 0 && ny >= 0 {
                    let alpha = (color.a() as f32 * mask_value * effective_opacity) as u8;
                    let new_color = if alpha > 0 {
                        Some(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha))
                    } else {
                        None
                    };
                    
                    record_change(nx as usize, ny as usize, new_color);
                }
            }
        }
    }
    
    pub fn draw_line(&mut self, start: (i32, i32), end: (i32, i32), color: Color32, pressure: f32, record_change: &mut dyn FnMut(usize, usize, Option<Color32>)) {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        
        let _total_length = ((x1 - x0).pow(2) + (y1 - y0).pow(2)) as f32;
        let spacing = (self.active_brush().spacing * self.current_size).max(1.0);
        let mut points = Vec::new();
        points.push((x, y));
        let mut accumulated_distance = 0.0;
        
        loop {
            let old_x = x;
            let old_y = y;
            
            let e2 = 2 * err;
            if e2 >= dy {
                if x == x1 { break; }
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                if y == y1 { break; }
                err += dx;
                y += sy;
            }
            
            let segment_length = ((x - old_x).pow(2) + (y - old_y).pow(2)) as f32;
            accumulated_distance += segment_length;
            
            if accumulated_distance >= spacing {
                points.push((x, y));
                accumulated_distance = 0.0;
            }
        }
        
        if points.last() != Some(&(x1, y1)) {
            points.push((x1, y1));
        }
        
        for &(px, py) in &points {
            self.draw_point(px, py, color, pressure, record_change);
        }
    }
    
    pub fn brush_selector_grid(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context, language: crate::localization::Language) -> bool {
        use crate::localization::get_text;
        let mut changed = false;
        
        ui.heading(get_text("select_brush", language));
        ui.separator();
        
        let cell_size = Vec2::new(64.0, 64.0);
        let margin = 8.0;
        let total_size = cell_size + Vec2::splat(margin * 2.0);
        let available_width = ui.available_width();
        let columns = (available_width / total_size.x).floor().max(1.0) as usize;
        
        egui::Grid::new("brush_selector_grid")
            .spacing([margin, margin])
            .min_col_width(cell_size.x)
            .min_row_height(cell_size.y)
            .show(ui, |ui| {
                let mut col = 0;
                
                let brush_types: Vec<(usize, BrushType)> = self.brushes
                    .iter()
                    .enumerate()
                    .map(|(i, brush)| (i, brush.brush_type))
                    .collect();
                let active_index = self.active_brush_index;
                
                for (i, brush_type) in brush_types {
                    let (rect, response) = ui.allocate_exact_size(cell_size, egui::Sense::click());
                    
                    let is_active = i == active_index;
                    ui.painter().rect(
                        rect,
                        4.0,
                        if is_active {
                            ui.style().visuals.selection.bg_fill
                        } else if response.hovered() {
                            ui.style().visuals.widgets.hovered.bg_fill
                        } else {
                            ui.style().visuals.widgets.inactive.bg_fill
                        },
                        if is_active {
                            egui::Stroke::new(2.0, ui.style().visuals.selection.stroke.color)
                        } else {
                            egui::Stroke::NONE
                        }
                    );
                    
                    let preview_size = Vec2::new(48.0, 48.0);
                    let preview_rect = Rect::from_center_size(
                        Pos2::new(rect.center().x, rect.min.y + preview_size.y * 0.5 + 8.0),
                        preview_size
                    );
                    
                    self.draw_brush_preview(ui, preview_rect, &brush_type);
                    
                    let text_pos = Pos2::new(rect.center().x, rect.max.y - 12.0);
                    let text_rect = Rect::from_center_size(text_pos, Vec2::new(cell_size.x - 4.0, 16.0));
                    
                    ui.painter().rect_filled(
                        text_rect,
                        2.0,
                        egui::Color32::from_black_alpha(120)
                    );
                    
                    ui.painter().text(
                        text_pos,
                        egui::Align2::CENTER_CENTER,
                        brush_type.get_name(language),
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                    
                    if response.clicked() {
                        self.active_brush_index = i;
                        changed = true;
                    }
                    
                    col += 1;
                    if col >= columns {
                        col = 0;
                        ui.end_row();
                    }
                }
                
                if col > 0 && col < columns {
                    for _ in col..columns {
                        ui.add_space(cell_size.x);
                    }
                }
            });
        
        changed
    }
    
    fn draw_brush_preview(&self, ui: &mut egui::Ui, rect: egui::Rect, brush_type: &BrushType) {
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, Color32::from_gray(240));
        
        match brush_type {
            BrushType::Round => {
                let center = rect.center();
                let radius = rect.width() * 0.25;
                painter.circle_filled(center, radius, Color32::BLACK);
            },
            BrushType::Flat => {
                let center = rect.center();
                let width = rect.width() * 0.7;
                let height = rect.height() * 0.1;
                let rect = Rect::from_center_size(center, Vec2::new(width, height));
                painter.rect_filled(rect, 1.0, Color32::BLACK);
            },
            BrushType::Bright => {
                let center = rect.center();
                let width = rect.width() * 0.5;
                let height = rect.height() * 0.15;
                let rect = Rect::from_center_size(center, Vec2::new(width, height));
                painter.rect_filled(rect, 1.0, Color32::BLACK);
            },
            BrushType::Filbert => {
                let center = rect.center();
                let width = rect.width() * 0.5;
                let height = rect.height() * 0.3;
                let rect = Rect::from_center_size(center, Vec2::new(width, height));
                painter.rect_filled(rect, 12.0, Color32::BLACK);
            },
            BrushType::Fan => {
                let center = rect.center();
                let radius = rect.width() * 0.3;
                let angles = [-25.0, -12.0, 0.0, 12.0, 25.0];
                
                for (i, angle_deg) in angles.iter().enumerate() {
                    let angle = (*angle_deg as f32).to_radians();
                    let length = radius * (0.8 + 0.2 * (2.0 - i as f32).abs() / 2.0);
                    let dir_x = angle.sin() * length;
                    let dir_y = -angle.cos() * length;
                    
                    let stroke_width = if i == 2 { 2.5 } else { 1.5 };
                    
                    painter.line_segment(
                        [center, Pos2::new(center.x + dir_x, center.y + dir_y)],
                        Stroke::new(stroke_width, Color32::BLACK)
                    );
                }
            },
            BrushType::Angle => {
                let center = rect.center();
                let size = rect.width() * 0.15;
                
                let points = vec![
                    Pos2::new(center.x - size * 2.0, center.y),
                    Pos2::new(center.x + size * 0.8, center.y - size * 1.5),
                    Pos2::new(center.x + size * 0.8, center.y + size * 1.5),
                ];
                
                painter.add(egui::Shape::convex_polygon(
                    points,
                    Color32::BLACK,
                    Stroke::NONE,
                ));
            },
            BrushType::Mop => {
                let center = rect.center();
                let radius = rect.width() * 0.35;
                
                for r in (1..=8).rev() {
                    let alpha = (r as f32 / 8.0 * 180.0) as u8;
                    let color = Color32::from_rgba_unmultiplied(0, 0, 0, alpha);
                    let r_scaled = radius * (r as f32 / 8.0);
                    painter.circle_filled(center, r_scaled, color);
                }
            },
            BrushType::Rigger => {
                let center = rect.center();
                let height = rect.height() * 0.8;
                
                painter.line_segment(
                    [
                        Pos2::new(center.x, center.y - height * 0.5),
                        Pos2::new(center.x, center.y + height * 0.5)
                    ],
                    Stroke::new(1.5, Color32::BLACK)
                );
            },
        }
    }
}
