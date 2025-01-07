use crate::technical_terms::TermsDatabase;
use crate::standards::StandardsDatabase;
use crate::technical_dictionary::TechnicalDictionary;
use whatlang::detect;
use anyhow::{Result, Context};
use rust_bert::pipelines::translation::{Language, TranslationModelBuilder};
use std::sync::Arc;
use std::sync::Mutex;

pub struct Translator {
    terms_db: Arc<TermsDatabase>,
    standards_db: Arc<StandardsDatabase>,
    technical_dictionary: Arc<Mutex<TechnicalDictionary>>,
    model: Option<TranslationModelBuilder>,
}

impl Translator {
    pub fn new(
        terms_db: Arc<TermsDatabase>,
        standards_db: Arc<StandardsDatabase>,
        technical_dictionary: Arc<Mutex<TechnicalDictionary>>,
    ) -> Self {
        Self {
            terms_db,
            standards_db,
            technical_dictionary,
            model: None,
        }
    }

    pub fn translate(&self, text: &str) -> Result<String> {
        // זיהוי שפת המקור
        let lang_info = detect(text)
            .context("לא ניתן לזהות את שפת המקור")?;
        
        match lang_info.lang().code() {
            "he" => self.translate_with_languages(text, "he", "ru"),
            "ru" => self.translate_with_languages(text, "ru", "he"),
            _ => anyhow::bail!("שפה לא נתמכת: {}", lang_info.lang().code()),
        }
    }

    pub fn translate_with_languages(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        // בדיקה במילון הטכני
        if let Ok(dict) = self.technical_dictionary.lock() {
            let translated = dict.translate(text, source_lang, target_lang);
            if translated != text {
                return Ok(translated);
            }
        }
        
        // בדיקה אם מדובר במונח טכני ממסד הנתונים הקבוע
        if let Some(term) = self.terms_db.get_translation(text) {
            return Ok(term.to_string());
        }
        
        // בדיקה אם מדובר בתקן
        if let Some(standard) = self.standards_db.get_standard_by_code(text) {
            return Ok(standard.name.to_string());
        }
        
        // פיצול הטקסט למשפטים
        let sentences: Vec<&str> = text.split(['.', '!', '?', '\n'])
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        let mut translated = String::new();
        
        for sentence in sentences {
            // ניקוי רווחים מיותרים
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            
            // תרגום המשפט
            let translated_sentence = match (source_lang, target_lang) {
                ("he", "ru") => self.translate_he_to_ru(sentence)?,
                ("ru", "he") => self.translate_ru_to_he(sentence)?,
                _ => anyhow::bail!("צמד השפות לא נתמך: {} -> {}", source_lang, target_lang),
            };
            
            // הוספת המשפט המתורגם
            if !translated.is_empty() {
                translated.push(' ');
            }
            translated.push_str(&translated_sentence);
            translated.push('.');
        }
        
        Ok(translated)
    }

    fn translate_he_to_ru(&self, text: &str) -> Result<String> {
        // כאן יש להוסיף את הלוגיקה הספציפית לתרגום מעברית לרוסית
        // לדוגמה, שימוש במודל נוירונים או שירות תרגום חיצוני
        
        // לצורך הדוגמה, נשתמש בתרגום פשוט
        let translated = match text {
            "תקין" => "исправен",
            "לא נמצאו" => "не обнаружены",
            "נדרש ניקוי שנתי" => "требуется ежегодная очистка",
            "בוצע כיול שנתי" => "выполнена ежегодная калибровка",
            "צביעה מחדש נדרשת בעוד כשנה" => "перекраска требуется через год",
            "מערכת ספרינקלרים אוטומטית" => "автоматическая спринклерная система",
            "תל אביב" => "Тель-Авив",
            "ישראל ישראלי" => "Исраэль Исраэли",
            _ => text, // במקרה של טקסט לא מוכר, נחזיר את המקור
        };
        
        Ok(translated.to_string())
    }

    fn translate_ru_to_he(&self, text: &str) -> Result<String> {
        // כאן יש להוסיף את הלוגיקה הספציפית לתרגום מרוסית לעברית
        // לדוגמה, שימוש במודל נוירונים או שירות תרגום חיצוני
        
        // לצורך הדוגמה, נשתמש בתרגום פשוט
        let translated = match text {
            "исправен" => "תקין",
            "не обнаружены" => "לא נמצאו",
            "требуется ежегодная очистка" => "נדרש ניקוי שנתי",
            "выполнена ежегодная калибровка" => "בוצע כיול שנתי",
            "перекраска требуется через год" => "צביעה מחדש נדרשת בעוד כשנה",
            "автоматическая спринклерная система" => "מערכת ספרינקלרים אוטומטית",
            "Тель-Авив" => "תל אביב",
            "Исраэль Исраэли" => "ישראל ישראלי",
            _ => text, // במקרה של טקסט לא מוכר, נחזיר את המקור
        };
        
        Ok(translated.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technical_terms::create_initial_terms;
    use crate::standards::create_initial_standards;

    #[test]
    fn test_translation() {
        let terms_db = create_initial_terms();
        let standards_db = create_initial_standards();
        let translator = Translator::new(terms_db, standards_db);

        // בדיקת תרגום מעברית לרוסית
        let hebrew_text = "תקין";
        let result = translator.translate_with_languages(hebrew_text, "he", "ru");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "исправен");

        // בדיקת תרגום מרוסית לעברית
        let russian_text = "исправен";
        let result = translator.translate_with_languages(russian_text, "ru", "he");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "תקין");
    }
} 