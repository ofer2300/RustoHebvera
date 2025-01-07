use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use chrono::Utc;
use crate::translation_engine::TranslationEngine;
use crate::quality_control::{QualityControl, IssueSeverity};

pub struct ModernGui {
    translation_engine: Arc<TranslationEngine>,
    quality_control: Arc<QualityControl>,
    source_text: String,
    target_text: String,
    source_lang: String,
    target_lang: String,
}

impl ModernGui {
    pub fn new(translation_engine: Arc<TranslationEngine>, quality_control: Arc<QualityControl>) -> Self {
        Self {
            translation_engine,
            quality_control,
            source_text: String::new(),
            target_text: String::new(),
            source_lang: "he".to_string(),
            target_lang: "ru".to_string(),
        }
    }
}

impl eframe::App for ModernGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("מערכת תרגום טכני");

            ui.horizontal(|ui| {
                ui.label("שפת מקור:");
                ui.radio_value(&mut self.source_lang, "he".to_string(), "עברית");
                ui.radio_value(&mut self.source_lang, "ru".to_string(), "רוסית");
            });

            ui.horizontal(|ui| {
                ui.label("שפת יעד:");
                ui.radio_value(&mut self.target_lang, "ru".to_string(), "רוסית");
                ui.radio_value(&mut self.target_lang, "he".to_string(), "עברית");
            });

            ui.text_edit_multiline(&mut self.source_text);

            if ui.button("תרגם").clicked() {
                let engine = self.translation_engine.clone();
                let source_text = self.source_text.clone();
                let source_lang = self.source_lang.clone();
                let target_lang = self.target_lang.clone();

                tokio::spawn(async move {
                    if let Ok(translated) = engine.translate(&source_text, &source_lang, &target_lang).await {
                        // TODO: עדכון התרגום בממשק
                    }
                });
            }

            ui.text_edit_multiline(&mut self.target_text);
        });
    }
} 