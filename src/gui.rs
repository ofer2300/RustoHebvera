use eframe::{egui, epi};
use egui::{Color32, RichText, TextEdit, Ui};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

use crate::translation_engine::TranslationEngine;
use crate::quality_control::{QualityControl, ValidationReport};
use crate::learning_manager::{LearningManager, LearningEvent, LearningEventType, UserFeedback};
use chrono::Utc;

pub struct ModernGui {
    translation_engine: Arc<TranslationEngine>,
    quality_control: Arc<QualityControl>,
    learning_manager: Arc<LearningManager>,
    
    // מצב הממשק
    source_text: String,
    target_text: String,
    source_language: Language,
    target_language: Language,
    is_processing: bool,
    
    // תוצאות בדיקה
    validation_report: Option<ValidationReport>,
    
    // משוב משתמש
    user_rating: Option<u8>,
    user_comments: String,
    
    // מצב תצוגה
    show_advanced: bool,
    show_statistics: bool,
    dark_mode: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Language {
    Hebrew,
    Russian,
}

impl Default for ModernGui {
    fn default() -> Self {
        Self {
            translation_engine: Arc::new(TranslationEngine::new()),
            quality_control: Arc::new(QualityControl::new()),
            learning_manager: Arc::new(LearningManager::new()),
            
            source_text: String::new(),
            target_text: String::new(),
            source_language: Language::Hebrew,
            target_language: Language::Russian,
            is_processing: false,
            
            validation_report: None,
            
            user_rating: None,
            user_comments: String::new(),
            
            show_advanced: false,
            show_statistics: false,
            dark_mode: false,
        }
    }
}

impl epi::App for ModernGui {
    fn name(&self) -> &str {
        "RustoHebvera - מתרגם טכני"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        self.render_main_panel(ctx);
        
        if self.show_advanced {
            self.render_advanced_panel(ctx);
        }
        
        if self.show_statistics {
            self.render_statistics_panel(ctx);
        }
    }
}

impl ModernGui {
    pub fn new(
        translation_engine: Arc<TranslationEngine>,
        quality_control: Arc<QualityControl>,
        learning_manager: Arc<LearningManager>,
    ) -> Self {
        Self {
            translation_engine,
            quality_control,
            learning_manager,
            ..Default::default()
        }
    }

    fn render_main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // כותרת
            ui.heading("RustoHebvera - מתרגם טכני");
            ui.add_space(20.0);
            
            // בחירת שפות
            self.render_language_selector(ui);
            ui.add_space(10.0);
            
            // אזור טקסט מקור
                    ui.group(|ui| {
                ui.label(RichText::new("טקסט מקור").size(16.0));
                ui.add_sized(
                    [ui.available_width(), 150.0],
                    TextEdit::multiline(&mut self.source_text)
                );
            });
            
            // כפתור תרגום
            if ui.add_enabled(
                !self.is_processing && !self.source_text.is_empty(),
                egui::Button::new("תרגם")
            ).clicked() {
                self.start_translation();
            }
            
            // אזור טקסט מתורגם
            ui.group(|ui| {
                ui.label(RichText::new("תרגום").size(16.0));
                ui.add_sized(
                    [ui.available_width(), 150.0],
                    TextEdit::multiline(&mut self.target_text)
                        .text_color(if self.is_processing {
                            Color32::GRAY
                                } else {
                            Color32::WHITE
                        })
                );
            });
            
            // תוצאות בדיקת איכות
            if let Some(report) = &self.validation_report {
                self.render_validation_results(ui, report);
            }
            
            // משוב משתמש
            self.render_feedback_section(ui);
            
            // כפתורים נוספים
                            ui.horizontal(|ui| {
                if ui.button("הגדרות מתקדמות").clicked() {
                    self.show_advanced = !self.show_advanced;
                }
                if ui.button("סטטיסטיקות").clicked() {
                    self.show_statistics = !self.show_statistics;
                }
                if ui.button(if self.dark_mode {
                    "מצב בהיר"
                } else {
                    "מצב כהה"
                }).clicked() {
                    self.dark_mode = !self.dark_mode;
                    self.update_theme(ctx);
                }
            });
        });
    }

    fn render_language_selector(&mut self, ui: &mut Ui) {
                ui.horizontal(|ui| {
            ui.label("שפת מקור:");
            ui.selectable_value(&mut self.source_language, Language::Hebrew, "עברית");
            ui.selectable_value(&mut self.source_language, Language::Russian, "רוסית");
            
            ui.separator();
            
            ui.label("שפת יעד:");
            ui.selectable_value(&mut self.target_language, Language::Hebrew, "עברית");
            ui.selectable_value(&mut self.target_language, Language::Russian, "רוסית");
            
            if ui.button("⇄").clicked() {
                std::mem::swap(&mut self.source_language, &mut self.target_language);
                std::mem::swap(&mut self.source_text, &mut self.target_text);
            }
        });
    }

    fn render_validation_results(&mut self, ui: &mut Ui, report: &ValidationReport) {
                ui.group(|ui| {
            ui.label(RichText::new("תוצאות בדיקת איכות").size(16.0));

                ui.horizontal(|ui| {
                ui.label("ציון דקדוק:");
                ui.label(format!("{:.1}%", report.grammar_score * 100.0));
                });

                ui.horizontal(|ui| {
                ui.label("ציון סגנון:");
                ui.label(format!("{:.1}%", report.style_score * 100.0));
                });

                ui.horizontal(|ui| {
                ui.label("ציון מונחים:");
                ui.label(format!("{:.1}%", report.terminology_score * 100.0));
            });
            
            if !report.issues.is_empty() {
                ui.label(RichText::new("בעיות שזוהו:").color(Color32::YELLOW));
                for issue in &report.issues {
                    ui.label(RichText::new(&issue.description)
                        .color(match issue.severity {
                            IssueSeverity::Low => Color32::GRAY,
                            IssueSeverity::Medium => Color32::YELLOW,
                            IssueSeverity::High => Color32::LIGHT_RED,
                            IssueSeverity::Critical => Color32::RED,
                        })
                    );
                }
            }
        });
    }

    fn render_feedback_section(&mut self, ui: &mut Ui) {
                    ui.group(|ui| {
            ui.label(RichText::new("משוב").size(16.0));
            
            // דירוג
                        ui.horizontal(|ui| {
                ui.label("דירוג:");
                for rating in 1..=5 {
                    if ui.selectable_label(
                        self.user_rating == Some(rating),
                        "★"
                    ).clicked() {
                        self.user_rating = Some(rating);
                    }
                }
            });
            
            // הערות
                        ui.label("הערות:");
            ui.text_edit_multiline(&mut self.user_comments);
            
            if ui.button("שלח משוב").clicked() {
                self.submit_feedback();
            }
        });
    }

    fn render_advanced_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("הגדרות מתקדמות")
            .open(&mut self.show_advanced)
            .show(ctx, |ui| {
                // TODO: הוספת הגדרות מתקדמות
            });
    }

    fn render_statistics_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("סטטיסטיקות")
            .open(&mut self.show_statistics)
            .show(ctx, |ui| {
                // TODO: הצגת סטטיסטיקות
            });
    }

    fn start_translation(&mut self) {
        self.is_processing = true;
        self.validation_report = None;
        self.user_rating = None;
        self.user_comments.clear();
        
        // יצירת אירוע תרגום
        let event = LearningEvent {
            timestamp: Utc::now(),
            event_type: LearningEventType::Translation,
            source_text: self.source_text.clone(),
            target_text: String::new(),
            validation_report: None,
            user_feedback: None,
            confidence_score: 0.0,
        };
        
        // הפעלת מנוע התרגום
        let translation_engine = self.translation_engine.clone();
        let quality_control = self.quality_control.clone();
        let learning_manager = self.learning_manager.clone();
        let source_text = self.source_text.clone();
        
        tokio::spawn(async move {
            // תרגום
            let translation = translation_engine.translate(&source_text).await?;
            
            // בדיקת איכות
            let validation = quality_control.validate_deep(&translation).await?;
            
            // עדכון אירוע הלמידה
            let mut event = event;
            event.target_text = translation.clone();
            event.validation_report = Some(validation.clone());
            learning_manager.record_event(event).await?;
            
            Ok::<_, anyhow::Error>((translation, validation))
        });
    }

    fn submit_feedback(&mut self) {
        if let Some(rating) = self.user_rating {
            let feedback = UserFeedback {
                rating,
                comments: if self.user_comments.is_empty() {
                    None
                            } else {
                    Some(self.user_comments.clone())
                },
                corrections: None,
            };
            
            let event = LearningEvent {
                timestamp: Utc::now(),
                event_type: LearningEventType::Feedback,
                source_text: self.source_text.clone(),
                target_text: self.target_text.clone(),
                validation_report: self.validation_report.clone(),
                user_feedback: Some(feedback),
                confidence_score: 1.0,
            };
            
            let learning_manager = self.learning_manager.clone();
            tokio::spawn(async move {
                learning_manager.record_event(event).await?;
                Ok::<_, anyhow::Error>(())
                        });
                    }
                }

    fn update_theme(&self, ctx: &egui::Context) {
        let mut visuals = if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        
        // התאמות נוספות לערכת הנושא
        visuals.window_rounding = 10.0.into();
        visuals.window_shadow.extrusion = 20.0;
        
        ctx.set_visuals(visuals);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_creation() {
        let gui = ModernGui::default();
        assert_eq!(gui.source_language, Language::Hebrew);
        assert_eq!(gui.target_language, Language::Russian);
        assert!(!gui.is_processing);
    }

    #[test]
    fn test_language_swap() {
        let mut gui = ModernGui::default();
        gui.source_text = "שלום".to_string();
        gui.target_text = "привет".to_string();
        
        let original_source = gui.source_language.clone();
        let original_target = gui.target_language.clone();
        
        // החלפת שפות
        std::mem::swap(&mut gui.source_language, &mut gui.target_language);
        std::mem::swap(&mut gui.source_text, &mut gui.target_text);
        
        assert_eq!(gui.source_language, original_target);
        assert_eq!(gui.target_language, original_source);
        assert_eq!(gui.source_text, "привет");
        assert_eq!(gui.target_text, "שלום");
    }
} 