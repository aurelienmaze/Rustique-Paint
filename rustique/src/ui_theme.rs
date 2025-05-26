use eframe::egui;
use egui::{Color32, Stroke, Rounding, Vec2, Margin, RichText, FontId, Style, Visuals};

pub struct RustiqueTheme;

impl RustiqueTheme {
    pub const BACKGROUND_PRIMARY: Color32 = Color32::from_rgb(18, 18, 24);
    pub const BACKGROUND_SECONDARY: Color32 = Color32::from_rgb(24, 24, 32);
    pub const BACKGROUND_TERTIARY: Color32 = Color32::from_rgb(32, 32, 42);
    
    pub const SURFACE_PRIMARY: Color32 = Color32::from_rgb(42, 42, 54);
    pub const SURFACE_SECONDARY: Color32 = Color32::from_rgb(54, 54, 68);
    pub const SURFACE_HOVER: Color32 = Color32::from_rgb(64, 64, 80);
    
    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(138, 101, 255);
    pub const ACCENT_SECONDARY: Color32 = Color32::from_rgb(88, 166, 255);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(158, 121, 255);
    
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 245);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 190);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 140, 150);
    
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(80, 80, 95);
    pub const BORDER_ACCENT: Color32 = Color32::from_rgb(138, 101, 255);
    
    pub const SUCCESS: Color32 = Color32::from_rgb(76, 175, 80);
    pub const WARNING: Color32 = Color32::from_rgb(255, 193, 7);
    pub const ERROR: Color32 = Color32::from_rgb(244, 67, 54);
    
    pub fn rounding_small() -> Rounding { Rounding::same(4.0) }
    pub fn rounding_medium() -> Rounding { Rounding::same(8.0) }
    pub fn rounding_large() -> Rounding { Rounding::same(12.0) }
    
    pub const SPACING_XS: f32 = 4.0;
    pub const SPACING_SM: f32 = 8.0;
    pub const SPACING_MD: f32 = 16.0;
    pub const SPACING_LG: f32 = 24.0;
    pub const SPACING_XL: f32 = 32.0;
    
    pub fn apply_theme(ctx: &egui::Context) {
        let mut style = Style::default();
        let mut visuals = Visuals::dark();
        
        visuals.window_fill = Self::BACKGROUND_PRIMARY;
        visuals.panel_fill = Self::BACKGROUND_SECONDARY;
        visuals.faint_bg_color = Self::SURFACE_PRIMARY;
        visuals.extreme_bg_color = Self::BACKGROUND_TERTIARY;
        
        visuals.widgets.noninteractive.bg_fill = Self::SURFACE_PRIMARY;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_SECONDARY);
        visuals.widgets.noninteractive.rounding = Self::rounding_small();
        
        visuals.widgets.inactive.bg_fill = Self::SURFACE_PRIMARY;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Self::rounding_small();
        
        visuals.widgets.hovered.bg_fill = Self::SURFACE_HOVER;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.rounding = Self::rounding_small();
        
        visuals.widgets.active.bg_fill = Self::ACCENT_PRIMARY;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.active.rounding = Self::rounding_small();
        
        visuals.widgets.open.bg_fill = Self::SURFACE_SECONDARY;
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.open.rounding = Self::rounding_small();
        
        visuals.selection.bg_fill = Self::ACCENT_PRIMARY.linear_multiply(0.8);
        visuals.selection.stroke = Stroke::new(1.0, Self::ACCENT_PRIMARY);
        
        visuals.window_stroke = Stroke::new(1.0, Self::BORDER_LIGHT);
        visuals.window_rounding = Self::rounding_medium();
        
        style.visuals = visuals;
        style.spacing.window_margin = Margin::same(Self::SPACING_MD);
        style.spacing.button_padding = Vec2::new(Self::SPACING_MD, Self::SPACING_SM);
        style.spacing.item_spacing = Vec2::new(Self::SPACING_SM, Self::SPACING_SM);
        style.spacing.indent = Self::SPACING_MD;
        
        ctx.set_style(style);
    }
    
    pub fn primary_button() -> egui::Button {
        egui::Button::new("")
            .fill(Self::ACCENT_PRIMARY)
            .stroke(Stroke::new(1.0, Self::ACCENT_PRIMARY))
            .rounding(Self::rounding_medium())
    }
    
    pub fn secondary_button() -> egui::Button {
        egui::Button::new("")
            .fill(Self::SURFACE_PRIMARY)
            .stroke(Stroke::new(1.0, Self::BORDER_LIGHT))
            .rounding(Self::rounding_medium())
    }
    
    pub fn icon_button() -> egui::Button {
        egui::Button::new("")
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, Self::BORDER_LIGHT))
            .rounding(Self::rounding_small())
    }
    
    pub fn tool_button(active: bool) -> egui::Button {
        if active {
            egui::Button::new("")
                .fill(Self::ACCENT_PRIMARY)
                .stroke(Stroke::new(2.0, Self::ACCENT_PRIMARY))
                .rounding(Self::rounding_small())
        } else {
            egui::Button::new("")
                .fill(Self::SURFACE_PRIMARY)
                .stroke(Stroke::new(1.0, Self::BORDER_LIGHT))
                .rounding(Self::rounding_small())
        }
    }
    
    pub fn heading_text(text: &str, size: f32) -> RichText {
        RichText::new(text)
            .size(size)
            .color(Self::TEXT_PRIMARY)
            .font(FontId::proportional(size))
            .strong()
    }
    
    pub fn body_text(text: &str) -> RichText {
        RichText::new(text)
            .size(14.0)
            .color(Self::TEXT_PRIMARY)
    }
    
    pub fn muted_text(text: &str) -> RichText {
        RichText::new(text)
            .size(12.0)
            .color(Self::TEXT_MUTED)
    }
    
    pub fn accent_text(text: &str) -> RichText {
        RichText::new(text)
            .size(14.0)
            .color(Self::ACCENT_PRIMARY)
            .strong()
    }
    
    pub fn card_frame() -> egui::Frame {
        egui::Frame::none()
            .fill(Self::SURFACE_PRIMARY)
            .stroke(Stroke::new(1.0, Self::BORDER_LIGHT))
            .rounding(Self::rounding_medium())
            .inner_margin(Margin::same(Self::SPACING_MD))
            .shadow(egui::epaint::Shadow {
                extrusion: 4.0,
                color: Color32::from_black_alpha(40),
            })
    }
    
    pub fn panel_frame() -> egui::Frame {
        egui::Frame::none()
            .fill(Self::BACKGROUND_SECONDARY)
            .stroke(Stroke::new(1.0, Self::BORDER_LIGHT))
            .rounding(Self::rounding_small())
            .inner_margin(Margin::same(Self::SPACING_SM))
    }
}
