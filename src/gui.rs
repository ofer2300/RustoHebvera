use eframe::egui;
use egui::{Color32, RichText, Stroke, Rounding, Vec2};
use std::sync::Arc;
use std::path::PathBuf;
use rfd::FileDialog;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use crate::templates::TemplateManager;
use crate::template_translator::TemplateTranslator;
use crate::metadata::DocumentMetadata;
use crate::technical_dictionary::{TechnicalDictionary, TechnicalTerm, SearchQuery};
use std::sync::Mutex;

pub struct ModernGui {
    translation_engine: Arc<OptimizedTranslationEngine>,
    input_text: String,
    output_text: String,
    selected_source_lang: String,
    selected_target_lang: String,
    is_processing: bool,
    theme: Theme,
    animation_state: AnimationState,
    quality_report: Option<ValidationReport>,
    suggestions: Vec<String>,
    history: Vec<TranslationRecord>,
    is_dark_mode: bool,
}

struct Theme {
    primary_color: Color32,
    secondary_color: Color32,
    background_color: Color32,
    text_color: Color32,
    accent_color: Color32,
    error_color: Color32,
    success_color: Color32,
}

struct AnimationState {
    progress: f32,
    translation_opacity: f32,
    suggestions_height: f32,
}

impl ModernGui {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = Theme::light();
        
        let translation_engine = Arc::new(OptimizedTranslationEngine::new());
        
        Self {
            translation_engine,
            input_text: String::new(),
            output_text: String::new(),
            selected_source_lang: "he".to_string(),
            selected_target_lang: "ru".to_string(),
            is_processing: false,
            theme,
            animation_state: AnimationState::default(),
            quality_report: None,
            suggestions: Vec::new(),
            history: Vec::new(),
            is_dark_mode: false,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        self.update_animations(ctx.frame_time());
        
        self.theme = if self.is_dark_mode {
            Theme::dark()
        } else {
            Theme::light()
        };
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_header(ui);
            self.draw_main_content(ui);
            self.draw_footer(ui);
        });
        
        if let Some(report) = &self.quality_report {
            self.draw_quality_report(ctx, report);
        }
        
        egui::Window::new("היסטוריה")
            .open(&mut self.show_history)
            .show(ctx, |ui| {
                self.draw_history(ui);
            });
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.image(self.theme.logo_id, Vec2::new(32.0, 32.0));
            
            ui.heading(
                RichText::new("RustoHebru - מתרגם מתקדם")
                    .color(self.theme.primary_color)
                    .size(24.0)
            );
            
            ui.with_layout(egui::Layout::right_to_left(true), |ui| {
                if ui.button("⚙️").clicked() {
                    self.show_settings = true;
                }
                
                ui.toggle_value(&mut self.is_dark_mode, "🌙");
            });
        });
    }

    fn draw_main_content(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.draw_language_selector(ui);
        });
        
        ui.add_space(10.0);
        
        ui.group(|ui| {
            ui.set_height(200.0);
            self.draw_input_area(ui);
        });
        
        let translate_button = ui.add_sized(
            [120.0, 40.0],
            egui::Button::new(
                RichText::new(if self.is_processing { "מתרגם..." } else { "תרגם" })
                    .size(18.0)
            ).fill(self.theme.primary_color)
        );
        
        if translate_button.clicked() && !self.is_processing {
            self.start_translation();
        }
        
        ui.group(|ui| {
            ui.set_height(200.0);
            self.draw_output_area(ui);
        });
        
        if !self.suggestions.is_empty() {
            self.draw_suggestions(ui);
        }
    }

    fn draw_suggestions(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("הצעות שיפור")
            .default_open(true)
            .show(ui, |ui| {
                for suggestion in &self.suggestions {
                    ui.label(
                        RichText::new(suggestion)
                            .color(self.theme.accent_color)
                    );
                }
            });
    }

    fn update_animations(&mut self, delta_time: f32) {
        if self.is_processing {
            self.animation_state.progress = (self.animation_state.progress + delta_time * 2.0) % 1.0;
        }
        
        if !self.output_text.is_empty() {
            self.animation_state.translation_opacity = 
                (self.animation_state.translation_opacity + delta_time * 3.0).min(1.0);
        }
    }
}

impl Theme {
    fn light() -> Self {
        Self {
            primary_color: Color32::from_rgb(63, 81, 181),
            secondary_color: Color32::from_rgb(103, 58, 183),
            background_color: Color32::from_rgb(250, 250, 250),
            text_color: Color32::from_rgb(33, 33, 33),
            accent_color: Color32::from_rgb(0, 150, 136),
            error_color: Color32::from_rgb(244, 67, 54),
            success_color: Color32::from_rgb(76, 175, 80),
        }
    }
    
    fn dark() -> Self {
        Self {
            primary_color: Color32::from_rgb(98, 0, 238),
            secondary_color: Color32::from_rgb(3, 218, 197),
            background_color: Color32::from_rgb(18, 18, 18),
            text_color: Color32::from_rgb(255, 255, 255),
            accent_color: Color32::from_rgb(0, 200, 180),
            error_color: Color32::from_rgb(255, 82, 82),
            success_color: Color32::from_rgb(100, 221, 23),
        }
    }
}

// ... existing code ... 