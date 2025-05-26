use eframe::egui;
use egui::{Color32, Vec2, RichText, Pos2, Rect, Stroke, TextureHandle, TextureOptions};
use crate::localization::{Language, get_text};
use crate::ui_theme::RustiqueTheme;

pub enum MenuAction {
    NewCanvas(u32, u32),
    OpenFile,
}

pub enum MenuResult {
    Action(MenuAction),
    LanguageChanged(Language),
}

#[derive(Clone, Copy)]
enum ScreenSize {
    Mobile,
    Tablet, 
    Desktop,
}

impl ScreenSize {
    fn from_width(width: f32) -> Self {
        if width < 600.0 {
            Self::Mobile
        } else if width < 1000.0 {
            Self::Tablet
        } else {
            Self::Desktop
        }
    }
}

pub struct MainMenu {
    width: u32,
    height: u32,
    logo: Option<egui::TextureHandle>,
    background: Option<egui::TextureHandle>,
    language: Language,
}

impl MainMenu {
    pub fn new(language: Language) -> Self {
        Self {
            width: 800,
            height: 600,
            logo: None,
            background: None,
            language,
        }
    }
    
    pub fn show(&mut self, ctx: &egui::Context) -> Option<MenuResult> {
        RustiqueTheme::apply_theme(ctx);
        
        let mut result = None;
        let screen_size = ScreenSize::from_width(ctx.screen_rect().width());
        
        if self.logo.is_none() {
            self.logo = self.load_logo(ctx);
        }
        if self.background.is_none() {
            self.background = self.load_background(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(RustiqueTheme::BACKGROUND_PRIMARY))
            .show(ctx, |ui| {
                let screen_rect = ui.max_rect();
                
                self.draw_background(ui, screen_rect);
                
                match screen_size {
                    ScreenSize::Mobile => {
                        result = self.show_mobile_layout(ui, screen_rect);
                    },
                    ScreenSize::Tablet => {
                        result = self.show_tablet_layout(ui, screen_rect);
                    },
                    ScreenSize::Desktop => {
                        result = self.show_desktop_layout(ui, screen_rect);
                    },
                }
            });

        result
    }
    
    fn show_mobile_layout(&mut self, ui: &mut egui::Ui, screen_rect: Rect) -> Option<MenuResult> {
        let mut result = None;
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.allocate_ui_at_rect(
                    Rect::from_min_size(screen_rect.min, Vec2::new(screen_rect.width(), f32::MAX)),
                    |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            
                            self.draw_title_section_mobile(ui);
                            ui.add_space(30.0);
                            
                            if let Some(action_result) = self.draw_action_buttons_mobile(ui) {
                                result = Some(action_result);
                            }
                            ui.add_space(20.0);
                            
                            self.draw_canvas_section_mobile(ui, &mut result);
                            ui.add_space(30.0);
                            
                            if let Some(lang_result) = self.draw_language_section_mobile(ui) {
                                result = Some(lang_result);
                            }
                            ui.add_space(20.0);
                        });
                    }
                );
            });
        
        result
    }
    
    fn show_tablet_layout(&mut self, ui: &mut egui::Ui, _screen_rect: Rect) -> Option<MenuResult> {
        let mut result = None;
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    
                    self.draw_title_section_tablet(ui);
                    ui.add_space(40.0);
                    
                    if let Some(action_result) = self.draw_action_buttons_tablet(ui) {
                        result = Some(action_result);
                    }
                    ui.add_space(30.0);
                    
                    self.draw_canvas_section_tablet(ui, &mut result);
                    ui.add_space(40.0);
                    
                    if let Some(lang_result) = self.draw_language_section_tablet(ui) {
                        result = Some(lang_result);
                    }
                    ui.add_space(30.0);
                });
            });
        
        result
    }
    
    fn show_desktop_layout(&mut self, ui: &mut egui::Ui, _screen_rect: Rect) -> Option<MenuResult> {
        let mut result = None;
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    
                    self.draw_title_section(ui);
                    ui.add_space(50.0);
                    
                    if let Some(action_result) = self.draw_action_buttons(ui) {
                        result = Some(action_result);
                    }
                    ui.add_space(40.0);
                    
                    self.draw_canvas_section(ui, &mut result);
                    ui.add_space(50.0);
                    
                    if let Some(lang_result) = self.draw_language_section(ui) {
                        result = Some(lang_result);
                    }
                    ui.add_space(30.0);
                });
            });
        
        result
    }
    
    fn draw_title_section_mobile(&mut self, ui: &mut egui::Ui) {
        if let Some(logo) = &self.logo {
            let max_width = ui.available_width() * 0.8;
            let logo_size = Vec2::new(max_width.min(260.0), max_width.min(260.0) * 0.35);
            ui.image(logo, logo_size);
        } else {
            ui.label(
                RichText::new("Rustique")
                    .size(32.0)
                    .color(Color32::WHITE)
                    .strong()
            );
            ui.add_space(5.0);
            ui.label(
                RichText::new(get_text("modern_paint_application", self.language))
                    .size(14.0)
                    .color(Color32::from_gray(200))
            );
        }
    }
    
    fn draw_title_section_tablet(&mut self, ui: &mut egui::Ui) {
        if let Some(logo) = &self.logo {
            let max_width = ui.available_width() * 0.6;
            let logo_size = Vec2::new(max_width.min(280.0), max_width.min(280.0) * 0.35);
            ui.image(logo, logo_size);
        } else {
            ui.label(
                RichText::new("Rustique")
                    .size(42.0)
                    .color(Color32::WHITE)
                    .strong()
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(get_text("modern_paint_application", self.language))
                    .size(16.0)
                    .color(Color32::from_gray(200))
            );
        }
    }
    
    fn draw_action_buttons_mobile(&mut self, ui: &mut egui::Ui) -> Option<MenuResult> {
        let mut result = None;
        let button_width = ui.available_width() * 0.8;
        let button_height = 45.0;
        
        if ui.add(
            egui::Button::new(
                RichText::new(&format!("📄 {}", get_text("new_file", self.language)))
                    .size(16.0)
                    .color(Color32::WHITE)
                    .strong()
            )
            .fill(RustiqueTheme::ACCENT_PRIMARY)
            .stroke(Stroke::new(2.0, RustiqueTheme::ACCENT_PRIMARY))
            .rounding(RustiqueTheme::rounding_medium())
            .min_size(Vec2::new(button_width, button_height))
        ).clicked() {
            result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
        }
        
        ui.add_space(15.0);
        
        if ui.add(
            egui::Button::new(
                RichText::new(&format!("📂 {}", get_text("open_project", self.language)))
                    .size(16.0)
                    .color(RustiqueTheme::TEXT_PRIMARY)
                    .strong()
            )
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 30))
            .stroke(Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 100)))
            .rounding(RustiqueTheme::rounding_medium())
            .min_size(Vec2::new(button_width, button_height))
        ).clicked() {
            result = Some(MenuResult::Action(MenuAction::OpenFile));
        }
        
        result
    }
    
    fn draw_action_buttons_tablet(&mut self, ui: &mut egui::Ui) -> Option<MenuResult> {
        let mut result = None;
        let available_width = ui.available_width();
        let button_width = (available_width * 0.35).min(160.0);
        let spacing = 20.0;
        
        ui.horizontal(|ui| {
            let total_width = button_width * 2.0 + spacing;
            let padding = (available_width - total_width) / 2.0;
            
            if padding > 0.0 {
                ui.add_space(padding);
            }
            
            if ui.add(
                egui::Button::new(
                    RichText::new(&format!("📄 {}", get_text("new_file", self.language)))
                        .size(15.0)
                        .color(Color32::WHITE)
                        .strong()
                )
                .fill(RustiqueTheme::ACCENT_PRIMARY)
                .stroke(Stroke::new(2.0, RustiqueTheme::ACCENT_PRIMARY))
                .rounding(RustiqueTheme::rounding_medium())
                .min_size(Vec2::new(button_width, 45.0))
            ).clicked() {
                result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
            }
            
            ui.add_space(spacing);
            
            if ui.add(
                egui::Button::new(
                    RichText::new(&format!("📂 {}", get_text("open_project", self.language)))
                        .size(15.0)
                        .color(RustiqueTheme::TEXT_PRIMARY)
                        .strong()
                )
                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 30))
                .stroke(Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 100)))
                .rounding(RustiqueTheme::rounding_medium())
                .min_size(Vec2::new(button_width, 45.0))
            ).clicked() {
                result = Some(MenuResult::Action(MenuAction::OpenFile));
            }
        });
        
        result
    }
    
    fn draw_canvas_section_mobile(&mut self, ui: &mut egui::Ui, result: &mut Option<MenuResult>) {
        let panel_width = ui.available_width() * 0.95;
        
        ui.allocate_ui_with_layout(
            Vec2::new(panel_width, 280.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let panel_rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    panel_rect,
                    RustiqueTheme::rounding_large(),
                    Color32::from_rgba_unmultiplied(0, 0, 0, 120)
                );
                
                ui.add_space(15.0);
                
                ui.label(
                    RichText::new(get_text("canvas_settings", self.language))
                        .size(18.0)
                        .color(Color32::WHITE)
                        .strong()
                );
                
                ui.add_space(20.0);
                
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(&format!("{}:", get_text("canvas_dimensions", self.language)))
                            .size(14.0)
                            .color(Color32::from_gray(220))
                    );
                    
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        let field_width = (panel_width - 60.0) / 2.0;
                        
                        ui.add_sized(
                            Vec2::new(field_width, 28.0),
                            egui::DragValue::new(&mut self.width)
                                .speed(1)
                                .clamp_range(100..=4000)
                                .suffix(" px")
                        );
                        
                        ui.add_space(10.0);
                        
                        ui.add_sized(
                            Vec2::new(field_width, 28.0),
                            egui::DragValue::new(&mut self.height)
                                .speed(1)
                                .clamp_range(100..=4000)
                                .suffix(" px")
                        );
                    });
                });
                
                ui.add_space(15.0);
                
                self.draw_preset_buttons_mobile(ui, panel_width);
                
                ui.add_space(20.0);
                
                ui.vertical_centered(|ui| {
                    let button_width = panel_width * 0.8;
                    if ui.add(
                        egui::Button::new(
                            RichText::new(&format!("🎨 {}", get_text("create_new_canvas", self.language)))
                                .size(16.0)
                                .color(Color32::WHITE)
                                .strong()
                        )
                        .fill(RustiqueTheme::ACCENT_PRIMARY)
                        .stroke(Stroke::new(2.0, RustiqueTheme::ACCENT_HOVER))
                        .rounding(RustiqueTheme::rounding_medium())
                        .min_size(Vec2::new(button_width, 40.0))
                    ).clicked() {
                        *result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
                    }
                });
                
                ui.add_space(15.0);
            }
        );
    }
    
    fn draw_preset_buttons_mobile(&mut self, ui: &mut egui::Ui, panel_width: f32) {
        let button_width = (panel_width - 45.0) / 2.0;
        let button_height = 32.0;
        
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new(
                        RichText::new("HD")
                            .size(12.0)
                            .color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                    .rounding(RustiqueTheme::rounding_small())
                    .min_size(Vec2::new(button_width, button_height))
                ).clicked() {
                    self.width = 1280;
                    self.height = 720;
                }
                
                ui.add_space(15.0);
                
                if ui.add(
                    egui::Button::new(
                        RichText::new("FHD")
                            .size(12.0)
                            .color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                    .rounding(RustiqueTheme::rounding_small())
                    .min_size(Vec2::new(button_width, button_height))
                ).clicked() {
                    self.width = 1920;
                    self.height = 1080;
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new(
                        RichText::new("4K")
                            .size(12.0)
                            .color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                    .rounding(RustiqueTheme::rounding_small())
                    .min_size(Vec2::new(button_width, button_height))
                ).clicked() {
                    self.width = 3840;
                    self.height = 2160;
                }
                
                ui.add_space(15.0);
                
                if ui.add(
                    egui::Button::new(
                        RichText::new("Square")
                            .size(12.0)
                            .color(Color32::WHITE)
                    )
                    .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                    .rounding(RustiqueTheme::rounding_small())
                    .min_size(Vec2::new(button_width, button_height))
                ).clicked() {
                    self.width = 1024;
                    self.height = 1024;
                }
            });
        });
    }
    
    fn draw_canvas_section_tablet(&mut self, ui: &mut egui::Ui, result: &mut Option<MenuResult>) {
        let panel_width = (ui.available_width() * 0.8).min(500.0);
        
        ui.allocate_ui_with_layout(
            Vec2::new(panel_width, 240.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let panel_rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    panel_rect,
                    RustiqueTheme::rounding_large(),
                    Color32::from_rgba_unmultiplied(0, 0, 0, 120)
                );
                
                ui.add_space(18.0);
                
                ui.label(
                    RichText::new(get_text("canvas_settings", self.language))
                        .size(19.0)
                        .color(Color32::WHITE)
                        .strong()
                );
                
                ui.add_space(18.0);
                
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let total_content_width = 280.0;
                    let available_width = ui.available_width();
                    let padding = (available_width - total_content_width) / 2.0;
                    
                    if padding > 0.0 {
                        ui.add_space(padding);
                    }
                    
                    ui.label(
                        RichText::new(&format!("{}:", get_text("canvas_dimensions", self.language)))
                            .size(14.0)
                            .color(Color32::from_gray(220))
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.add_sized(
                        Vec2::new(85.0, 26.0),
                        egui::DragValue::new(&mut self.width)
                            .speed(1)
                            .clamp_range(100..=4000)
                            .suffix(" px")
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.label(
                        RichText::new("×")
                            .size(16.0)
                            .color(Color32::from_gray(180))
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.add_sized(
                        Vec2::new(85.0, 26.0),
                        egui::DragValue::new(&mut self.height)
                            .speed(1)
                            .clamp_range(100..=4000)
                            .suffix(" px")
                    );
                });
                
                ui.add_space(15.0);
                
                self.draw_preset_buttons_tablet(ui, panel_width);
                
                ui.add_space(18.0);
                
                ui.vertical_centered(|ui| {
                    if ui.add(
                        egui::Button::new(
                            RichText::new(&format!("🎨 {}", get_text("create_new_canvas", self.language)))
                                .size(17.0)
                                .color(Color32::WHITE)
                                .strong()
                        )
                        .fill(RustiqueTheme::ACCENT_PRIMARY)
                        .stroke(Stroke::new(2.0, RustiqueTheme::ACCENT_HOVER))
                        .rounding(RustiqueTheme::rounding_medium())
                        .min_size(Vec2::new(200.0, 42.0))
                    ).clicked() {
                        *result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
                    }
                });
                
                ui.add_space(18.0);
            }
        );
    }
    
    fn draw_preset_buttons_tablet(&mut self, ui: &mut egui::Ui, _panel_width: f32) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let preset_width = 70.0;
            let spacing = 10.0;
            let num_buttons = 4;
            let total_buttons_width = (preset_width * num_buttons as f32) + (spacing * (num_buttons - 1) as f32);
            let available_width = ui.available_width();
            let padding = (available_width - total_buttons_width) / 2.0;
            
            if padding > 0.0 {
                ui.add_space(padding);
            }
            
            let preset_size = Vec2::new(preset_width, 30.0);
            
            if ui.add(
                egui::Button::new(
                    RichText::new("HD")
                        .size(13.0)
                        .color(Color32::WHITE)
                )
                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                .rounding(RustiqueTheme::rounding_small())
                .min_size(preset_size)
            ).clicked() {
                self.width = 1280;
                self.height = 720;
            }
            
            ui.add_space(spacing);
            
            if ui.add(
                egui::Button::new(
                    RichText::new("FHD")
                        .size(13.0)
                        .color(Color32::WHITE)
                )
                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                .rounding(RustiqueTheme::rounding_small())
                .min_size(preset_size)
            ).clicked() {
                self.width = 1920;
                self.height = 1080;
            }
            
            ui.add_space(spacing);
            
            if ui.add(
                egui::Button::new(
                    RichText::new("4K")
                        .size(13.0)
                        .color(Color32::WHITE)
                )
                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                .rounding(RustiqueTheme::rounding_small())
                .min_size(preset_size)
            ).clicked() {
                self.width = 3840;
                self.height = 2160;
            }
            
            ui.add_space(spacing);
            
            if ui.add(
                egui::Button::new(
                    RichText::new("Square")
                        .size(13.0)
                        .color(Color32::WHITE)
                )
                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                .rounding(RustiqueTheme::rounding_small())
                .min_size(preset_size)
            ).clicked() {
                self.width = 1024;
                self.height = 1024;
            }
        });
    }
    
    fn draw_language_section_mobile(&mut self, ui: &mut egui::Ui) -> Option<MenuResult> {
        let mut result = None;
        
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(&format!("{}:", get_text("language", self.language)))
                    .size(14.0)
                    .color(Color32::from_gray(180))
            );
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                let button_width = (ui.available_width() - 15.0) / 2.0;
                
                let is_french = matches!(self.language, Language::French);
                if ui.add(
                    egui::Button::new("🇫🇷 Français")
                        .fill(if is_french { RustiqueTheme::ACCENT_PRIMARY } else { Color32::TRANSPARENT })
                        .stroke(Stroke::new(1.0, if is_french { RustiqueTheme::ACCENT_PRIMARY } else { Color32::from_gray(100) }))
                        .min_size(Vec2::new(button_width, 28.0))
                ).clicked() {
                    self.language = Language::French;
                    result = Some(MenuResult::LanguageChanged(Language::French));
                }
                
                ui.add_space(15.0);
                
                let is_english = matches!(self.language, Language::English);
                if ui.add(
                    egui::Button::new("🇺🇸 English")
                        .fill(if is_english { RustiqueTheme::ACCENT_PRIMARY } else { Color32::TRANSPARENT })
                        .stroke(Stroke::new(1.0, if is_english { RustiqueTheme::ACCENT_PRIMARY } else { Color32::from_gray(100) }))
                        .min_size(Vec2::new(button_width, 28.0))
                ).clicked() {
                    self.language = Language::English;
                    result = Some(MenuResult::LanguageChanged(Language::English));
                }
            });
        });
        
        result
    }
    
    fn draw_language_section_tablet(&mut self, ui: &mut egui::Ui) -> Option<MenuResult> {
        let mut result = None;
        
        ui.horizontal(|ui| {
            let total_width = 200.0;
            let available_width = ui.available_width();
            let padding = (available_width - total_width) / 2.0;
            
            if padding > 0.0 {
                ui.add_space(padding);
            }
            
            ui.label(
                RichText::new(&format!("{}:", get_text("language", self.language)))
                    .size(14.0)
                    .color(Color32::from_gray(180))
            );
            
            ui.add_space(10.0);
            
            let is_french = matches!(self.language, Language::French);
            if ui.add(
                egui::Button::new("🇫🇷 Français")
                    .fill(if is_french { RustiqueTheme::ACCENT_PRIMARY } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(1.0, if is_french { RustiqueTheme::ACCENT_PRIMARY } else { Color32::from_gray(100) }))
                    .min_size(Vec2::new(90.0, 26.0))
            ).clicked() {
                self.language = Language::French;
                result = Some(MenuResult::LanguageChanged(Language::French));
            }
            
            ui.add_space(5.0);
            
            let is_english = matches!(self.language, Language::English);
            if ui.add(
                egui::Button::new("🇺🇸 English")
                    .fill(if is_english { RustiqueTheme::ACCENT_PRIMARY } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(1.0, if is_english { RustiqueTheme::ACCENT_PRIMARY } else { Color32::from_gray(100) }))
                    .min_size(Vec2::new(90.0, 26.0))
            ).clicked() {
                self.language = Language::English;
                result = Some(MenuResult::LanguageChanged(Language::English));
            }
        });
        
        result
    }
    
    fn load_background(&self, ctx: &egui::Context) -> Option<TextureHandle> {
        if let Ok(image) = image::open("background.png") {
            let image_buffer = image.to_rgba8();
            let size = [image_buffer.width() as _, image_buffer.height() as _];
            let image_data = egui::ColorImage::from_rgba_unmultiplied(
                size, 
                image_buffer.as_flat_samples().as_slice()
            );
            Some(ctx.load_texture("background", image_data, TextureOptions::LINEAR))
        } else {
            None
        }
    }
    
    fn draw_background(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter();
        
        if let Some(bg_texture) = &self.background {
            painter.image(
                bg_texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE
            );
        } else {
            painter.rect_filled(rect, 0.0, RustiqueTheme::BACKGROUND_PRIMARY);
            
            let gradient_rect = Rect::from_min_size(
                rect.center() - Vec2::new(300.0, 200.0),
                Vec2::new(600.0, 400.0)
            );
            
            for i in 0..50 {
                let t = i as f32 / 50.0;
                let alpha = (30.0 * (1.0 - t * t)) as u8;
                let color = Color32::from_rgba_unmultiplied(138, 101, 255, alpha);
                let radius = 300.0 * t;
                
                painter.circle_filled(gradient_rect.center(), radius, color);
            }
        }
    }
    
    fn draw_title_section(&mut self, ui: &mut egui::Ui) {
        if let Some(logo) = &self.logo {
            let logo_size = Vec2::new(360.0, 120.0);
            ui.image(logo, logo_size);
        } else {
            ui.label(
                RichText::new("Rustique")
                    .size(56.0)
                    .color(Color32::WHITE)
                    .strong()
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(get_text("modern_paint_application", self.language))
                    .size(18.0)
                    .color(Color32::from_gray(200))
            );
        }
    }
    
    fn draw_action_buttons(&mut self, ui: &mut egui::Ui) -> Option<MenuResult> {
        let mut result = None;
        
        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let button_width = 180.0;
            let spacing = 30.0;
            let total_width = button_width * 2.0 + spacing;
            let padding = (available_width - total_width) / 2.0;
            
            ui.add_space(padding.max(0.0));
            
            if ui.add(
                egui::Button::new(
                    RichText::new(&format!("📄 {}", get_text("new_file", self.language)))
                        .size(16.0)
                        .color(Color32::WHITE)
                        .strong()
                )
                .fill(RustiqueTheme::ACCENT_PRIMARY)
                .stroke(Stroke::new(2.0, RustiqueTheme::ACCENT_PRIMARY))
                .rounding(RustiqueTheme::rounding_medium())
                .min_size(Vec2::new(button_width, 50.0))
            ).clicked() {
                result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
            }
            
            ui.add_space(spacing);
            
            if ui.add(
                egui::Button::new(
                    RichText::new(&format!("📂 {}", get_text("open_project", self.language)))
                        .size(16.0)
                        .color(RustiqueTheme::TEXT_PRIMARY)
                        .strong()
                )
                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 30))
                .stroke(Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 100)))
                .rounding(RustiqueTheme::rounding_medium())
                .min_size(Vec2::new(button_width, 50.0))
            ).clicked() {
                result = Some(MenuResult::Action(MenuAction::OpenFile));
            }
        });
        
        result
    }
    
    fn draw_canvas_section(&mut self, ui: &mut egui::Ui, result: &mut Option<MenuResult>) {
        ui.allocate_ui_with_layout(
            Vec2::new(450.0, 220.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let panel_rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    panel_rect,
                    RustiqueTheme::rounding_large(),
                    Color32::from_rgba_unmultiplied(0, 0, 0, 120)
                );
                
                ui.add_space(20.0);
                
                ui.label(
                    RichText::new(get_text("canvas_settings", self.language))
                        .size(20.0)
                        .color(Color32::WHITE)
                        .strong()
                );
                
                ui.add_space(20.0);
                
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let total_content_width = 250.0;
                    let available_width = ui.available_width();
                    let padding = (available_width - total_content_width) / 2.0;
                    
                    ui.add_space(padding.max(0.0));
                    
                    ui.label(
                        RichText::new(&format!("{}:", get_text("canvas_dimensions", self.language)))
                            .size(14.0)
                            .color(Color32::from_gray(220))
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.add_sized(
                        Vec2::new(80.0, 24.0),
                        egui::DragValue::new(&mut self.width)
                            .speed(1)
                            .clamp_range(100..=4000)
                            .suffix(" px")
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.label(
                        RichText::new("×")
                            .size(16.0)
                            .color(Color32::from_gray(180))
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.add_sized(
                        Vec2::new(80.0, 24.0),
                        egui::DragValue::new(&mut self.height)
                            .speed(1)
                            .clamp_range(100..=4000)
                            .suffix(" px")
                    );
                });
                
                ui.add_space(15.0);
                
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let preset_width = 65.0;
                    let spacing = 8.0;
                    let num_buttons = 4;
                    let total_buttons_width = (preset_width * num_buttons as f32) + (spacing * (num_buttons - 1) as f32);
                    let available_width = ui.available_width();
                    let padding = (available_width - total_buttons_width) / 2.0;
                    
                    ui.add_space(padding.max(0.0));
                    
                    let preset_size = Vec2::new(preset_width, 28.0);
                    
                    if ui.add(
                        egui::Button::new(
                            RichText::new("HD")
                                .size(13.0)
                                .color(Color32::WHITE)
                        )
                        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                        .rounding(RustiqueTheme::rounding_small())
                        .min_size(preset_size)
                    ).clicked() {
                        self.width = 1280;
                        self.height = 720;
                    }
                    
                    ui.add_space(spacing);
                    
                    if ui.add(
                        egui::Button::new(
                            RichText::new("FHD")
                                .size(13.0)
                                .color(Color32::WHITE)
                        )
                        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                        .rounding(RustiqueTheme::rounding_small())
                        .min_size(preset_size)
                    ).clicked() {
                        self.width = 1920;
                        self.height = 1080;
                    }
                    
                    ui.add_space(spacing);
                    
                    if ui.add(
                        egui::Button::new(
                            RichText::new("4K")
                                .size(13.0)
                                .color(Color32::WHITE)
                        )
                        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                        .rounding(RustiqueTheme::rounding_small())
                        .min_size(preset_size)
                    ).clicked() {
                        self.width = 3840;
                        self.height = 2160;
                    }
                    
                    ui.add_space(spacing);
                    
                    if ui.add(
                        egui::Button::new(
                            RichText::new("Square")
                                .size(13.0)
                                .color(Color32::WHITE)
                        )
                        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 20))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60)))
                        .rounding(RustiqueTheme::rounding_small())
                        .min_size(preset_size)
                    ).clicked() {
                        self.width = 1024;
                        self.height = 1024;
                    }
                });
                
                ui.add_space(20.0);
                
                ui.vertical_centered(|ui| {
                    if ui.add(
                        egui::Button::new(
                            RichText::new(&format!("🎨 {}", get_text("create_new_canvas", self.language)))
                                .size(18.0)
                                .color(Color32::WHITE)
                                .strong()
                        )
                        .fill(RustiqueTheme::ACCENT_PRIMARY)
                        .stroke(Stroke::new(2.0, RustiqueTheme::ACCENT_HOVER))
                        .rounding(RustiqueTheme::rounding_medium())
                        .min_size(Vec2::new(220.0, 45.0))
                    ).clicked() {
                        *result = Some(MenuResult::Action(MenuAction::NewCanvas(self.width, self.height)));
                    }
                });
                
                ui.add_space(20.0);
            }
        );
    }
    
    fn draw_language_section(&mut self, ui: &mut egui::Ui) -> Option<MenuResult> {
        let mut result = None;
        
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&format!("{}:", get_text("language", self.language)))
                    .size(14.0)
                    .color(Color32::from_gray(180))
            );
            
            ui.add_space(10.0);
            
            let is_french = matches!(self.language, Language::French);
            if ui.add(
                egui::Button::new("🇫🇷 Français")
                    .fill(if is_french { RustiqueTheme::ACCENT_PRIMARY } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(1.0, if is_french { RustiqueTheme::ACCENT_PRIMARY } else { Color32::from_gray(100) }))
                    .min_size(Vec2::new(90.0, 25.0))
            ).clicked() {
                self.language = Language::French;
                result = Some(MenuResult::LanguageChanged(Language::French));
            }
            
            ui.add_space(5.0);
            
            let is_english = matches!(self.language, Language::English);
            if ui.add(
                egui::Button::new("🇺🇸 English")
                    .fill(if is_english { RustiqueTheme::ACCENT_PRIMARY } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(1.0, if is_english { RustiqueTheme::ACCENT_PRIMARY } else { Color32::from_gray(100) }))
                    .min_size(Vec2::new(90.0, 25.0))
            ).clicked() {
                self.language = Language::English;
                result = Some(MenuResult::LanguageChanged(Language::English));
            }
        });
        
        result
    }
    
    fn load_logo(&self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        match image::open("rustique.png") {
            Ok(image) => {
                let image_buffer = image.to_rgba8();
                let size = [image_buffer.width() as _, image_buffer.height() as _];
                let image_data = egui::ColorImage::from_rgba_unmultiplied(
                    size, 
                    image_buffer.as_flat_samples().as_slice()
                );
                Some(ctx.load_texture("logo", image_data, TextureOptions::LINEAR))
            },
            Err(_) => None
        }
    }
}
