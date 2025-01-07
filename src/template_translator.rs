use crate::templates::{DocumentTemplate, TemplateSection, PlaceholderType};
use crate::translation::Translator;
use anyhow::{Result, Context};
use std::collections::HashMap;

pub struct TemplateTranslator {
    translator: Translator,
    cache: HashMap<String, String>,
}

impl TemplateTranslator {
    pub fn new(translator: Translator) -> Self {
        Self {
            translator,
            cache: HashMap::new(),
        }
    }

    pub fn translate_template(
        &mut self,
        template: &DocumentTemplate,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<DocumentTemplate> {
        if !template.supported_languages.contains(&source_lang.to_string()) ||
           !template.supported_languages.contains(&target_lang.to_string()) {
            anyhow::bail!("שפת המקור או היעד אינה נתמכת בתבנית זו");
        }

        let mut translated = template.clone();
        
        // תרגום מידע בסיסי
        translated.description = self.translate_text(&template.description, source_lang, target_lang)?;
        
        // תרגום סעיפים
        translated.sections = self.translate_sections(&template.sections, source_lang, target_lang)?;
        
        // עדכון הגדרות שפה
        translated.default_language = target_lang.to_string();
        translated.rtl = is_rtl_language(target_lang);
        
        Ok(translated)
    }

    fn translate_sections(
        &mut self,
        sections: &[TemplateSection],
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<TemplateSection>> {
        let mut translated_sections = Vec::new();
        
        for section in sections {
            let mut translated_section = section.clone();
            
            // תרגום כותרת
            translated_section.title = self.translate_text(&section.title, source_lang, target_lang)?;
            
            // תרגום תוכן
            let content = self.translate_content(
                &section.content,
                &section.id,
                source_lang,
                target_lang
            )?;
            translated_section.content = content;
            
            // תרגום תתי-סעיפים
            if !section.subsections.is_empty() {
                translated_section.subsections = self.translate_sections(
                    &section.subsections,
                    source_lang,
                    target_lang
                )?;
            }
            
            translated_sections.push(translated_section);
        }
        
        Ok(translated_sections)
    }

    fn translate_text(&mut self, text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        // בדיקה אם התרגום קיים במטמון
        let cache_key = format!("{}:{}:{}", text, source_lang, target_lang);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // תרגום הטקסט
        let translated = self.translator.translate_with_languages(
            text,
            source_lang,
            target_lang
        )?;
        
        // שמירה במטמון
        self.cache.insert(cache_key, translated.clone());
        
        Ok(translated)
    }

    fn translate_content(
        &mut self,
        content: &str,
        section_id: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        // פיצול התוכן לחלקים קבועים ומשתנים
        let parts: Vec<&str> = content.split("{{").collect();
        let mut translated_content = String::new();
        
        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                // החלק הראשון הוא תמיד טקסט קבוע
                if !part.is_empty() {
                    translated_content.push_str(&self.translate_text(part, source_lang, target_lang)?);
                }
            } else {
                // חלקים נוספים מכילים placeholders
                if let Some((placeholder, text)) = part.split_once("}}") {
                    // שמירה על ה-placeholder
                    translated_content.push_str("{{");
                    translated_content.push_str(placeholder);
                    translated_content.push_str("}}");
                    
                    // תרגום הטקסט שאחרי ה-placeholder
                    if !text.is_empty() {
                        translated_content.push_str(&self.translate_text(text, source_lang, target_lang)?);
                    }
                }
            }
        }
        
        Ok(translated_content)
    }

    pub fn translate_values(
        &mut self,
        values: &HashMap<String, String>,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<HashMap<String, String>> {
        let mut translated_values = HashMap::new();
        
        for (key, value) in values {
            let translated_value = self.translate_text(value, source_lang, target_lang)?;
            translated_values.insert(key.clone(), translated_value);
        }
        
        Ok(translated_values)
    }
}

fn is_rtl_language(lang: &str) -> bool {
    matches!(lang, "he" | "ar" | "fa")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technical_terms::create_initial_terms;
    use crate::standards::create_initial_standards;

    #[test]
    fn test_template_translation() {
        let terms_db = create_initial_terms();
        let standards_db = create_initial_standards();
        let translator = Translator::new(terms_db, standards_db);
        let mut template_translator = TemplateTranslator::new(translator);

        // יצירת תבנית לדוגמה
        let template = DocumentTemplate {
            name: "test_template".to_string(),
            description: "תבנית בדיקה".to_string(),
            version: "1.0.0".to_string(),
            template_type: crate::templates::TemplateType::TechnicalSpec,
            sections: vec![
                TemplateSection {
                    id: "test_section".to_string(),
                    title: "כותרת בדיקה".to_string(),
                    content: "תוכן בדיקה {{placeholder}}".to_string(),
                    required: true,
                    order: 1,
                    style: "default".to_string(),
                    subsections: vec![],
                },
            ],
            placeholders: HashMap::new(),
            styles: crate::templates::DocumentStyles {
                fonts: HashMap::new(),
                colors: HashMap::new(),
                spacing: crate::templates::SpacingStyle {
                    line_spacing: 1.0,
                    paragraph_spacing: 1.0,
                    margin_top: 0.0,
                    margin_bottom: 0.0,
                    margin_left: 0.0,
                    margin_right: 0.0,
                },
                page_layout: crate::templates::PageLayout {
                    size: "A4".to_string(),
                    orientation: "portrait".to_string(),
                    margins: crate::templates::SpacingStyle {
                        line_spacing: 1.0,
                        paragraph_spacing: 1.0,
                        margin_top: 0.0,
                        margin_bottom: 0.0,
                        margin_left: 0.0,
                        margin_right: 0.0,
                    },
                    header_height: 0.0,
                    footer_height: 0.0,
                },
            },
            rtl: true,
            default_language: "he".to_string(),
            supported_languages: vec!["he".to_string(), "ru".to_string()],
            default_metadata: None,
        };

        // תרגום התבנית
        let result = template_translator.translate_template(&template, "he", "ru");
        assert!(result.is_ok());

        let translated = result.unwrap();
        assert_eq!(translated.default_language, "ru");
        assert!(!translated.rtl);
    }
} 