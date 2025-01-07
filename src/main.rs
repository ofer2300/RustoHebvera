use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;

mod translation;
mod technical_terms;
mod standards;
mod document_processor;
mod file_processor;
mod fonts;
mod metadata;
mod templates;
mod template_translator;
mod gui;
mod technical_dictionary;
mod knowledge_sharing;

use translation::Translator;
use templates::TemplateManager;
use template_translator::TemplateTranslator;
use gui::RustoHebruApp;
use technical_dictionary::TechnicalDictionary;
use knowledge_sharing::KnowledgeManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // יצירת מסדי נתונים
    let terms_db = Arc::new(create_initial_terms());
    let standards_db = Arc::new(create_initial_standards());
    
    // יצירת המילון הטכני
    let technical_dictionary = Arc::new(Mutex::new(
        TechnicalDictionary::new("technical_dictionary.json".to_string())?
    ));
    
    // יצירת מנהל הידע
    let knowledge_manager = Arc::new(Mutex::new(KnowledgeManager::new()));
    
    // יצירת מנהל תבניות
    let template_manager = Arc::new(TemplateManager::new("templates".to_string())?);
    
    // יצירת מתרגם תבניות
    let translator = Translator::new(
        terms_db.clone(),
        standards_db.clone(),
        technical_dictionary.clone(),
    );
    let template_translator = Arc::new(TemplateTranslator::new(translator));
    
    // הגדרות חלון
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        min_window_size: Some(egui::vec2(400.0, 300.0)),
        centered: true,
        ..Default::default()
    };

    // הפעלת הממשק הגרפי
    eframe::run_native(
        "RustoHebru",
        native_options,
        Box::new(|_cc| Box::new(RustoHebruApp::new(
            template_manager.clone(),
            template_translator.clone(),
            technical_dictionary.clone(),
            knowledge_manager.clone(),
        ))),
    ).map_err(|e| anyhow::anyhow!("שגיאה בהפעלת הממשק הגרפי: {}", e))?;
    
    Ok(())
} 