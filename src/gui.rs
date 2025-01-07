use eframe::egui;
use egui::{Color32, RichText, Ui, Vec2};
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

pub struct RustoHebruApp {
    template_manager: Arc<TemplateManager>,
    template_translator: Arc<TemplateTranslator>,
    technical_dictionary: Arc<Mutex<TechnicalDictionary>>,
    selected_template: Option<String>,
    selected_source_lang: String,
    selected_target_lang: String,
    values: HashMap<String, String>,
    metadata: Option<DocumentMetadata>,
    output_path: Option<PathBuf>,
    status_message: Option<String>,
    dark_mode: bool,
    // שדות חדשים עבור המילון הטכני
    show_dictionary: bool,
    dictionary_search_text: String,
    dictionary_search_lang: String,
    dictionary_search_include_synonyms: bool,
    dictionary_search_exact_match: bool,
    selected_categories: HashSet<String>,
    selected_contexts: HashSet<String>,
    selected_tags: HashSet<String>,
    new_term: TechnicalTerm,
    show_add_term_dialog: bool,
    show_edit_term_dialog: bool,
    editing_term: Option<TechnicalTerm>,
    new_synonym_he: String,
    new_synonym_ru: String,
    new_usage_example: String,
    new_tag: String,
    show_create_version_dialog: bool,
    show_compare_versions_dialog: bool,
    show_change_history_dialog: bool,
    new_version_id: String,
    new_version_description: String,
    selected_version1: Option<String>,
    selected_version2: Option<String>,
    selected_term_for_history: Option<String>,
    show_active_users_dialog: bool,
    show_pending_reviews_dialog: bool,
    show_conflicts_dialog: bool,
    show_review_dialog: bool,
    selected_review: Option<String>,
    selected_resolution_type: ResolutionType,
    conflict_resolution_comments: String,
}

impl RustoHebruApp {
    pub fn new(
        template_manager: Arc<TemplateManager>,
        template_translator: Arc<TemplateTranslator>,
        technical_dictionary: Arc<Mutex<TechnicalDictionary>>,
    ) -> Self {
        Self {
            template_manager,
            template_translator,
            technical_dictionary,
            selected_template: None,
            selected_source_lang: "he".to_string(),
            selected_target_lang: "ru".to_string(),
            values: HashMap::new(),
            metadata: None,
            output_path: None,
            status_message: None,
            dark_mode: false,
            show_dictionary: false,
            dictionary_search_text: String::new(),
            dictionary_search_lang: "he".to_string(),
            dictionary_search_include_synonyms: true,
            dictionary_search_exact_match: false,
            selected_categories: HashSet::new(),
            selected_contexts: HashSet::new(),
            selected_tags: HashSet::new(),
            new_term: TechnicalTerm {
                hebrew: String::new(),
                russian: String::new(),
                context: None,
                category: None,
                notes: None,
                synonyms_he: Vec::new(),
                synonyms_ru: Vec::new(),
                usage_examples: Vec::new(),
                tags: HashSet::new(),
                last_updated: Utc::now(),
            },
            show_add_term_dialog: false,
            show_edit_term_dialog: false,
            editing_term: None,
            new_synonym_he: String::new(),
            new_synonym_ru: String::new(),
            new_usage_example: String::new(),
            new_tag: String::new(),
            show_create_version_dialog: false,
            show_compare_versions_dialog: false,
            show_change_history_dialog: false,
            new_version_id: String::new(),
            new_version_description: String::new(),
            selected_version1: None,
            selected_version2: None,
            selected_term_for_history: None,
            show_active_users_dialog: false,
            show_pending_reviews_dialog: false,
            show_conflicts_dialog: false,
            show_review_dialog: false,
            selected_review: None,
            selected_resolution_type: ResolutionType::KeepBase,
            conflict_resolution_comments: String::new(),
        }
    }

    fn show_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("RustoHebru")
                .color(if self.dark_mode { Color32::WHITE } else { Color32::BLACK })
                .size(24.0));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.dark_mode { "🌞" } else { "🌙" }).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
            });
        });
        ui.separator();
    }

    fn show_template_selector(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("בחירת תבנית");
            ui.horizontal(|ui| {
                if ui.button("טען תבנית").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file() {
                        // טעינת התבנית
                        self.status_message = Some("טוען תבנית...".to_string());
                    }
                }
                
                egui::ComboBox::from_label("תבנית")
                    .selected_text(self.selected_template.clone().unwrap_or_default())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_template,
                            Some("technical_specification".to_string()),
                            "מפרט טכני"
                        );
                        ui.selectable_value(
                            &mut self.selected_template,
                            Some("inspection_report".to_string()),
                            "דוח בדיקה"
                        );
                    });
            });
        });
    }

    fn show_language_selector(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("הגדרות שפה");
            ui.horizontal(|ui| {
                ui.label("שפת מקור:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.selected_source_lang)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_source_lang, "he".to_string(), "עברית");
                        ui.selectable_value(&mut self.selected_source_lang, "ru".to_string(), "רוסית");
                    });
                
                ui.label("שפת יעד:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.selected_target_lang)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_target_lang, "ru".to_string(), "רוסית");
                        ui.selectable_value(&mut self.selected_target_lang, "he".to_string(), "עברית");
                    });
            });
        });
    }

    fn show_template_values(&mut self, ui: &mut Ui) {
        if let Some(template_name) = &self.selected_template {
            if let Some(template) = self.template_manager.get_template(template_name) {
                ui.group(|ui| {
                    ui.heading("ערכי תבנית");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (key, _) in &template.placeholders {
                            let value = self.values.entry(key.clone())
                                .or_insert_with(String::new);
                            ui.horizontal(|ui| {
                                ui.label(key);
                                ui.text_edit_singleline(value);
                            });
                        }
                    });
                });
            }
        }
    }

    fn show_actions(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("פעולות");
            ui.horizontal(|ui| {
                if ui.button("בחר תיקיית פלט").clicked() {
                    if let Some(path) = FileDialog::new()
                        .pick_folder() {
                        self.output_path = Some(path);
                    }
                }
                
                if ui.button("צור מסמך").clicked() {
                    self.create_document();
                }
                
                if ui.button("תרגם מסמך").clicked() {
                    self.translate_document();
                }
            });
        });
    }

    fn show_status(&mut self, ui: &mut Ui) {
        if let Some(message) = &self.status_message {
            ui.horizontal(|ui| {
                ui.label("סטטוס:");
                ui.colored_label(
                    if message.contains("שגיאה") { Color32::RED } else { Color32::GREEN },
                    message
                );
            });
        }
    }

    fn create_document(&mut self) {
        if let Some(template_name) = &self.selected_template {
            match self.template_manager.create_document_from_template(
                template_name,
                &self.values,
                self.metadata.clone(),
            ) {
                Ok(content) => {
                    if let Some(output_path) = &self.output_path {
                        let file_name = format!("{}.txt", template_name);
                        let file_path = output_path.join(file_name);
                        if let Err(e) = std::fs::write(&file_path, content) {
                            self.status_message = Some(format!("שגיאה בשמירת המסמך: {}", e));
                        } else {
                            self.status_message = Some("המסמך נוצר בהצלחה".to_string());
                        }
                    } else {
                        self.status_message = Some("נא לבחור תיקיית פלט".to_string());
                    }
                }
                Err(e) => {
                    self.status_message = Some(format!("שגיאה ביצירת המסמך: {}", e));
                }
            }
        } else {
            self.status_message = Some("נא לבחור תבנית".to_string());
        }
    }

    fn translate_document(&mut self) {
        if let Some(template_name) = &self.selected_template {
            if let Some(template) = self.template_manager.get_template(template_name) {
                match self.template_translator.translate_template(
                    template,
                    &self.selected_source_lang,
                    &self.selected_target_lang,
                ) {
                    Ok(translated_template) => {
                        match self.template_translator.translate_values(
                            &self.values,
                            &self.selected_source_lang,
                            &self.selected_target_lang,
                        ) {
                            Ok(translated_values) => {
                                match self.template_manager.create_document_from_template(
                                    template_name,
                                    &translated_values,
                                    self.metadata.clone(),
                                ) {
                                    Ok(content) => {
                                        if let Some(output_path) = &self.output_path {
                                            let file_name = format!(
                                                "{}_{}.txt",
                                                template_name,
                                                self.selected_target_lang
                                            );
                                            let file_path = output_path.join(file_name);
                                            if let Err(e) = std::fs::write(&file_path, content) {
                                                self.status_message = Some(
                                                    format!("שגיאה בשמירת המסמך המתורגם: {}", e)
                                                );
                                            } else {
                                                self.status_message = Some(
                                                    "המסמך תורגם ונשמר בהצלחה".to_string()
                                                );
                                            }
                                        } else {
                                            self.status_message = Some("נא לבחור תיקיית פלט".to_string());
                                        }
                                    }
                                    Err(e) => {
                                        self.status_message = Some(
                                            format!("שגיאה ביצירת המסמך המתורגם: {}", e)
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                self.status_message = Some(format!("שגיאה בתרגום הערכים: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("שגיאה בתרגום התבנית: {}", e));
                    }
                }
            }
        } else {
            self.status_message = Some("נא לבחור תבנית".to_string());
        }
    }

    fn show_dictionary_panel(&mut self, ui: &mut Ui) {
        if !self.show_dictionary {
            if ui.button("פתח מילון טכני").clicked() {
                self.show_dictionary = true;
            }
            return;
        }

        ui.group(|ui| {
            // כותרת וכפתורים
            ui.horizontal(|ui| {
                ui.heading("מילון טכני");
                
                // כפתורי שיתוף ידע
                if ui.button("צור גרסה חדשה").clicked() {
                    self.show_create_version_dialog = true;
                }
                if ui.button("השווה גרסאות").clicked() {
                    self.show_compare_versions_dialog = true;
                }
                if ui.button("היסטוריית שינויים").clicked() {
                    self.show_change_history_dialog = true;
                }
                
                // כפתורי עבודה משותפת
                if ui.button("משתמשים פעילים").clicked() {
                    self.show_active_users_dialog = true;
                }
                if ui.button("סקירות ממתינות").clicked() {
                    self.show_pending_reviews_dialog = true;
                }
                if ui.button("קונפליקטים").clicked() {
                    self.show_conflicts_dialog = true;
                }
                
                if ui.button("סגור").clicked() {
                    self.show_dictionary = false;
                }
            });

            // תצוגת משתמשים פעילים
            ui.group(|ui| {
                ui.heading("משתמשים פעילים כעת");
                if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                    for collaborator in knowledge_manager.get_active_collaborators() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&collaborator.name).strong());
                            if let Some(activity) = &collaborator.current_activity {
                                let status_text = match activity.activity_type {
                                    ActivityType::Editing => "עורך",
                                    ActivityType::Reviewing => "סוקר",
                                    ActivityType::Comparing => "משווה",
                                    ActivityType::Exporting => "מייצא",
                                };
                                ui.label(format!("- {}", status_text));
                                if let Some(term_id) = &activity.term_id {
                                    ui.label(format!("({})", term_id));
                                }
                            }
                        });
                    }
                }
            });

            // תצוגת סקירות ממתינות
            if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                let pending_reviews = knowledge_manager.get_pending_reviews(&self.current_user_id);
                if !pending_reviews.is_empty() {
                    ui.group(|ui| {
                        ui.heading("סקירות ממתינות");
                        for review in pending_reviews {
                            ui.horizontal(|ui| {
                                ui.label(format!("מונח: {}", review.term_id));
                                ui.label(format!("מבקש: {}", review.requested_by));
                                if ui.button("סקור").clicked() {
                                    self.show_review_dialog = true;
                                    self.selected_review = Some(review.request_id.clone());
                                }
                            });
                        }
                    });
                }
            }

            // תצוגת קונפליקטים
            if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                let conflicts = knowledge_manager.get_edit_conflicts();
                if !conflicts.is_empty() {
                    ui.group(|ui| {
                        ui.heading("קונפליקטים פעילים");
                        for (term_id, lock) in conflicts {
                            ui.horizontal(|ui| {
                                ui.label(format!("מונח: {}", term_id));
                                ui.label(format!("נעול על ידי: {}", lock.locked_by));
                                if ui.button("פתור").clicked() {
                                    self.show_conflict_resolution_dialog = true;
                                    self.selected_conflict_term = Some(term_id.clone());
                                }
                            });
                        }
                    });
                }
            }

            // טיפוש מתקדם
            ui.group(|ui| {
                ui.heading("חיפוש מתקדם");
                ui.horizontal(|ui| {
                    ui.label("חיפוש:");
                    ui.text_edit_singleline(&mut self.dictionary_search_text);
                    
                    ui.label("שפה:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.dictionary_search_lang)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.dictionary_search_lang, "he".to_string(), "עברית");
                            ui.selectable_value(&mut self.dictionary_search_lang, "ru".to_string(), "רוסית");
                        });
                });

                ui.checkbox(&mut self.dictionary_search_include_synonyms, "כלול סינונימים");
                ui.checkbox(&mut self.dictionary_search_exact_match, "התאמה מדויקת");

                // בחירת קטגוריות
                if let Ok(dict) = self.technical_dictionary.lock() {
                    ui.collapsing("קטגוריות", |ui| {
                        for category in dict.get_all_categories() {
                            let mut selected = self.selected_categories.contains(&category);
                            if ui.checkbox(&mut selected, &category).changed() {
                                if selected {
                                    self.selected_categories.insert(category.clone());
                                } else {
                                    self.selected_categories.remove(&category);
                                }
                            }
                        }
                    });

                    // בחירת הקשרים
                    ui.collapsing("הקשרים", |ui| {
                        for context in dict.get_all_contexts() {
                            let mut selected = self.selected_contexts.contains(&context);
                            if ui.checkbox(&mut selected, &context).changed() {
                                if selected {
                                    self.selected_contexts.insert(context.clone());
                                } else {
                                    self.selected_contexts.remove(&context);
                                }
                            }
                        }
                    });

                    // בחירת תגיות
                    ui.collapsing("תגיות", |ui| {
                        for tag in dict.get_all_tags() {
                            let mut selected = self.selected_tags.contains(&tag);
                            if ui.checkbox(&mut selected, &tag).changed() {
                                if selected {
                                    self.selected_tags.insert(tag.clone());
                                } else {
                                    self.selected_tags.remove(&tag);
                                }
                            }
                        }
                    });
                }
            });

            // כפתור הוספת מונח חדש
            if ui.button("הוסף מונח חדש").clicked() {
                self.show_add_term_dialog = true;
            }

            // הצגת תוצאות החיפוש
            ui.separator();
            ui.heading("תוצאות");
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Ok(dict) = self.technical_dictionary.lock() {
                    let query = SearchQuery {
                        text: self.dictionary_search_text.clone(),
                        lang: self.dictionary_search_lang.clone(),
                        categories: if self.selected_categories.is_empty() {
                            None
                        } else {
                            Some(self.selected_categories.iter().cloned().collect())
                        },
                        contexts: if self.selected_contexts.is_empty() {
                            None
                        } else {
                            Some(self.selected_contexts.iter().cloned().collect())
                        },
                        tags: if self.selected_tags.is_empty() {
                            None
                        } else {
                            Some(self.selected_tags.iter().cloned().collect())
                        },
                        include_synonyms: self.dictionary_search_include_synonyms,
                        exact_match: self.dictionary_search_exact_match,
                    };

                    let results = dict.search(&query);
                    for term in results {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&term.hebrew).strong());
                                ui.label("→");
                                ui.label(RichText::new(&term.russian).strong());
                                
                                if ui.button("ערוך").clicked() {
                                    self.editing_term = Some(term.clone());
                                    self.show_edit_term_dialog = true;
                                }
                            });

                            if !term.synonyms_he.is_empty() {
                                ui.label(format!("סינונימים בעברית: {}", term.synonyms_he.join(", ")));
                            }
                            if !term.synonyms_ru.is_empty() {
                                ui.label(format!("סינונימים ברוסית: {}", term.synonyms_ru.join(", ")));
                            }
                            if !term.usage_examples.is_empty() {
                                ui.label("דוגמאות שימוש:");
                                for example in &term.usage_examples {
                                    ui.label(format!("• {}", example));
                                }
                            }
                            if let Some(context) = &term.context {
                                ui.label(format!("הקשר: {}", context));
                            }
                            if let Some(category) = &term.category {
                                ui.label(format!("קטגוריה: {}", category));
                            }
                            if !term.tags.is_empty() {
                                ui.label(format!("תגיות: {}", term.tags.iter().cloned().collect::<Vec<_>>().join(", ")));
                            }
                            if let Some(notes) = &term.notes {
                                ui.label(format!("הערות: {}", notes));
                            }
                            ui.label(format!("עודכן לאחרונה: {}", term.last_updated.format("%Y-%m-%d %H:%M:%S")));
                        });
                    }
                }
            });
        });

        // חלון דו-שיח להוספת מונח חדש
        if self.show_add_term_dialog {
            self.show_add_term_dialog_window(ui);
        }

        // חלון דו-שיח לעריכת מונח
        if self.show_edit_term_dialog {
            self.show_edit_term_dialog_window(ui);
        }

        // חלונות דו-שיח לשיתוף ידע
        if self.show_create_version_dialog {
            self.show_create_version_dialog_window(ui);
        }
        if self.show_compare_versions_dialog {
            self.show_compare_versions_dialog_window(ui);
        }
        if self.show_change_history_dialog {
            self.show_change_history_dialog_window(ui);
        }

        // חלונות דו-שיח נוספים
        if self.show_active_users_dialog {
            self.show_active_users_dialog_window(ui);
        }
        if self.show_pending_reviews_dialog {
            self.show_pending_reviews_dialog_window(ui);
        }
        if self.show_conflicts_dialog {
            self.show_conflicts_dialog_window(ui);
        }
        if self.show_review_dialog {
            self.show_review_dialog_window(ui);
        }
        if self.show_conflict_resolution_dialog {
            self.show_conflict_resolution_dialog_window(ui);
        }
    }

    fn show_add_term_dialog_window(&mut self, ui: &mut Ui) {
        egui::Window::new("הוספת מונח חדש")
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("מונח בעברית:");
                    ui.text_edit_singleline(&mut self.new_term.hebrew);
                });
                ui.horizontal(|ui| {
                    ui.label("מונח ברוסית:");
                    ui.text_edit_singleline(&mut self.new_term.russian);
                });
                
                // סינונימים בעברית
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("סינונים בעברית:");
                        ui.text_edit_singleline(&mut self.new_synonym_he);
                        if ui.button("הוסף").clicked() && !self.new_synonym_he.is_empty() {
                            self.new_term.synonyms_he.push(self.new_synonym_he.clone());
                            self.new_synonym_he.clear();
                        }
                    });
                    for (i, synonym) in self.new_term.synonyms_he.clone().iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(synonym);
                            if ui.button("הסר").clicked() {
                                self.new_term.synonyms_he.remove(i);
                            }
                        });
                    }
                });

                // סינונימים ברוסית
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("סינונים ברוסית:");
                        ui.text_edit_singleline(&mut self.new_synonym_ru);
                        if ui.button("הוסף").clicked() && !self.new_synonym_ru.is_empty() {
                            self.new_term.synonyms_ru.push(self.new_synonym_ru.clone());
                            self.new_synonym_ru.clear();
                        }
                    });
                    for (i, synonym) in self.new_term.synonyms_ru.clone().iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(synonym);
                            if ui.button("הסר").clicked() {
                                self.new_term.synonyms_ru.remove(i);
                            }
                        });
                    }
                });

                // דוגמאות שימוש
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("דוגמת שימוש:");
                        ui.text_edit_singleline(&mut self.new_usage_example);
                        if ui.button("הוסף").clicked() && !self.new_usage_example.is_empty() {
                            self.new_term.usage_examples.push(self.new_usage_example.clone());
                            self.new_usage_example.clear();
                        }
                    });
                    for (i, example) in self.new_term.usage_examples.clone().iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(example);
                            if ui.button("הסר").clicked() {
                                self.new_term.usage_examples.remove(i);
                            }
                        });
                    }
                });

                // תגיות
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("תגית:");
                        ui.text_edit_singleline(&mut self.new_tag);
                        if ui.button("הוסף").clicked() && !self.new_tag.is_empty() {
                            self.new_term.tags.insert(self.new_tag.clone());
                            self.new_tag.clear();
                        }
                    });
                    for tag in self.new_term.tags.clone() {
                        ui.horizontal(|ui| {
                            ui.label(&tag);
                            if ui.button("הסר").clicked() {
                                self.new_term.tags.remove(&tag);
                            }
                        });
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("הקשר:");
                    let mut context = self.new_term.context.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut context).changed() {
                        self.new_term.context = Some(context);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("קטגוריה:");
                    let mut category = self.new_term.category.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut category).changed() {
                        self.new_term.category = Some(category);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("הערות:");
                    let mut notes = self.new_term.notes.clone().unwrap_or_default();
                    if ui.text_edit_multiline(&mut notes).changed() {
                        self.new_term.notes = Some(notes);
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("שמור").clicked() {
                        if !self.new_term.hebrew.is_empty() && !self.new_term.russian.is_empty() {
                            if let Ok(mut dict) = self.technical_dictionary.lock() {
                                if let Err(e) = dict.add_term(self.new_term.clone()) {
                                    self.status_message = Some(format!("שגיאה בהוספת המונח: {}", e));
                                } else {
                                    self.status_message = Some("המונח נוסף בהצלחה".to_string());
                                    self.new_term = TechnicalTerm {
                                        hebrew: String::new(),
                                        russian: String::new(),
                                        context: None,
                                        category: None,
                                        notes: None,
                                        synonyms_he: Vec::new(),
                                        synonyms_ru: Vec::new(),
                                        usage_examples: Vec::new(),
                                        tags: HashSet::new(),
                                        last_updated: Utc::now(),
                                    };
                                    self.show_add_term_dialog = false;
                                }
                            }
                        } else {
                            self.status_message = Some("נא למלא את המונח בעברית וברוסית".to_string());
                        }
                    }
                    if ui.button("בטל").clicked() {
                        self.show_add_term_dialog = false;
                    }
                });
            });
    }

    fn show_edit_term_dialog_window(&mut self, ui: &mut Ui) {
        if let Some(term) = &mut self.editing_term {
            egui::Window::new("עריכת מונח")
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("מונח בעברית:");
                        ui.text_edit_singleline(&mut term.hebrew);
                    });
                    ui.horizontal(|ui| {
                        ui.label("מונח ברוסית:");
                        ui.text_edit_singleline(&mut term.russian);
                    });
                    
                    // סינונימים בעברית
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("סינונים בעברית:");
                            ui.text_edit_singleline(&mut self.new_synonym_he);
                            if ui.button("הוסף").clicked() && !self.new_synonym_he.is_empty() {
                                term.synonyms_he.push(self.new_synonym_he.clone());
                                self.new_synonym_he.clear();
                            }
                        });
                        for (i, synonym) in term.synonyms_he.clone().iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(synonym);
                                if ui.button("הסר").clicked() {
                                    term.synonyms_he.remove(i);
                                }
                            });
                        }
                    });

                    // סינונימים ברוסית
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("סינונים ברוסית:");
                            ui.text_edit_singleline(&mut self.new_synonym_ru);
                            if ui.button("הוסף").clicked() && !self.new_synonym_ru.is_empty() {
                                term.synonyms_ru.push(self.new_synonym_ru.clone());
                                self.new_synonym_ru.clear();
                            }
                        });
                        for (i, synonym) in term.synonyms_ru.clone().iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(synonym);
                                if ui.button("הסר").clicked() {
                                    term.synonyms_ru.remove(i);
                                }
                            });
                        }
                    });

                    // דוגמאות שימוש
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("דוגמת שימוש:");
                            ui.text_edit_singleline(&mut self.new_usage_example);
                            if ui.button("הוסף").clicked() && !self.new_usage_example.is_empty() {
                                term.usage_examples.push(self.new_usage_example.clone());
                                self.new_usage_example.clear();
                            }
                        });
                        for (i, example) in term.usage_examples.clone().iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(example);
                                if ui.button("הסר").clicked() {
                                    term.usage_examples.remove(i);
                                }
                            });
                        }
                    });

                    // תגיות
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("תגית:");
                            ui.text_edit_singleline(&mut self.new_tag);
                            if ui.button("הוסף").clicked() && !self.new_tag.is_empty() {
                                term.tags.insert(self.new_tag.clone());
                                self.new_tag.clear();
                            }
                        });
                        for tag in term.tags.clone() {
                            ui.horizontal(|ui| {
                                ui.label(&tag);
                                if ui.button("הסר").clicked() {
                                    term.tags.remove(&tag);
                                }
                            });
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("הקשר:");
                        let mut context = term.context.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut context).changed() {
                            term.context = Some(context);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("קטגוריה:");
                        let mut category = term.category.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut category).changed() {
                            term.category = Some(category);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("הערות:");
                        let mut notes = term.notes.clone().unwrap_or_default();
                        if ui.text_edit_multiline(&mut notes).changed() {
                            term.notes = Some(notes);
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("שמור").clicked() {
                            if !term.hebrew.is_empty() && !term.russian.is_empty() {
                                if let Ok(mut dict) = self.technical_dictionary.lock() {
                                    if let Err(e) = dict.update_term(&term.hebrew, term.clone()) {
                                        self.status_message = Some(format!("שגיאה בעדכון המונח: {}", e));
                                    } else {
                                        self.status_message = Some("המונח עודכן בהצלחה".to_string());
                                        self.show_edit_term_dialog = false;
                                        self.editing_term = None;
                                    }
                                }
                            } else {
                                self.status_message = Some("נא למלא את המונח בעברית וברוסית".to_string());
                            }
                        }
                        if ui.button("בטל").clicked() {
                            self.show_edit_term_dialog = false;
                            self.editing_term = None;
                        }
                    });
                });
        }
    }

    fn show_create_version_dialog_window(&mut self, ui: &mut Ui) {
        egui::Window::new("יצירת גרסה חדשה")
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("מזהה גרסה:");
                    ui.text_edit_singleline(&mut self.new_version_id);
                });
                ui.horizontal(|ui| {
                    ui.label("תיאור:");
                    ui.text_edit_multiline(&mut self.new_version_description);
                });
                ui.horizontal(|ui| {
                    if ui.button("צור").clicked() {
                        if !self.new_version_id.is_empty() {
                            if let Ok(mut dict) = self.technical_dictionary.lock() {
                                if let Ok(mut knowledge_manager) = self.knowledge_manager.lock() {
                                    if let Err(e) = knowledge_manager.create_version(
                                        &dict,
                                        self.new_version_id.clone(),
                                        "משתמש נוכחי".to_string(), // יש להחליף במזהה משתמש אמיתי
                                        self.new_version_description.clone(),
                                    ) {
                                        self.status_message = Some(format!("שגיאה ביצירת גרסה: {}", e));
                                    } else {
                                        self.status_message = Some("הגרסה נוצרה בהצלחה".to_string());
                                        self.show_create_version_dialog = false;
                                        self.new_version_id.clear();
                                        self.new_version_description.clear();
                                    }
                                }
                            }
                        } else {
                            self.status_message = Some("נא להזין מזהה גרסה".to_string());
                        }
                    }
                    if ui.button("בטל").clicked() {
                        self.show_create_version_dialog = false;
                    }
                });
            });
    }

    fn show_compare_versions_dialog_window(&mut self, ui: &mut Ui) {
        egui::Window::new("השוואת גרסאות")
            .resizable(false)
            .show(ui.ctx(), |ui| {
                if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                    let versions = knowledge_manager.get_all_versions();
                    
                    ui.horizontal(|ui| {
                        ui.label("גרסה 1:");
                        egui::ComboBox::from_label("")
                            .selected_text(self.selected_version1.clone().unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for version in &versions {
                                    ui.selectable_value(
                                        &mut self.selected_version1,
                                        Some(version.version_id.clone()),
                                        &version.version_id,
                                    );
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("גרסה 2:");
                        egui::ComboBox::from_label("")
                            .selected_text(self.selected_version2.clone().unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for version in &versions {
                                    ui.selectable_value(
                                        &mut self.selected_version2,
                                        Some(version.version_id.clone()),
                                        &version.version_id,
                                    );
                                }
                            });
                    });

                    if let (Some(v1), Some(v2)) = (&self.selected_version1, &self.selected_version2) {
                        if let Some(report) = knowledge_manager.compare_versions(v1, v2) {
                            ui.separator();
                            ui.heading("תוצאות ההשוואה");
                            
                            ui.collapsing("מונחים חדשים", |ui| {
                                for term in &report.added_terms {
                                    ui.label(format!("• {} → {}", term.hebrew, term.russian));
                                }
                            });
                            
                            ui.collapsing("מונחים שעודכנו", |ui| {
                                for term in &report.updated_terms {
                                    ui.label(format!("• {} → {}", term.hebrew, term.russian));
                                }
                            });
                            
                            ui.collapsing("קונפליקטים", |ui| {
                                for (term1, term2) in &report.conflicting_terms {
                                    ui.group(|ui| {
                                        ui.label(format!("מונח: {}", term1.hebrew));
                                        ui.label("גרסה 1:");
                                        ui.label(format!("  • תרגום: {}", term1.russian));
                                        ui.label("גרסה 2:");
                                        ui.label(format!("  • תרגום: {}", term2.russian));
                                    });
                                }
                            });
                        }
                    }

                    ui.horizontal(|ui| {
                        if ui.button("סגור").clicked() {
                            self.show_compare_versions_dialog = false;
                        }
                    });
                }
            });
    }

    fn show_change_history_dialog_window(&mut self, ui: &mut Ui) {
        egui::Window::new("היסטוריית שינויים")
            .resizable(false)
            .show(ui.ctx(), |ui| {
                if let Ok(dict) = self.technical_dictionary.lock() {
                    if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                        ui.horizontal(|ui| {
                            ui.label("בחר מונח:");
                            egui::ComboBox::from_label("")
                                .selected_text(self.selected_term_for_history.clone().unwrap_or_default())
                                .show_ui(ui, |ui| {
                                    for term in dict.get_all_terms() {
                                        ui.selectable_value(
                                            &mut self.selected_term_for_history,
                                            Some(term.hebrew.clone()),
                                            &term.hebrew,
                                        );
                                    }
                                });
                        });

                        if let Some(term_id) = &self.selected_term_for_history {
                            if let Some(history) = knowledge_manager.get_term_history(term_id) {
                                ui.separator();
                                ui.heading("שינויים");
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    for change in &history.changes {
                                        ui.group(|ui| {
                                            ui.label(format!("תאריך: {}", change.timestamp.format("%Y-%m-%d %H:%M:%S")));
                                            ui.label(format!("משתמש: {}", change.changed_by));
                                            ui.label(format!("שדה: {}", change.field));
                                            match change.change_type {
                                                ChangeType::Addition => {
                                                    ui.label("סוג: הוספה");
                                                    if let Some(value) = &change.new_value {
                                                        ui.label(format!("ערך: {}", value));
                                                    }
                                                }
                                                ChangeType::Modification => {
                                                    ui.label("סוג: עדכון");
                                                    if let Some(old) = &change.old_value {
                                                        ui.label(format!("ערך קודם: {}", old));
                                                    }
                                                    if let Some(new) = &change.new_value {
                                                        ui.label(format!("ערך חדש: {}", new));
                                                    }
                                                }
                                                ChangeType::Deletion => {
                                                    ui.label("סוג: מחיקה");
                                                    if let Some(value) = &change.old_value {
                                                        ui.label(format!("ערך שנמחק: {}", value));
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            } else {
                                ui.label("אין היסטוריית שינויים למונח זה");
                            }
                        }

                        ui.horizontal(|ui| {
                            if ui.button("סגור").clicked() {
                                self.show_change_history_dialog = false;
                            }
                        });
                    }
                }
            });
    }

    fn show_active_users_dialog_window(&mut self, ui: &mut Ui) {
        egui::Window::new("משתמשים פעילים")
            .resizable(true)
            .show(ui.ctx(), |ui| {
                if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                    for collaborator in knowledge_manager.get_active_collaborators() {
                        ui.group(|ui| {
                            ui.heading(&collaborator.name);
                            ui.label(format!("תפקיד: {:?}", collaborator.role));
                            ui.label(format!(
                                "פעיל לאחרונה: {}", 
                                collaborator.last_active.format("%H:%M:%S")
                            ));
                            
                            if let Some(activity) = &collaborator.current_activity {
                                ui.label("פעילות נוכחית:");
                                ui.indent("activity", |ui| {
                                    ui.label(format!("סוג: {:?}", activity.activity_type));
                                    if let Some(term_id) = &activity.term_id {
                                        ui.label(format!("מונח: {}", term_id));
                                    }
                                    ui.label(format!("סטטוס: {:?}", activity.status));
                                });
                            }

                            ui.collapsing("היסטוריית פעילות", |ui| {
                                for change in &collaborator.edit_history {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{} - {} - {}",
                                            change.timestamp.format("%H:%M:%S"),
                                            change.field,
                                            change.change_type
                                        ));
                                    });
                                }
                            });
                        });
                    }
                }

                if ui.button("סגור").clicked() {
                    self.show_active_users_dialog = false;
                }
            });
    }

    fn show_review_dialog_window(&mut self, ui: &mut Ui) {
        if let Some(review_id) = &self.selected_review {
            egui::Window::new("סקירת מונח")
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    if let Ok(knowledge_manager) = self.knowledge_manager.lock() {
                        if let Some(review) = knowledge_manager.get_review_request(review_id) {
                            ui.heading(format!("סקירת מונח: {}", review.term_id));
                            
                            // הצגת המונח הנוכחי
                            if let Ok(dict) = self.technical_dictionary.lock() {
                                if let Some(term) = dict.get_term(&review.term_id) {
                                    ui.group(|ui| {
                                        ui.label(format!("עברית: {}", term.hebrew));
                                        ui.label(format!("רוסית: {}", term.russian));
                                        if let Some(context) = &term.context {
                                            ui.label(format!("הקשר: {}", context));
                                        }
                                        if let Some(category) = &term.category {
                                            ui.label(format!("קטגוריה: {}", category));
                                        }
                                    });
                                }
                            }

                            // הערות קודמות
                            ui.group(|ui| {
                                ui.heading("הערות קודמות");
                                for comment in &review.comments {
                                    ui.horizontal(|ui| {
                                        ui.label(format!(
                                            "{} - {} - {}",
                                            comment.timestamp.format("%H:%M:%S"),
                                            comment.author,
                                            comment.content
                                        ));
                                    });
                                }
                            });

                            // הוספת הערה חדשה
                            ui.group(|ui| {
                                ui.heading("הוסף הערה");
                                ui.text_edit_multiline(&mut self.new_review_comment);
                                if ui.button("שלח הערה").clicked() && !self.new_review_comment.is_empty() {
                                    if let Ok(mut km) = self.knowledge_manager.lock() {
                                        if let Err(e) = km.add_review_comment(
                                            review_id,
                                            self.current_user_id.clone(),
                                            self.new_review_comment.clone(),
                                            None,
                                        ) {
                                            self.status_message = Some(
                                                format!("שגיאה בהוספת הערה: {}", e)
                                            );
                                        } else {
                                            self.new_review_comment.clear();
                                        }
                                    }
                                }
                            });

                            // כפתורי פעולה
                            ui.horizontal(|ui| {
                                if ui.button("אשר").clicked() {
                                    if let Ok(mut km) = self.knowledge_manager.lock() {
                                        if let Err(e) = km.update_review_status(
                                            review_id,
                                            ReviewStatus::Approved,
                                        ) {
                                            self.status_message = Some(
                                                format!("שגיאה באישור הסקירה: {}", e)
                                            );
                                        } else {
                                            self.show_review_dialog = false;
                                        }
                                    }
                                }
                                if ui.button("דחה").clicked() {
                                    if let Ok(mut km) = self.knowledge_manager.lock() {
                                        if let Err(e) = km.update_review_status(
                                            review_id,
                                            ReviewStatus::Rejected,
                                        ) {
                                            self.status_message = Some(
                                                format!("שגיאה בדחיית הסקירה: {}", e)
                                            );
                                        } else {
                                            self.show_review_dialog = false;
                                        }
                                    }
                                }
                                if ui.button("דרוש שינויים").clicked() {
                                    if let Ok(mut km) = self.knowledge_manager.lock() {
                                        if let Err(e) = km.update_review_status(
                                            review_id,
                                            ReviewStatus::NeedsChanges,
                                        ) {
                                            self.status_message = Some(
                                                format!("שגיאה בעדכון סטטוס הסקירה: {}", e)
                                            );
                                        } else {
                                            self.show_review_dialog = false;
                                        }
                                    }
                                }
                                if ui.button("סגור").clicked() {
                                    self.show_review_dialog = false;
                                }
                            });
                        }
                    }
                });
        }
    }

    fn show_conflict_resolution_dialog_window(&mut self, ui: &mut Ui) {
        if let Some(term_id) = &self.selected_conflict_term {
            egui::Window::new("פתרון קונפליקט")
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    ui.heading(format!("פתרון קונפליקט עבור מונח: {}", term_id));
                    
                    ui.group(|ui| {
                        ui.radio_value(
                            &mut self.selected_resolution_type,
                            ResolutionType::KeepBase,
                            "השאר את הגרסה הבסיסית"
                        );
                        ui.radio_value(
                            &mut self.selected_resolution_type,
                            ResolutionType::AcceptChanges,
                            "קבל את השינויים"
                        );
                        ui.radio_value(
                            &mut self.selected_resolution_type,
                            ResolutionType::Merge,
                            "מזג את השינויים"
                        );
                        ui.radio_value(
                            &mut self.selected_resolution_type,
                            ResolutionType::Custom,
                            "פתרון מותאם אישית"
                        );
                    });

                    ui.label("הערות לפתרון:");
                    ui.text_edit_multiline(&mut self.conflict_resolution_comments);

                    ui.horizontal(|ui| {
                        if ui.button("פתור").clicked() {
                            if let Ok(mut km) = self.knowledge_manager.lock() {
                                if let Err(e) = km.resolve_conflict(
                                    term_id.clone(),
                                    self.current_user_id.clone(),
                                    self.selected_resolution_type.clone(),
                                    self.conflict_resolution_comments.clone(),
                                ) {
                                    self.status_message = Some(
                                        format!("שגיאה בפתרון הקונפליקט: {}", e)
                                    );
                                } else {
                                    self.show_conflict_resolution_dialog = false;
                                    self.conflict_resolution_comments.clear();
                                }
                            }
                        }
                        if ui.button("בטל").clicked() {
                            self.show_conflict_resolution_dialog = false;
                        }
                    });
                });
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_header(ui);
            
            ui.add_space(10.0);
            self.show_template_selector(ui);
            
            ui.add_space(10.0);
            self.show_language_selector(ui);
            
            ui.add_space(10.0);
            self.show_template_values(ui);
            
            ui.add_space(10.0);
            self.show_actions(ui);

            ui.add_space(10.0);
            self.show_dictionary_panel(ui);
            
            ui.add_space(10.0);
            self.show_status(ui);
        });
    }
} 